#!/bin/sh

set -eu

ARCH=$(uname -m)
SHARUN="https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/refs/heads/main/useful-tools/quick-sharun.sh"
DEBLOATED_PKGS="https://raw.githubusercontent.com/pkgforge-dev/Anylinux-AppImages/refs/heads/main/useful-tools/get-debloated-pkgs.sh"

if [ -n "${GITHUB_REF_NAME:-}" ] && [ "${GITHUB_REF_TYPE:-}" = "tag" ]; then
    VERSION="${GITHUB_REF_NAME}"
else
    VERSION=$(pacman -Q omikuji-git | awk '{print $2; exit}')
fi
export ARCH VERSION
export OUTPATH=./dist
export ADD_HOOKS="self-updater.hook"
export UPINFO="gh-releases-zsync|${GITHUB_REPOSITORY%/*}|${GITHUB_REPOSITORY#*/}|latest|*$ARCH.AppImage.zsync"
export ICON=/usr/share/icons/hicolor/512x512/apps/io.github.reakjra.omikuji.png
export DESKTOP=/usr/share/applications/io.github.reakjra.omikuji.desktop

#Remove leftovers
rm -rf AppDir dist

# ADD LIBRARIES
wget --retry-connrefused --tries=30 "$DEBLOATED_PKGS" -O ./get-debloated-pkgs
wget --retry-connrefused --tries=30 "$SHARUN" -O ./quick-sharun
chmod +x ./quick-sharun ./get-debloated-pkgs

# Debloated pkgs
./get-debloated-pkgs -add-common --prefer-nano

# Deploy dependencies
./quick-sharun /usr/bin/omikuji

# Turn AppDir into AppImage
./quick-sharun --make-appimage
