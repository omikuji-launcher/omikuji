# Installation

This is how you install omikuji. Pick whatever fits your setup, cutie pie~

## Arch (AUR)

```sh
yay -S omikuji-bin   # prebuilt
yay -S omikuji-git   # builds from latest source
```

## Fedora (COPR)

```sh
sudo dnf copr enable reakjra/omikuji
sudo dnf install omikuji
```

Fedora 43 and 44. Or grab the `.rpm` from the [releases page](https://github.com/omikuji-launcher/omikuji/releases).

## Flatpak

Not on Flathub yet. There is a signed repository instead:

```sh
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak remote-add --if-not-exists omikuji https://omikuji-launcher.github.io/omikuji/flatpak/index.flatpakrepo
flatpak install omikuji io.github.reakjra.omikuji
```

Flathub is required as well: omikuji builds on the KDE and Wine runtimes, which are hosted there.

A single `.flatpak` bundle is also attached to every [release](https://github.com/omikuji-launcher/omikuji/releases).

```sh
flatpak install omikuji.flatpak
```

## Nix

> For flake issues, mention @claymorwan when opening a issue.

On NixOS with flakes, add the input:

```nix
# flake.nix
inputs.omikuji = {
    url = "github:omikuji-launcher/omikuji";
    inputs.nixpkgs.follows = "nixpkgs";
};
```

Then install via the Home Manager module (recommended) or as a package:

```nix
# home-manager module
programs.omikuji.enable = true;

# or as a package (NixOS or home-manager)
environment.systemPackages = [
    inputs.omikuji.packages.${pkgs.stdenv.hostPlatform.system}.default
];
```

To skip compiling, add the Cachix cache:

```nix
nix.settings = {
    substituters = [ "https://omikuji.cachix.org" ];
    trusted-substituters = [ "https://omikuji.cachix.org" ];
    trusted-public-keys = [ "omikuji.cachix.org-1:dS6sbpMxarHWIIk3y0R7KXz3eVHUg1lo/y3gMbv4JhM=" ];
};
```

Or run it without installing:

```sh
nix run github:omikuji-launcher/omikuji
```

## From source

Needs Rust (2024 edition), Qt 6.7+, `pkgconf`, and `cmake`.

```sh
git clone https://github.com/omikuji-launcher/omikuji.git
cd omikuji
cargo build --release
./target/release/omikuji
```
