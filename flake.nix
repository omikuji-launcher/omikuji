{
  description = "Omikuji flake";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

      perSystem = { config, self', inputs', pkgs, system, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            (import inputs.rust-overlay)
            (final: prev: {
              omikuji = final.callPackage ./packaging/nix/package { };
              omikuji-unwrapped = final.callPackage ./packaging/nix/package/unwrapped.nix { flakeRoot = self; };
            })
          ];
          config = { };
        };

        devShells.default = import ./packaging/nix/shell.nix { inherit pkgs; };

        packages = {
          omikuji = pkgs.omikuji;
          omikuji-unwrapped = pkgs.omikuji-unwrapped;
          default = self'.packages.omikuji;
        };
      };

      flake.homeModules = {
        omikuji = import ./packaging/nix/module.nix self;
        default = self.homeModules.omikuji;
      };
    };
}
