# Interface (`app.toml`)

UI preferences live in `~/.local/share/omikuji/app.toml`: categories, nav rail, tab visibility, zoom, theme. Almost all of it is set through the app, and the file is live-watched, so edits apply without a restart.

A couple of theme knobs have no control in the UI and can only be set by editing `app.toml`:

```toml
[theme]
fill_fields = true
radius_scale = 1.0
```

- **`fill_fields`** (default `true`): text fields, dropdowns, and file pickers render as filled containers. Set it `false` for the older outline-only look.
- **`radius_scale`** (default `1.0`): a multiplier on the corner radii across the app. Above `1.0` rounds everything more, below `1.0` sharpens it.

Everything else in `app.toml` mirrors a control in the app, so use those instead of hand-editing.
