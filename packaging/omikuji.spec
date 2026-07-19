%global debug_package %{nil}

Name:           omikuji
Version:        0.8.1
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
* Sun Jul 19 2026 reakjra <reakjra@proton.me> - 0.8.1-1
- Fix GOG and Epic Games imports
- Refractor runners and components fetcher
- Refractor proton/wine kinds detection

* Fri Jul 17 2026 reakjra <reakjra@proton.me> - 0.8.0-1
- Added community scripts
- Fixed super blurry zoom
- Env-marker to properly track multiple running games
- Wine command prompt along the other winetools
- Run wine commands dialog along other winetools
- Custom template literals for input fields
- Icon-only collapsable navbar
- 3-mode (Normal, Fit, Frameless) game cards style
- Resizible modals and dialogs
- Maximizable modals/dialogs size on near-full window size
- Customizable font sizes with its modal
- Move envs/dlls sets to "Presets" tab in settings
- Epic and GOG games installations dialogues expandable details (plot and system requirements)

* Mon Jul 13 2026 reakjra <reakjra@proton.me> - 0.7.0-1
- Kuro games now use krpdiff to update.
- Fix winetricks not opening by not inheriting blank DISPLAY env (#51)
- Fix modals blur on fractional coords + dropdown scroll wheel leakthrough
- downloads page items overhaul + shm leak on cancel (GOGDL/Legendary)
- Move value on top of the sliders
- Move to the left the theme colors value
- Refine spinboxes 
- add filled svgs with filled toggle and rework ofudas, steam and gog svgs
- Refine toasts
- Add hide functionality for main library cards
- Port settingd and downloads bridges onto kushi codegen
- Add kushi crate dep

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
