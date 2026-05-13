# Building

Disclaimer: This was written with arch in mind. So the package names might be different or some might not even exist on your distro.

1. Install the dependencies;
```
sudo pacman -S pkg-config \
cmake \
systemd-libs \
fuse2 \
wget \
protobuf \
rust \
cmake \
libxkbcommon \
xcb-util-cursor \
libcups \
glib2 \
libproxy \
qt6-base \
qt6-tools \
qt6-wayland \
qt6-shadertools \
qt6-declarative \
qt6-5compat \
qt6-svg \
libfbclient \
mariadb \
unixodbc \
postgresql-libs \
jxrlib

// On some distros
libqt6waylandclient6
qml6-module-qtwayland-compositor
libqt6core5compat6
libqt6core5compat6-dev
qt6-base-private-dev
```

2. Add `/usr/lib/qt6/bin` to your `$PATH` if needed

```sh
export PATH="/usr/lib/qt6/bin:$PATH"
```

3. Compile the app
```sh
cargo build --release -p omikuji
```

4.Run `./packaging/make-appimage.sh` to make an appimage.
