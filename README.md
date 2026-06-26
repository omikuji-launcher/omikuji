# <img src="crates/omikuji/qml/icons/app.png" width="40" align="left"/>   omikuji

A Qt/QML based games/apps launcher for Linux. Built 'cause I couldn't bear having 3 different launchers for just games.

Manages wine/proton runners, wineprefixes, DXVK/VKD3D, and game launching. Imports from Steam, installs Epic, GOG and Waifu machine slots games directly. 

## Read the docs! 

> [!IMPORTANT]
> You feel lost, want more infos or just waste time? Read the [Docs](https://reakjra.github.io/omikuji).

- [Configuration](https://reakjra.github.io/omikuji/user/configuration.html): `settings.toml`, custom runners, DLL packs
- [Nix Home Manager options](docs/hm-module.md): Every options available in the Home Manager module

## Screenshots

| Library                                          | Edit Game                                                 |
|:---|:---|
| <img src="docs/screenshots/main_library.png"/>   | <img src="docs/screenshots/edit_page.png"/>               |
| Epic Games Store                                 | Epic Install Dialog                                       |
| <img src="docs/screenshots/epic_games_store.png"/> | <img src="docs/screenshots/epic_games_store_download.png"/> |
| Gacha Store                                      | Gacha Install Dialog                                      |
| <img src="docs/screenshots/gacha_store.png"/>    | <img src="docs/screenshots/gacha_store_download.png"/>    |
| Interface Settings                               | Components Settings                                       |
| <img src="docs/screenshots/settings_page_interface.png"/> | <img src="docs/screenshots/settings_page_components.png"/> |
| Console Mode (Aurora background)                 | Console Mode (Sakura background)   
| <img src="docs/screenshots/console_mode_1.png"/> | <img src="docs/screenshots/console_mode_2.png"/> |



## Installation / Building

#### Arch (malware repository aka AUR)

```sh
yay -S omikuji-git
# or 
yay -S omikuji-bin
```

#### Fedora COPR (43 - 44)

```sh
sudo dnf copr enable reakjra/omikuji
sudo dnf install omikuji
```
> or manually install with the `.rpm` in the [releases page](https://github.com/reakjra/omikuji/releases).

#### Flatpak 

Until I bother with reading the flathub documentation to submit, you can manually install it yourself:

Grab the `.flatpak` file in the [releases page](https://github.com/reakjra/omikuji/releases)

Install the application by running: 

```sh
flatpak install omikuji.flatpak
```

#### Manual building

Requires Rust (2024 edition), Qt 6.7+, plus `pkgconf` and `cmake`.

```sh
git clone https://github.com/reakjra/omikuji.git
cd omikuji
cargo build --release
```
> (let me know if after 8 hours rust finished compiling 👍👍👍)

Run it straight from the build dir:
```sh
./target/release/omikuji
```


#### Nix:

<details>
<summary><b>Click to expand Nix related stuff</b></summary>

> For any issues related to the flake, mention @claymorwan in your issue.
</br>

If you're on NixOS and using flakes, add the flake to your inputs:
```nix
# flake.nix
{
	
	inputs = {
		nixpkgs.url = "nixpkgs/nixos-unstable";
		
		omikuji = {
			url = "github:reakjra/omikuji";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};
}
```

Then install the app:
```nix
{ inputs, pkgs, ... }:

{
  # Using the home-manager module (recommended)
  programs.omikuji.enable = true;

	# Or NixOS side installation
	environment.systemPackages = [
		inputs.omikuji.packages.${pkgs.stdenv.hostPlatform.system}.default
	];

	# Or home-manager side installation
	home.packages = [
		inputs.omikuji.packages.${pkgs.stdenv.hostPlatform.system}.default
	];
}
```

If you don't want to compile the full package, there's a cachix binary cache from where you can pull the precompiled package:
```nix
{
  nix.settings = {
    substituters = [
      "https://omikuji.cachix.org"
    ];
      
    trusted-substituters = [
      "https://omikuji.cachix.org"
    ];

    trusted-public-keys = [
      "omikuji.cachix.org-1:dS6sbpMxarHWIIk3y0R7KXz3eVHUg1lo/y3gMbv4JhM="
    ];

  };
}
```
And restart the nix daemon to apply them, then you can install the package
> More info about substituter [here](https://wiki.nixos.org/wiki/Binary_Cache#Using_a_binary_cache)

To run it without installing:
```sh
nix run github:reakjra/omikuji
# Add #omikuji-unwrapped to run the unwrapped package
```

Building the package itself:
```sh
nix build github:reakjra/omikuji
```

If you want to straight up build the app itself (during development for example), the flake also comes with a dev shell:
```sh
git clone https://github.com/reakjra/omikuji
cd omikuji
nix develop
# Then just run the usual commands like cargo build or cargo run
```

> In almost any of these cases (apart from `nix run`) you can replace `.default` with `.omikuji-unwrapped` to refer to the unwrapped package.
  Note that the unwrapped package isn't meant to be installed directly.

</details>

## What does it do?!

- **Game library** one TOML per game, shareable, git-friendly.
- **Wine / Proton**: auto-detects Steam-installed Proton, has its own fetcher in the settings.
- **Translation layers**: DXVK, VKD3D, DXVK-NVAPI. Auto-fetched from upstream releases.
- **Stores**: import from Steam (locally), install Epic games (via legendary), GOG (via gogdl), HoYoverse / Kuro / Gryphline gachas (direct downloads and updates).
- **Wine tools**: winecfg, winetricks, regedit, cmd, winefile, run-exe, kill-wineserver.
- **Art fetch**: SteamGridDB for banners, covers, icons.

#### CLI commands

| Command   | args |  Description       										   					 |
| ------ | ---------- | ---------- 												   	 				 |
| `omikuji`         |   `path/to/.exe`     |  Opens a modal for ephemeral runs.			         	 |
| `omikuji run`     |     `slug_or_id`     |  runs a game from the library headless.			     | 
| `omikuji console` |     `None`  	       |  runs Omikuji in console mode.			     			 | 


## Status/Infos

Runtime tools are lazy fetched dont really worry about it. If it's something particular is missing check `settings > components` page at the bottom.

Data lives in `~/.local/share/omikuji/`.

Usable. Daily-driven by me. Its pretty pls tell me its pretty


QML side held up with tape and prays🙏 


Not implemented/WIP/Planning to add: 
- i18n/qsTr (ehahahshhaha) < why this exists
- Amazon Games (Nile)


## Contributing

Bug reports (especially these), requests and PRs welcome. A few notes:

- To get debug logs, in your terminal: RUST_LOG=debug omikuji 
- Open an issue before a big change so we can talk about it first.
- Match the existing code style. (literally just make it better than mine)
- Keep PRs focused. One thing at a time.
- Be thorough in explaining a issue/request/PR, im dummy
- Whatever other 20 reasons people usually list in their contributing section

- assets repo: [omikuji-assets](https://github.com/reakjra/omikuji-assets)

> See also: [Dev Infos](https://reakjra.github.io/omikuji/dev/overview.html)

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).

## Credits

Heavy debt to the prior art:

- [cxx-qt](https://github.com/KDAB/cxx-qt): lovely super-glue.
- [Lutris](https://github.com/lutris/lutris): the wrapping chain, env setup, runner detection logic, and a lot of wine know-how is ported from here.
- [Heroic Games Launcher](https://github.com/Heroic-Games-Launcher/HeroicGamesLauncher): Epic and GOG integration patterns. Also the source of the bundled `gogdl` binary.
- [AAG](https://github.com/an-anime-team/): gacha launcher reference. HoYo Sophon, CDN methods all from them <3. 

Bundled icon set: [Material Symbols](https://github.com/google/material-design-icons) (Apache-2.0).
