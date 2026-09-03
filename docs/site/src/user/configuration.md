# Configuration files

Main config lives at `~/.local/share/omikuji/settings.toml`. Edit it and restart to apply (`app.toml` and the others are live-watched).

Most sections rarely need touching. However older installs may be missing some keys added in later versions. The app fallbacks internally, but if you need to override one that misses in your `settings.toml` just check down here.

## `settings.toml`

### `[paths]`

Where omikuji keeps its data.

```toml
# !! data_dir is not changeable! Even if editing the line, it won't actually change it. It's to avoid handling ugly behaviours.

[paths]
data_dir = "/home/reakjra/.local/share/omikuji"
library_dir = "/home/reakjra/.local/share/omikuji/library"
gachas_dir = "/home/reakjra/.local/share/omikuji/gachas"
components_dir = "/home/reakjra/.local/share/omikuji/components"
runners_dir = "/home/reakjra/.local/share/omikuji/components/runners"
layers_dir = "/home/reakjra/.local/share/omikuji/components/layers"
tools_dir = "/home/reakjra/.local/share/omikuji/components/tools"
prefixes_dir = "/home/reakjra/.local/share/omikuji/prefixes"
cache_dir = "/home/reakjra/.local/share/omikuji/cache"
logs_dir = "/home/reakjra/.local/share/omikuji/logs"
runtime_dir = "/home/reakjra/.local/share/omikuji/runtime"
scripts_dir = "/home/reakjra/.local/share/omikuji/scripts"
```

A leading `~` is expanded to `$HOME` on read (crazy right?).

### `[assets]`

Where gacha manifests and artwork are fetched from.

```toml
[assets]
fetch_url = "https://raw.githubusercontent.com/reakjra/omikuji-assets/main"
```

### `[scripts]`

Where the community scripts registry is fetched from. Point it elsewhere to use your own registry instead.

```toml
[scripts]
fetch_url = "https://raw.githubusercontent.com/reakjra/omikuji-scripts/master"
```

### `[components]`

Download URLs for the runtime tools (`umu`, `hpatchz`, `legendary`, `gogdl`, `EGL dummy`). They're fetched when first needed, like a store login. `umu` is never fetched on its own, it'll always prompt.

```toml
[components]
umu_run = "https://api.github.com/repos/Open-Wine-Components/umu-launcher/releases/latest"
hpatchz = "https://api.github.com/repos/sisong/HDiffPatch/releases/latest"
legendary = "https://api.github.com/repos/derrod/legendary/releases/latest"
gogdl = "https://api.github.com/repos/Heroic-Games-Launcher/heroic-gogdl/releases/latest"
egl_dummy = "https://raw.githubusercontent.com/reakjra/omikuji-assets/main/runtime/epic/EpicGamesLauncher.exe"
```

### `[steam]`

```toml
[steam]
api_key = ""
install_dirs = [""]
```

Optional Steam Web API key ([get one here](https://steamcommunity.com/dev/apikey)). Without it, Steam library listing still works (read locally from ACF files), only remote playtime sync is off.

`install_dirs` points omikuji at Steam installations outside the checked locations (`~/.steam`, `~/.local/share/Steam`, the Flatpak and Snap paths, `/usr/share/steam`), for example `install_dirs = ["/mnt/games/Steam"]`. Entries are read before the built-in locations and in the order given, so they also decide which install is used when more than one exists, and they cover library listing, Proton detection and compatibility tool links. Library folders on other drives are read from `libraryfolders.vdf` and do not need listing here.


## `app.toml`

UI preferences and app behaviour live in `~/.local/share/omikuji/app.toml`: categories, nav rail, tab visibility, zoom, theme, bandwidth limit, threads, etc. Almost all of it is set through the app, and the file is live-watched, so edits apply without a restart.

The only option that isn't in the GUI is this one, makes the fields like the older version. (equals making them ugly)

```toml
[theme]
fill_fields = true
```

## `components.toml`

file that holds all the runners and layers sources. Plus, layers' state (active)

example one: 

```toml
[[runners]]
name = "Proton-Spritz"
kind = "proton"
api_url = "https://api.github.com/repos/NelloKudo/proton-cachyos/releases"
desc = ""
asset_priority = []
require_asset_match = false

[[runners]]
name = "Proton-GE"
kind = "proton"
api_url = "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases"
desc = ""
asset_priority = []
require_asset_match = false

[[runners]]
name = "Dawn Winery Proton"
kind = "proton"
api_url = "https://dawn.wine/api/v1/repos/dawn-winery/dwproton/releases"
desc = ""
asset_priority = []
require_asset_match = false

[[runners]]
name = "Proton-Cachyos"
kind = "proton"
api_url = "https://api.github.com/repos/CachyOS/proton-cachyos/releases"
desc = ""
asset_priority = ["_v3"]
require_asset_match = false

[[runners]]
name = "Wine-Spritz"
kind = "wine"
api_url = "https://api.github.com/repos/NelloKudo/spritz-wine/releases"
desc = ""
asset_priority = []
require_asset_match = false

[[layers]]
name = "DXVK"
kind = "dxvk"
api_url = "https://api.github.com/repos/doitsujin/dxvk/releases"
desc = ""
asset_priority = []
require_asset_match = false

[[layers]]
name = "VKD3D-Proton"
kind = "vkd3d"
api_url = "https://api.github.com/repos/HansKristian-Work/vkd3d-proton/releases"
desc = ""
asset_priority = []
require_asset_match = false

[[layers]]
name = "DXVK-NVAPI"
kind = "dxvk_nvapi"
api_url = "https://api.github.com/repos/jp7677/dxvk-nvapi/releases"
desc = ""
asset_priority = []
require_asset_match = false

[active]
DXVK-NVAPI = "v0.9.2"
DXVK = "dxvk-3.0.2"
VKD3D-Proton = "vkd3d-proton-3.0.1"
```

## `defaults.toml`

file that holds the global defaults for games (`Settings -> Defaults`).

example:

```toml
[wine]
version = "Proton-Cachyos-Latest"
prefix = "${prefixes_path}/PrefixGE"
esync = true
fsync = true
ntsync = true
vkd3d = false
dxvk_nvapi = false
battleye = false

[launch.env]
PROTON_USE_NTSYNC = "1"
ENABLE_VKSUMI = "1"
VKMIRU = "1"
OBS_VKCAPTURE = "1"

[graphics]
gpu = "00000000-0300-0000-0000-000000000000"

[graphics.gamescope]

[system]
gamemode = false
cpu_limit = 0
```
