# Home Manager Module Options


## [`programs.omikuji.enable`](#L20)

Whether to enable omikuji.

**Type:** `boolean`

**Default:** `false`

**Example:** `true`

## [`programs.omikuji.extraPackages`](#L23)


List of packages to pass as extraPkgs to lutris.
Please note runners are not detected properly this way, use a proper option for those.


**Type:** `with types; listOf package`

**Default:** `[ ]`

**Example:** `"with pkgs; [mangohud winetricks gamescope gamemode umu-launcher]"`

## [`programs.omikuji.steamPackage`](#L33)


This must be the same you use for your system, or two instances will conflict,
for example, if you configure steam through the nixos module, a good value is "osConfig.programs.steam.package"


**Type:** `with types; nullOr package`

**Default:** `null`

**Example:** `"pkgs.steam or osConfig.programs.steam.package"`

## [`programs.omikuji.winePackages`](#L43)


List of wine packages to be added for omikuji to use.


**Type:** `with types; listOf package`

**Default:** `[ ]`

**Example:** `"[ pkgs.wineWow64Packages.full ]"`

## [`programs.omikuji.protonPackages`](#L52)


List of proton packages to be added for omikuji to use with umu-launcher.


**Type:** `with types; listOf package`

**Default:** `[ ]`

**Example:** `"[ pkgs.proton-ge-bin ]"`

## [`programs.omikuji.defaultWinePackage`](#L61)


Default wine/proton package used in the settings.


**Type:** `with types; nullOr package`

**Default:** `null`

**Example:** `"pkgs.proton-ge-bin"`

## [`programs.omikuji.settings.mutableDefaults`](#L72)


Wether configuration in `defaults.toml` can be updated by omikuji.


**Type:** `types.bool`

**Default:** `true`

## [`programs.omikuji.settings.defaults`](#L80)


Configuration written to
`$XDG_DATA_HOME/omikuji/defaults.toml`.


**Type:** `any`

**Default:** `{ }`

**Example:**

```nix
wine = {
  ntsync = true
  dxvk = true
  vkd3d = true
  d3d_extras = true
};

launch.env = {
  PROTON_USE_WAYLAND = "1";
};

graphics.mangohud = true;
system.gamemode = true;
```

## [`programs.omikuji.settings.mutableSettings`](#L104)


Wether configuration in `settings.toml` can be updated by omikuji.


**Type:** `types.bool`

**Default:** `true`

## [`programs.omikuji.settings.settings`](#L112)


Configuration written to
`$XDG_DATA_HOME/omikuji/settings.toml`.


**Type:** `any`

**Default:** `{ }`

**Example:**

```nix
runners = [
  {
    name = "Proton-GE";
    kind = "proton";
    api_url = "https://api.github.com/repos/GloriousEggroll/proton-ge-custom/releases";
    asset_pattern = ".tar.gz";
    extract = "tar_gz";
  }
  {
    name = "Proton-Cachyos";
    kind = "proton";
    api_url = "https://api.github.com/repos/CachyOS/proton-cachyos/releases";
    asset_pattern = ".tar.xz";
    extract = "tar_xz";
  }
];

dll_packs = [
  {
    name = "DXVK";
    kind = "dxvk";
    api_url = "https://api.github.com/repos/doitsujin/dxvk/releases";
    asset_pattern = ".tar.gz";
    extract = "tar_gz";
  }
];
```

## [`programs.omikuji.settings.mutableUi`](#L149)


Wether configuration in `ui.toml` can be updated by omikuji.


**Type:** `types.bool`

**Default:** `true`

## [`programs.omikuji.settings.ui`](#L157)


Configuration written to
`$XDG_DATA_HOME/omikuji/ui.toml`.


**Type:** `any`

**Default:** `{ }`

**Example:**

```nix
theme = {
  follow_system_colors = false;
  colors = {
    bg = "#181825";
    surface = "#1e1e2e";
    accent = "#cba6f7";
    accentText = "#11111b";
    text = "#cdd6f4";
    error = "#f38ba8";
    success = "#a6e3a1";
    warning = "#f9e2af";
  };
};

console_mode = {
  background = "wave";
  active = false;
};
```

---
*Generated with [nix-doc](https://github.com/Thunderbottom/nix-doc)*
