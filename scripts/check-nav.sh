#!/usr/bin/env bash
# keyboard/controller navigation checks from Nav.js. that thing is brain damage beyond human comprehension.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE="$ROOT/crates/omikuji"
COMPONENTS="$CRATE/qml/components"

failures=0

fail() {
    echo "nav check: $1" >&2
    failures=$((failures + 1))
}

qml_files() {
    find "$CRATE/qml" -name "*.qml" -print0
}

blocks() {
    qml_files | xargs -0 awk -v type="$1" '
        FNR == 1 { open = 0 }
        {
            line = $0
            if (!open && line ~ ("(^|[^A-Za-z0-9_.])" type " \\{")) {
                open = 1
                start = FNR
                depth = 0
                body = ""
            }
            if (open) {
                body = body " " line
                depth += gsub(/\{/, "{", line) - gsub(/\}/, "}", line)
                if (depth <= 0) {
                    print FILENAME ":" start "\t" body
                    open = 0
                }
            }
        }'
}

relative() {
    sed "s|^$CRATE/||"
}

check_raw_clickables() {
    local allowed='qml/components/(controls/(PressArea|IconButton|M3Button|FieldButton|M3Switch|M3Dropdown|M3SpinBox|SegmentedControl)|navigation/(NavTabs|SubNavRail)|cards/(BaseCard|CardGrid|StoreCardAction)|popups/ContextMenu|dialogs/DialogCard|settings/SettingsModal|logs/GameLogsWindow|consolemode/)'
    while IFS= read -r hit; do
        fail "$hit: raw MouseArea with onClicked is unreachable by keyboard, use PressArea or CheckRow"
    done < <(blocks MouseArea | grep "onClicked" | grep -v "onClicked: *forceActiveFocus()" | cut -f1 | relative | grep -Ev "^$allowed" || true)
}

check_spinbox_value_changed() {
    while IFS= read -r hit; do
        fail "$hit: M3SpinBox uses onValueChanged, which rewrites the value on every external refresh, use onMoved"
    done < <(blocks M3SpinBox | grep "onValueChanged" | cut -f1 | relative || true)
}

check_controlled_values() {
    local controls=(M3Switch M3SpinBox M3Slider M3Dropdown)
    for control in "${controls[@]}"; do
        while IFS= read -r hit; do
            fail "$control.qml:$hit: assigns its own bound value, emit the change instead"
        done < <(grep -nE '(^|[^.A-Za-z0-9_])(root\.)?(checked|value|currentIndex)\s*=[^=]' "$COMPONENTS/controls/$control.qml" | cut -d: -f1 || true)
    done
}

check_list_key_navigation() {
    while IFS= read -r hit; do
        fail "$hit: ListView without keyNavigationEnabled: false fights spatial navigation"
    done < <(blocks ListView | grep -v "keyNavigationEnabled: false" | cut -f1 | relative | grep -Ev '^qml/components/(popups/ToastManager|consolemode/)' || true)
}

run_tests() {
    local runner
    runner="$(qmake6 -query QT_INSTALL_BINS)/qmltestrunner"
    QT_QPA_PLATFORM=offscreen "$runner" -input "$CRATE/tests/qml" -import "$CRATE/tests/qml/imports" || failures=$((failures + 1))
}

check_raw_clickables
check_spinbox_value_changed
check_controlled_values
check_list_key_navigation
run_tests

if [ "$failures" -gt 0 ]; then
    echo "nav check: $failures failure(s)" >&2
    exit 1
fi
echo "nav check: ok"
