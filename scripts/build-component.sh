#!/usr/bin/env bash
set -euo pipefail

DEFAULT_COMPONENTS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/omikuji/components/layers"
BUILD_ROOT="${TMPDIR:-/tmp}/omikuji-component-build"

DXVK_REPO="https://github.com/doitsujin/dxvk.git"
VKD3D_REPO="https://github.com/HansKristian-Work/vkd3d-proton.git"

BUILD_DEPS=(
    "git=git"
    "meson=meson"
    "ninja=ninja"
    "gcc=gcc"
    "glslangValidator|glslang=glslang"
    "x86_64-w64-mingw32-g++=mingw-w64-gcc"
    "i686-w64-mingw32-g++=mingw-w64-gcc"
)

if [[ -t 1 ]]; then
    B=$'\e[1m'; D=$'\e[2m'; G=$'\e[32m'; C=$'\e[36m'; Y=$'\e[33m'; R=$'\e[31m'; X=$'\e[0m'
else
    B=''; D=''; G=''; C=''; Y=''; R=''; X=''
fi
hr() { printf '%s──────────────────────────────────────────%s\n' "$D" "$X"; }

PERSIST=n
cleanup() {
    [[ "$PERSIST" == y ]] && return 0
    [[ -n "${BUILD_ROOT:-}" ]] && rm -rf "$BUILD_ROOT"
}
trap cleanup EXIT

printf '\n%s%s  omikuji component builder%s\n' "$B" "$C" "$X"
hr
printf '%sbuilds dxvk / vkd3d-proton from master and installs into omikuji%s\n\n' "$D" "$X"

printf '%schecking build deps%s\n' "$B" "$X"
MISSING=()
for dep in "${BUILD_DEPS[@]}"; do
    cmds="${dep%%=*}"; pkg="${dep##*=}"
    found=""
    IFS='|' read -ra alts <<< "$cmds"
    for c in "${alts[@]}"; do
        if command -v "$c" >/dev/null 2>&1; then found="$c"; break; fi
    done
    if [[ -n "$found" ]]; then
        printf '  %s[ok]%s %s\n' "$G" "$X" "$found"
    else
        printf '  %s[no]%s %s %s(%s)%s\n' "$R" "$X" "${alts[0]}" "$D" "$pkg" "$X"
        MISSING+=("$pkg")
    fi
done

if (( ${#MISSING[@]} )); then
    mapfile -t MISSING < <(printf '%s\n' "${MISSING[@]}" | sort -u)
    printf '\n%smissing deps, install with:%s\n' "$R" "$X"
    printf '  %ssudo pacman -S --needed %s%s\n\n' "$C" "${MISSING[*]}" "$X"
    exit 1
fi
printf '\n'

printf '%swhich component?%s\n' "$B" "$X"
printf '  %s1%s  DXVK          %s(doitsujin/dxvk)%s\n' "$C" "$X" "$D" "$X"
printf '  %s2%s  VKD3D-Proton  %s(HansKristian-Work/vkd3d-proton)%s\n' "$C" "$X" "$D" "$X"
read -rp "${B}choice${X} [1]: " CHOICE
case "${CHOICE:-1}" in
    1) KEY=dxvk;  NAME="DXVK";         REPO="$DXVK_REPO"  ;;
    2) KEY=vkd3d; NAME="VKD3D-Proton"; REPO="$VKD3D_REPO" ;;
    *) printf '%sunknown choice: %s%s\n' "$R" "$CHOICE" "$X"; exit 1 ;;
esac

read -rp "${B}components dir${X} [$DEFAULT_COMPONENTS_DIR]: " COMPONENTS_DIR
COMPONENTS_DIR="${COMPONENTS_DIR:-$DEFAULT_COMPONENTS_DIR}"
COMPONENTS_DIR="${COMPONENTS_DIR/#\~/$HOME}"

read -rp "${B}keep build files after?${X} (y/N): " ANS
[[ "${ANS,,}" == y ]] && PERSIST=y

SRC="$BUILD_ROOT/src/$KEY"
STAGING="$BUILD_ROOT/out/$KEY"
mkdir -p "$BUILD_ROOT"

printf '\n%sfetching %s%s\n' "$B" "$NAME" "$X"
hr
if [[ -d "$SRC/.git" ]]; then
    git -C "$SRC" fetch --tags --prune origin
    git -C "$SRC" reset --hard '@{u}'
    git -C "$SRC" submodule update --init --recursive
else
    rm -rf "$SRC"
    git clone --recursive "$REPO" "$SRC"
fi
hr

TAG="$(git -C "$SRC" describe --tags --always)"
TAG="${TAG//\//-}"

printf '\n%sbuilding%s %s %s%s%s\n' "$B" "$X" "$NAME" "$C" "$TAG" "$X"
hr
rm -rf "$STAGING"; mkdir -p "$STAGING"
if ! ( cd "$SRC" && ./package-release.sh "$TAG" "$STAGING" --no-package ); then
    hr
    printf '%sbuild failed, scroll up for the compiler error.%s\n' "$R" "$X"
    exit 1
fi
hr

BUILT="$(find "$STAGING" -mindepth 1 -maxdepth 1 -type d | head -n1)"
if [[ -z "$BUILT" || ! -d "$BUILT/x64" ]]; then
    printf '%sbuild produced no x64 output in %s%s\n' "$R" "$STAGING" "$X"
    exit 1
fi

DEST="$COMPONENTS_DIR/$NAME/$TAG"
rm -rf "$DEST"; mkdir -p "$DEST"
cp -r "$BUILT/x64" "$DEST/"
for d in x32 x86; do
    [[ -d "$BUILT/$d" ]] && cp -r "$BUILT/$d" "$DEST/"
done
printf '{"source":"%s","tag":"%s"}\n' "$NAME" "$TAG" > "$DEST/.omikuji.json"

printf '\n%s%s  done > %s %s installed%s\n' "$B" "$G" "$NAME" "$TAG" "$X"
hr
printf '  %s%s%s\n' "$C" "$DEST" "$X"
for d in "$DEST"/*/; do printf '    %s%s%s\n' "$D" "$(basename "$d")/" "$X"; done
hr
printf '\n%spick %s%s%s under %s in omikuji > settings > components.%s\n' "$B" "$C" "$TAG" "$X" "$NAME" "$X"
[[ "$PERSIST" == y ]] && printf '%sbuild files kept in %s%s\n' "$D" "$BUILD_ROOT" "$X"
printf '\n'
