# Translations

omikuji's UI strings are wrapped in Qt's `qsTr()`. A `QTranslator` loads a compiled `.qm` at startup and resolves them; an unwrapped or untranslated string falls back to its English source. Translations live in `crates/omikuji/i18n/`, one `.ts` per language. `build.rs` compiles each `.ts` to a `.qm` and embeds it, so only the `.ts` is committed. The scope is the QML UI: the CLI and the Rust backend stay in English (aint doing allat).

`omikuji_en.ts` is the English source catalog. It is the template new languages are created from, and it is not compiled or shipped.

## Weblate

Translations are hosted on [Weblate](https://hosted.weblate.org/projects/omikuji/omikuji/). Pick a language or start a new one, translate in the browser, and Weblate opens a pull request against `master`. No local setup, no Qt tools.

## Prerequisites

For working on the catalogs locally: the Qt Linguist tools, `lupdate` and `lrelease`. On Arch they ship in `qt6-tools` (as `lupdate6` / `lrelease6`), idk other distros. `scripts/update-translations.sh` accepts either the suffixed or the plain name. Without them the build still works and ships English only.

## Adding a language locally

As an example, for Italian (`it`):

1. `./scripts/update-translations.sh it` harvests every `qsTr`/`tr` string into `crates/omikuji/i18n/omikuji_it.ts`. The file name carries the locale code Qt expects (`omikuji_it.ts`, `omikuji_pt_BR.ts`, `omikuji_ja.ts`).

2. Translate `omikuji_it.ts`, in either any text editor, by filling the `<translation>` elements, or `QtLinguistic` if you're sane.

3. Build. The language appears in the picker at Settings > Interface under its own native name.

4. Commit `omikuji_it.ts`.

## Updating a language

After UI strings change, refresh the catalogs:

`./scripts/update-translations.sh` with no arguments re-harvests every language already in `i18n/`. New strings land in the `.ts` marked unfinished. Commit the updated `.ts`.

Weblate picks the new strings up on its next pull and marks them for translation. Changed source strings mark the existing translation as needing update.

## Wrapping a new UI string

Any user-visible literal in a `.qml` file goes through `qsTr`:

```qml
text: qsTr("Play")
```

Text combined with data uses placeholders, not concatenation, because word order differs between languages:

```qml
text: qsTr("%1 left").arg(formatEta(secs))
text: qsTr("%n game(s)", "", count)
```

`%n` picks the plural form for the count.

Left unwrapped: icon names (`name:`, `icon:`), color tokens, runner and kind values (`"wine"`, `"native"`, `"steam"`), any literal used in a comparison, config keys, paths, and bare brand names (Steam, Epic Games, GOG, Proton, DXVK). A brand inside a sentence stays literal while the sentence is wrapped, as in `qsTr("Run with Omikuji")`.
