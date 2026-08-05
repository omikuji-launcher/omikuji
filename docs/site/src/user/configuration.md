# Configuration

Main config lives at `~/.local/share/omikuji/settings.toml`, auto-generated on first run. Edit it and restart to apply (only `ui.toml` is live-watched).

Most sections rarely need touching. The two worth knowing are `[[runners]]` and `[[dll_packs]]`, which let you add your own wine/proton/DXVK sources without touching code.

## `[paths]`

Where omikuji keeps its data. leave them unless you have a reason.

```toml
[paths]
data_dir = "~/.local/share/omikuji"
library_dir = "~/.local/share/omikuji/library"
gachas_dir = "~/.local/share/omikuji/gachas"
runners_dir = "~/.local/share/omikuji/runners"
dll_packs_dir = "~/.local/share/omikuji/components"
prefixes_dir = "~/.local/share/omikuji/prefixes"
cache_dir = "~/.local/share/omikuji/cache"
logs_dir = "~/.local/share/omikuji/logs"
runtime_dir = "~/.local/share/omikuji/runtime"
```

A leading `~` is expanded to `$HOME` on read (crazy right?).

## `[assets]`

Where gacha manifests and artwork are fetched from.

```toml
[assets]
fetch_url = "https://raw.githubusercontent.com/reakjra/omikuji-assets/main"
```

Point it at a fork to add your own gachas. hmph. 

## `[components]`

Download URLs for the runtime tools (umu, hpatchz, legendary, gogdl, EGL dummy). `umu-run` is fetched on first launch; the rest are fetched when first needed, like a store login or a game install.

```toml
[components]
umu_run = "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases/latest"
hpatchz = "https://api.github.com/repos/sisong/HDiffPatch/releases/latest"
legendary = "https://api.github.com/repos/derrod/legendary/releases/latest"
gogdl = "https://api.github.com/repos/Heroic-Games-Launcher/heroic-gogdl/releases/latest"
egl_dummy = "https://raw.githubusercontent.com/reakjra/omikuji-assets/main/runtime/epic/EpicGamesLauncher.exe"
```

Leave them unless an upstream tool moves repos.

## `[steam]`

```toml
[steam]
api_key = ""
```

Optional Steam Web API key ([get one here](https://steamcommunity.com/dev/apikey)). Without it, Steam library listing still works (read locally from ACF files), only remote playtime sync is off. Playtime for games launched through omikuji is tracked either way.

## `[[runners]]`

The sources the runner manager pulls wine/proton from. Each entry points at a releases API plus a pattern to match the right asset.

| field | meaning |
|-------|---------|
| `name` | display name in the runner manager. arbitrary. |
| `kind` | `"wine"` or `"proton"`. drives variant detection at launch. |
| `api_url` | a GitHub releases API (`https://api.github.com/repos/{owner}/{repo}/releases`). Gitea/Codeberg instances work too if they expose the same JSON. |
| `asset_pattern` | substring matched against each asset filename, first match wins. specific enough to skip `.sha256sum` and the like. |
| `extract` | `tar_gz` / `tar_xz` / `tar_zst` / `zip`. |

Ships with Proton-Spritz, Proton-GE, Dawn Winery Proton, and Proton-Cachyos. To add one, e.g. wine-tkg:

```toml
[[runners]]
name = "Wine-TkG"
kind = "wine"
api_url = "https://api.github.com/repos/Frogging-Family/wine-tkg-git/releases"
asset_pattern = "-x86_64.tar.zst"
extract = "tar_zst"
```

Restart, then Settings => Components lists the new source with its installable versions. A pattern that matches nothing is skipped.

## `[[dll_packs]]`

Same shape as `[[runners]]`, for DXVK / VKD3D-Proton / DXVK-NVAPI. Installed under `components/{name}/{tag}/` so colliding tags don't clobber each other. Ships with DXVK, VKD3D-Proton, and DXVK-NVAPI. `kind` is a free string, used only for grouping in the UI.

Which version of each pack gets auto-injected lives in `components_state.toml`, next to `settings.toml`:

```toml
[dll_packs]
DXVK = "v2.4"
VKD3D-Proton = "v2.13"
DXVK-NVAPI = ""  # empty = disabled
```

The UI manages this, no need to hand-edit unless something's off.
