# Shell made by https://github.com/Thra11, and sligthly modified

{ pkgs ? import <nixpkgs> { } }:
let
  rustVersion = "latest";
  # rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
  #   extensions = [
  #     "rust-src" # for rust-analyzer
  #   ];
  # };
  qtEnv = with pkgs.qt6; env "qt-custom-${qtbase.version}" [
    qtbase
    qtdeclarative
    qtsvg
    qtshadertools
    qt5compat
  ];
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    qtEnv
    makeWrapper
    pkg-config
    cmake
    protobuf
  ] ++ (with rust-bin.stable.${rustVersion}.default; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
  ]);

  buildInputs = with pkgs; [
    rust-bin.stable.${rustVersion}.default
    # rust-analyzer
    udev
    libglvnd
    pkg-config
    qtEnv
    openssl
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  shellHook = ''
    # Add Qt-related environment variables.
    # https://discourse.nixos.org/t/python-qt-woes/11808/10
    setQtEnvironment=$(mktemp)
    random=$(openssl rand -base64 20 | sed "s/[^a-zA-Z0-9]//g")
    makeWrapper "$(type -p sh)" "$setQtEnvironment" "''${qtWrapperArgs[@]}" --argv0 "$random"
    sed "/$random/d" -i "$setQtEnvironment"
    source "$setQtEnvironment"
    export QMAKE="${qtEnv}/bin/qmake"
  '';
}
