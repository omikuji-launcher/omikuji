# The cxx-qt bridge

A bridge is a QObject whose state and methods live in Rust. QML instantiates it, reads its properties, calls its methods, and reacts to its signals. Each bridge is one file in `crates/omikuji/src/bridge/`.

## Anatomy

A bridge has three parts in the same file: the bridge module, the Rust state struct, and the impl.

```rust
#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(i32, count)]
        type Counter = super::CounterRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        fn increment(self: Pin<&mut Counter>);
    }
}

pub struct CounterRust {
    pub count: i32,
}

impl Default for CounterRust {
    fn default() -> Self {
        Self { count: 0 }
    }
}

impl qobject::Counter {
    fn increment(mut self: Pin<&mut Self>) {
        let next = self.count + 1;
        self.as_mut().set_count(next);
    }
}
```

- `#[qobject]` + `#[qml_element]` declare the QObject and make it instantiable from QML.
- `type Counter = super::CounterRust;` ties the QObject to the plain Rust struct that holds its state.
- The first `extern "RustQt"` block declares properties. The `unsafe extern "RustQt"` block declares signals and invokables.
- `impl qobject::Counter` holds the method bodies.

## Wiring it up

`build.rs` has two lists. A new handwritten bridge `.rs` goes in the `kushi::stage_files([...])` list, a new `.qml` goes in `qml_files([...])`.

```rust
let staged = kushi::stage_files(
    [
        "src/bridge/counter.rs",
        // ...
    ],
    &out_dir,
);
```

The staging copies each file into `OUT_DIR` before handing it to cxx-qt: cxx-qt-build accepts only one directory per QML module ([QTBUG-93443](https://bugreports.qt.io/browse/QTBUG-93443)), and the generated bridges already live there. rustc compiles the originals in `src/bridge/` as normal modules; the copies only feed cxx-qt's parser.

Forgetting to register a new `.qml` is the usual trip: use `MyWidget {}` somewhere and you get "MyWidget is not a type" at runtime, with no compile error and nothing from `qmllint`. Add the file to `qml_files` and rebuild.

Once registered, QML imports the module and instantiates the type:

```qml
import omikuji 1.0

Counter { id: counter }
```

The module name is set in `build.rs` with `QmlModule::new("omikuji")`.

## Properties, signals, invokables

**Properties.** `#[qproperty(T, name)]` generates a getter, a setter (`set_name`), and a `name_changed` signal. QML binds to the property and re-evaluates when the signal fires. `cxx_name` sets the name QML sees:

```rust
#[qproperty(bool, is_logged_in, cxx_name = "isLoggedIn")]
```

Rust calls it `is_logged_in`, QML sees `isLoggedIn`. Without `cxx_name`, the Rust name is used as-is.

**Signals.** Declared with `#[qsignal]` in an `unsafe extern "RustQt"` block. You emit one by calling it on `self`. QML listens with `onNameChanged` handlers or a `Connections` block.

**Invokables.** `#[qinvokable]` exposes a method to QML. Invokables are *not* auto-camelCased the way properties are: without `cxx_name`, QML calls the snake_case Rust name.

```rust
#[qinvokable]
fn get_login_url(self: &Counter) -> QString;   // QML: counter.get_login_url()

#[qinvokable]
#[cxx_name = "installAll"]
fn install_all(self: Pin<&mut Counter>);        // QML: counter.installAll()
```

A property is `isLoggedIn` but a plain invokable next to it stays `get_login_url`. If you want camelCase on an invokable, give it a `cxx_name`.

## Reading and writing state

Methods that read take `&self`, methods that mutate take `Pin<&mut Self>`. The struct fields are reachable directly, and there are helpers for the rest:

```rust
fn example(mut self: Pin<&mut Self>) {
    let current = self.count;                       // read a field directly
    self.as_mut().set_count(current + 1);           // property setter, fires count_changed
    self.as_mut().rust_mut().get_mut().count = 0;   // write the field directly, no signal
}
```

Reach for `set_*` when QML needs to react to the change. Use `rust_mut().get_mut()` for fields that aren't properties, or for batch edits that fire one signal at the end. `self.rust()` returns the whole struct. `self.as_mut()` reborrows the pin for chaining further calls.

## The paperwork

Adding one user-visible value is more steps than it looks. A property that persists needs a field on the Rust struct, the `#[qproperty]` line, an entry wherever the struct is built (`Default`, or a `from_settings`), and usually an invokable that sets the value and writes it to disk. Settings also mirror into a core struct, so the same field name shows up in the core type, the bridge struct, and two or three conversion functions.

That repetition is handled two ways. For settings and the download model, the whole mirror is generated from a declaration; see [Generated bridges](generated.md). In the handwritten bridges, `macro_rules!` does the same job in-file: `defaults_fields!` and `game_fields!` build whole families of getters and setters from a single field table. The rule of thumb: when the only thing that changes between copies is a name (a field, a type, a signal), that is generation's job, not copy-paste. [Adding things](adding.md) has the full add-a-setting walkthrough.

## Threading

A bridge object lives on the Qt thread and can only be touched there. To update it from other work, grab a handle first, move it into the work, then queue a closure back onto the Qt thread:

```rust
fn refresh(mut self: Pin<&mut Self>) {
    let handle = self.as_mut().qt_thread();
    tokio::spawn(async move {
        let value = fetch_something().await;
        let _ = handle.queue(move |mut obj: Pin<&mut qobject::Counter>| {
            obj.as_mut().set_count(value);
        });
    });
}
```

`qt_thread()` needs `impl cxx_qt::Threading for Counter {}` in the bridge module. The closure given to `queue` runs back on the Qt thread with a fresh `Pin<&mut>` to the object, which is the only place `set_*` is safe to call.

## Blocking on async

`main` is `#[tokio::main]`, so invokable bodies run inside the runtime, and calling `block_on` there panics with "cannot start a runtime from within a runtime". When you need a result synchronously, run the work on a separate OS thread with its own runtime:

```rust
std::thread::spawn(move || {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        // ...
    });
});
```

## Binding bridges into delegates

Inside a QML `Component` (a `Repeater`/`ListView` delegate, a `Loader` source), a binding like `model: gameModel` resolves the right-hand `gameModel` to a property on the delegate itself, which is null, not the outer `id`. The fix used throughout `Main.qml` is a `*Ref` alias on the root:

```qml
readonly property var gameModelRef: gameModel

// inside a delegate:
SomeItem {
    model: root.gameModelRef
}
```

Bind bridges into delegates through these aliases, never by the bare same name.

## List models

A bridge that backs a QML list (the library grid, the stores, downloads) is a `QAbstractListModel`. It adds `#[base = QAbstractListModel]` and overrides three methods:

```rust
extern "RustQt" {
    #[qobject]
    #[qml_element]
    #[base = QAbstractListModel]
    type GameList = super::GameListRust;
}

unsafe extern "RustQt" {
    #[cxx_name = "rowCount"]
    #[cxx_override]
    fn row_count(self: &GameList, parent: &QModelIndex) -> i32;

    #[cxx_override]
    fn data(self: &GameList, index: &QModelIndex, role: i32) -> QVariant;

    #[cxx_name = "roleNames"]
    #[cxx_override]
    fn role_names(self: &GameList) -> QHash_i32_QByteArray;
}
```

Roles are an enum, and `role_names` maps each to the name QML reads (`model.title`, `model.banner`, ...):

```rust
enum Role {
    Title = 0,
    Banner = 1,
}

// role_names:
roles.insert_clone(&(Role::Title as i32), &QByteArray::from("title"));

// data:
match role {
    r if r == Role::Title as i32 => QVariant::from(&QString::from(&item.title)),
    _ => QVariant::default(),
}
```

Mutating the backing data has to be wrapped in the inherited model signals or QML won't update. A full swap uses reset:

```rust
self.as_mut().begin_reset_model();
self.as_mut().rust_mut().get_mut().items = new_items;
self.as_mut().end_reset_model();
```

For finer changes there's `begin_insert_rows(&QModelIndex::default(), row, row)` / `end_insert_rows`, and `data_changed(&index, &index, &roles)` to repaint one row. These inherited methods (`begin_reset_model`, `begin_insert_rows`, `data_changed`, `model_index`, ...) are declared with `#[inherit]` in their own `unsafe extern "RustQt"` block before use. When the data comes from async work, run the fetch off-thread and do the reset inside `qt_thread().queue(...)`.


