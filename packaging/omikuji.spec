%global debug_package %{nil}

Name:           omikuji
Version:        0.6.0
Release:        1%{?dist}
Summary:        Qt/QML based wine apps launcher for Linux

License:        GPL-3.0-or-later
URL:            https://github.com/reakjra/omikuji
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  cmake
BuildRequires:  gcc-c++
BuildRequires:  mold
BuildRequires:  pkgconf-pkg-config
BuildRequires:  protobuf-compiler
BuildRequires:  systemd-devel
BuildRequires:  openssl-devel
BuildRequires:  qt6-qtbase-devel
BuildRequires:  qt6-qtdeclarative-devel
BuildRequires:  qt6-qtsvg-devel
BuildRequires:  qt6-qtshadertools

Requires:       qt6-qtbase
Requires:       qt6-qtdeclarative
Requires:       qt6-qtsvg
Requires:       qt6-qt5compat
Requires:       qt6-qtwayland
Recommends:     vulkan-loader

%description
A Qt/QML based games and apps launcher for Linux with wine/flatpak/native runners, Epic Games (Legendary), GOG (gogdl) and Gacha stores.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --locked

%install
install -Dm0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dm0644 crates/omikuji/qml/icons/app.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/io.github.reakjra.omikuji.png
install -Dm0644 packaging/io.github.reakjra.omikuji.desktop.in %{buildroot}%{_datadir}/applications/io.github.reakjra.omikuji.desktop
install -Dm0644 packaging/io.github.reakjra.omikuji.metainfo.xml %{buildroot}%{_datadir}/metainfo/io.github.reakjra.omikuji.metainfo.xml

%files
%license LICENSE
%{_bindir}/%{name}
%{_datadir}/applications/io.github.reakjra.omikuji.desktop
%{_datadir}/icons/hicolor/512x512/apps/io.github.reakjra.omikuji.png
%{_datadir}/metainfo/io.github.reakjra.omikuji.metainfo.xml

%changelog
* Thu Jul 09 2026 reakjra <reakjra@proton.me> - 0.6.0-1
- Data structure migration + migration modal 
- UI polish (smaller buttons, headers action buttons, log windows refinement, logs custom colors regex)
- nvapi injection following source kind

* Wed Jul 08 2026 reakjra <reakjra@proton.me> - 0.5.3-1
- Main library cards drag and drop with custom order.
- Pre-launch scripts now executing before resolving the executable.

* Fri Jul 03 2026 reakjra <reakjra@proton.me> - 0.5.2-1
- Main library cards A-Z/Z-A sorting
- M3Dropdown widget polish

* Thu Jul 02 2026 reakjra <reakjra@proton.me> - 0.5.1-1
- Env sets for flatpak/native/steam games.
- GOG uninstall button
- Various minor fixes

* Sat Jun 27 2026 reakjra <reakjra@proton.me> - 0.5.0-1
- Ofuda system (prefix manager)
- Prefix prep modal
- Shared scroll between fixed in modals

* Thu Jun 25 2026 reakjra <reakjra@proton.me> - 0.4.4-1
- Lazy runtime tools fetch (Only umu will be fetched on boot)
- Added env/dll sets.
- VKD3D toggle now overrides d3d12core too.

* gio giu 25 2026 reakjra <reakjra@proton.me> - 0.4.3-1
- Fix GOG and Epic Games login page (+ DRY)

* Tue Jun 23 2026 reakjra <reakjra@proton.me> - 0.4.2-1
- "Open with Omikuji" picker for .exe files
- Theme fallbacks when the system color scheme is unknown
- RPM packaging
