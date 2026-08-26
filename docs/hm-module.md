# NixOS Module Options


<a id="options-programs-omikuji-enable"></a>
## [`options.programs.omikuji.enable`](module.nix#L36)

Whether to enable omikuji.

**Type:** `boolean`

**Default:** `false`

**Example:** `true`

<a id="options-programs-omikuji-package"></a>
## [`options.programs.omikuji.package`](module.nix#L37)

The omikuji package to use.

**Type:** `package`

**Default:** `pkgs.omikuji`

<a id="options-programs-omikuji-extraPackages"></a>
## [`options.programs.omikuji.extraPackages`](module.nix#L39)


List of packages to pass as extraPkgs to lutris.
Please note runners are not detected properly this way, use a proper option for those.


**Type:** `list of package`

**Default:** `[ ]`

**Example:** `"with pkgs; [mangohud winetricks gamescope gamemode umu-launcher]"`

<a id="options-programs-omikuji-steamPackage"></a>
## [`options.programs.omikuji.steamPackage`](module.nix#L49)


This must be the same you use for your system, or two instances will conflict,
for example, if you configure steam through the nixos module, a good value is "osConfig.programs.steam.package"


**Type:** `null or package`

**Default:** `null`

**Example:** `"pkgs.steam or osConfig.programs.steam.package"`

<a id="options-programs-omikuji-winePackages"></a>
## [`options.programs.omikuji.winePackages`](module.nix#L59)


List of wine packages to be added for omikuji to use.


**Type:** `list of package`

**Default:** `[ ]`

**Example:** `"[ pkgs.wineWow64Packages.full ]"`

<a id="options-programs-omikuji-protonPackages"></a>
## [`options.programs.omikuji.protonPackages`](module.nix#L68)


List of proton packages to be added for omikuji to use with umu-launcher.


**Type:** `list of package`

**Default:** `[ ]`

**Example:** `"[ pkgs.proton-ge-bin ]"`

<a id="options-programs-omikuji-defaultWinePackage"></a>
## [`options.programs.omikuji.defaultWinePackage`](module.nix#L77)


Default wine/proton package used in the settings.


**Type:** `null or package`

**Default:** `null`

**Example:** `"pkgs.proton-ge-bin"`

<a id="options-programs-omikuji-settings-mutableDefaults"></a>
## [`options.programs.omikuji.settings.mutableDefaults`](module.nix#L88)


Wether configuration in `defaults.toml` can be updated by omikuji.


**Type:** `boolean`

**Default:** `true`

<a id="options-programs-omikuji-settings-defaults"></a>
## [`options.programs.omikuji.settings.defaults`](module.nix#L96)


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

<a id="options-programs-omikuji-settings-mutableSettings"></a>
## [`options.programs.omikuji.settings.mutableSettings`](module.nix#L120)


Wether configuration in `settings.toml` can be updated by omikuji.


**Type:** `boolean`

**Default:** `true`

<a id="options-programs-omikuji-settings-settings"></a>
## [`options.programs.omikuji.settings.settings`](module.nix#L128)


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

<a id="options-programs-omikuji-settings-mutableApps"></a>
## [`options.programs.omikuji.settings.mutableApps`](module.nix#L165)


Wether configuration in `app.toml` can be updated by omikuji.


**Type:** `boolean`

**Default:** `true`

<a id="options-programs-omikuji-settings-apps"></a>
## [`options.programs.omikuji.settings.apps`](module.nix#L173)


Configuration written to
`$XDG_DATA_HOME/omikuji/app.toml`.


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

<a id="options-programs-omikuji-settings-ui"></a>
## [`options.programs.omikuji.settings.ui`](module.nix#L25)

> [!WARNING]
> This option was renamed. Use [`programs.omikuji.settings.apps`](#options-programs-omikuji-settings-apps) instead.

**Type:** `renamed option`

<a id="options-programs-omikuji-settings-mutableUi"></a>
## [`options.programs.omikuji.settings.mutableUi`](module.nix#L29)

> [!WARNING]
> This option was renamed. Use [`programs.omikuji.settings.mutableApps`](#options-programs-omikuji-settings-mutableApps) instead.

**Type:** `renamed option`

---
*Generated with [nix-options-doc](https://github.com/Thunderbottom/nix-options-doc)*
