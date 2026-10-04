import QtQuick
import omikuji 1.0

M3Dropdown {
    property string arch: "win64"

    label: SettingLabels.label("wine.prefix_arch")
    options: [
        { label: qsTr("64-bit (win64)"), value: "win64" },
        { label: qsTr("32-bit (win32)"), value: "win32" }
    ]
    currentIndex: arch === "win32" ? 1 : 0
}
