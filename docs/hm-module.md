# Omikuji Home Manager Module Options


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

## [`programs.omikuji.settings.defaults`](#L71)


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

## [`programs.omikuji.settings.settings`](#L95)


Configuration written to
`$XDG_DATA_HOME/omikuji/settings.toml`.


**Type:** `any`

**Default:** `{ }`

## [`programs.omikuji.settings.ui`](#L104)


Configuration written to
`$XDG_DATA_HOME/omikuji/ui.toml`.


**Type:** `any`

**Default:** `{ }`

---
*Generated with [nix-doc](https://github.com/Thunderbottom/nix-doc)*
