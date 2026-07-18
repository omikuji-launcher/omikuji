#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

CARGO_APP="crates/omikuji/Cargo.toml"
CARGO_CORE="crates/omikuji-core/Cargo.toml"
PKGBUILD_BIN="packaging/PKGBUILD-bin"
FLATPAK="flatpak/io.github.reakjra.omikuji.yaml"
SPEC="packaging/omikuji.spec"
METAINFO="packaging/io.github.reakjra.omikuji.metainfo.xml"

MAINTAINER="reakjra <reakjra@proton.me>"
COPR_URL="https://copr.fedorainfracloud.org/coprs/reakjra/omikuji/"

if [[ -t 1 ]]; then
    B=$'\e[1m'; D=$'\e[2m'; G=$'\e[32m'; C=$'\e[36m'; Y=$'\e[33m'; X=$'\e[0m'
else
    B=''; D=''; G=''; C=''; Y=''; X=''
fi
hr() { printf '%s──────────────────────────────────────────%s\n' "$D" "$X"; }

printf '\n%s%s  omikuji release bump%s\n' "$B" "$C" "$X"
hr
printf '\n'

read -rp "${B}version${X} to bump to (e.g. 0.4.3): " VER
VER="${VER#v}"
read -rp "${B}date${X} [$(date +%F)]: " DATE
DATE="${DATE:-$(date +%F)}"

printf '%srelease notes, one bullet per line (empty line ends):%s\n' "$B" "$X"
BULLETS=()
while IFS= read -r line; do
    [[ -z "$line" ]] && break
    BULLETS+=("$line")
done

sed -i "s/^version = \".*\"/version = \"$VER\"/" "$CARGO_APP"
sed -i "s/^version = \".*\"/version = \"$VER\"/" "$CARGO_CORE"
sed -i "s/^pkgver=.*/pkgver=$VER/" "$PKGBUILD_BIN"
sed -i "s/^pkgrel=.*/pkgrel=1/" "$PKGBUILD_BIN"
sed -i "s/tag: \"v.*\"/tag: \"v$VER\"/" "$FLATPAK"
sed -i "s/^Version:.*/Version:        $VER/" "$SPEC"

block=$(mktemp)
{
    printf '    <release version="%s" date="%s">\n' "$VER" "$DATE"
    printf '      <description>\n        <ul>\n'
    for b in "${BULLETS[@]}"; do printf '%s\n' "          <li>$b</li>"; done
    printf '        </ul>\n      </description>\n    </release>\n'
} > "$block"
sed -i "/<releases>/r $block" "$METAINFO"
rm -f "$block"

log=$(mktemp)
{
    printf '* %s %s - %s-1\n' "$(LC_ALL=C date -d "$DATE" '+%a %b %d %Y')" "$MAINTAINER" "$VER"
    for b in "${BULLETS[@]}"; do printf '%s\n' "- $b"; done
    printf '\n'
} > "$log"
sed -i "/^%changelog/r $log" "$SPEC"
rm -f "$log"

printf '\n%s%s  bumped > %s  (%s)%s\n' "$B" "$G" "$VER" "$DATE" "$X"
hr
grep -HE '^version = ' "$CARGO_APP" "$CARGO_CORE" || true
grep -HE '^pkgver='    "$PKGBUILD_BIN" || true
grep -HE '^pkgrel='    "$PKGBUILD_BIN" || true
grep -HE 'tag: "v'     "$FLATPAK" || true
grep -HE '^Version:'   "$SPEC" || true
grep -HE "release version=\"$VER\"" "$METAINFO" || true
hr

printf '\n%sGood job, crazy labour! Now:%s\n' "$B" "$X"
printf '  %scargo build%s              %s# syncs Cargo.lock or pain will happen%s\n' "$C" "$X" "$D" "$X"
printf '  %sgit commit -m "bump to v%s"  &&  git tag v%s  &&  git push  &&  git push origin v%s%s\n' "$C" "$VER" "$VER" "$VER" "$X"

printf '\n%s%sAfter the release CI finishes%s\n' "$B" "$Y" "$X"
printf '  %srebuild COPR%s  (New Build > SCM, committish master)\n' "$B" "$X"
printf '    %s%s%s\n\n' "$C" "$COPR_URL" "$X"
printf '  %supdate AUR -bin%s  (needs the published AppImage first)\n' "$B" "$X"
printf '    %scd into omikuji-bin aur folder (wherever the PKGBUILD is), then:%s\n\n' "$D" "$X"
printf '    %scp %s/%s ./PKGBUILD%s\n' "$C" "$ROOT" "$PKGBUILD_BIN" "$X"
printf '    %supdpkgsums && makepkg --printsrcinfo > .SRCINFO%s\n' "$C" "$X"
printf '    %sgit add PKGBUILD .SRCINFO%s\n' "$C" "$X"
printf '    %sgit commit -m "bump to v%s" && git push%s\n' "$C" "$VER" "$X"
printf '\n'
