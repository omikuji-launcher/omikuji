{
  lib,
  flakeRoot ? ../../../.,
  rustPlatform,
  qt6,
  pkg-config,
  cmake,
  protobuf,
  makeWrapper,
  openssl,
  imagemagick,
}:

let
  cargoToml = fromTOML (builtins.readFile "${flakeRoot}/crates/omikuji/Cargo.toml");
  qtDeps = with qt6; [
    qtbase
    qtdeclarative
    qtsvg
    qtshadertools
    qt5compat
  ];

  qtEnv = with qt6; env "qt-custom-${qtbase.version}" qtDeps;
in 
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "omikuji-unwrapped";
  version = cargoToml.package.version;
  src = flakeRoot;
  cargoLock.lockFile = "${flakeRoot}/Cargo.lock";

  nativeBuildInputs = [
    qtEnv
    pkg-config
    cmake
    qt6.qmake
    qt6.wrapQtAppsHook
    protobuf
    makeWrapper
  ];

  buildInputs = [
    qtEnv
    openssl
  ]
  ++ qtDeps;

  doCheck = false;

  # Needed for omikuji to be able to run appimages
  qtWrapperArgs = [
    "--prefix APPIMAGE_EXTRACT_AND_RUN : 1"
  ];  
  
  prePatch = ''
    substituteInPlace ./crates/omikuji/build.rs \
      --replace-fail '"/usr/lib/qt6/bin/qsb"' '"${qtEnv}/bin/qsb"'
  '';

  preBuild = ''
    # Add Qt-related environment variables.
    # https://discourse.nixos.org/t/python-qt-woes/11808/10
    setQtEnvironment=$(mktemp)
    random=$(openssl rand -base64 20 | sed "s/[^a-zA-Z0-9]//g")
    makeWrapper "$(type -p sh)" "$setQtEnvironment" "''${qtWrapperArgs[@]}" --argv0 "$random"
    sed "/$random/d" -i "$setQtEnvironment"
    source "$setQtEnvironment"
    export QMAKE="${qtEnv}/bin/qmake"
  '';

  postInstall = ''
    install -Dm444 $src/packaging/io.github.reakjra.omikuji.desktop.in \
      $out/share/applications/io.github.reakjra.omikuji.desktop


    install -Dm444 \
      $src/crates/omikuji/qml/icons/app.png \
      $out/share/icons/hicolor/512x512/apps/io.github.reakjra.omikuji.png

    for size in 16 24 32 48 64 128 256; do
      mkdir -p $out/share/icons/hicolor/"$size"x"$size"/apps
      ${lib.getExe imagemagick} \
        $src/crates/omikuji/qml/icons/app.png \
        -resize "$size"x"$size" \
        $out/share/icons/hicolor/"$size"x"$size"/apps/io.github.reakjra.omikuji.png
    done
  '';

  meta = {
    description = "Qt/QML based game launcher for Linux";
    homepage = "https://github.com/reakjra/omikuji";
    license = lib.licenses.gpl3Only;
    maintainers = with lib.maintainers; [ claymorwan ];
    platforms = lib.platforms.linux;
    mainProgram = "omikuji";
  };
})
