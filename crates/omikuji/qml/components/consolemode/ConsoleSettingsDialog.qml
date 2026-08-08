import QtQuick
import omikuji 1.0
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects
import "../controls"
import "../primitives"

Item {
    id: root

    property string currentBackground: "wave"
    property bool isDropdownHost: true
    property int focusedRow: -1

    readonly property bool dropdownOpen: bgDropdown.popupOpen
    readonly property int rowCount: 1

    signal backgroundSelected(string name)

    visible: false
    z: 2000

    readonly property var _options: [
        { label: qsTr("Wave"),      value: "wave" },
        { label: qsTr("Metaballs"), value: "metaballs" },
        { label: qsTr("Veins"),     value: "veins" },
        { label: qsTr("Aurora"),    value: "aurora" },
        { label: qsTr("Sakura"),    value: "sakura" },
        { label: qsTr("Hero"),      value: "hero" }
    ]

    function open() {
        visible = true
        forceActiveFocus()
    }

    function close() {
        if (bgDropdown.popupOpen) bgDropdown.closePopupCancel()
        focusedRow = -1
        visible = false
    }

    function focusFirst() { focusedRow = 0 }
    function clearFocus() { focusedRow = -1 }

    function handleDpadUp() {
        if (dropdownOpen) bgDropdown.highlightPrev()
        else if (focusedRow > 0) focusedRow -= 1
    }
    function handleDpadDown() {
        if (dropdownOpen) bgDropdown.highlightNext()
        else if (focusedRow < rowCount - 1) focusedRow += 1
    }
    function handleAPress() {
        if (dropdownOpen) {
            bgDropdown.closePopupCommit()
        } else if (focusedRow === 0) {
            bgDropdown.openPopup()
        }
    }
    function handleBPress() {
        if (dropdownOpen) bgDropdown.closePopupCancel()
        else close()
    }

    Keys.onEscapePressed: close()

    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, 0.55)
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.AllButtons
            onClicked: root.close()
            onWheel: (wheel) => wheel.accepted = true
            cursorShape: Qt.ArrowCursor
        }
    }

    Rectangle {
        id: card
        anchors.centerIn: parent
        width: Math.min(parent.width - 120, 520)
        height: contentCol.implicitHeight + 44
        radius: Theme.radius.xl
        color: Theme.surface
        border.width: 1
        border.color: Theme.alpha(Theme.text, 0.08)

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            onClicked: {}
            onWheel: (wheel) => wheel.accepted = true
        }

        layer.enabled: true
        layer.effect: DropShadow {
            radius: 24
            samples: 32
            color: Qt.rgba(0, 0, 0, 0.4)
            horizontalOffset: 0
            verticalOffset: 6
        }

        ColumnLayout {
            id: contentCol
            anchors.fill: parent
            anchors.margins: 22
            spacing: 20

            Text {
                Layout.fillWidth: true
                text: qsTr("Settings")
                color: Theme.text
                font.pixelSize: 18
                font.weight: Font.DemiBold
            }

            Item {
                Layout.fillWidth: true
                implicitHeight: bgRow.implicitHeight + 16

                Rectangle {
                    anchors.fill: parent
                    radius: Theme.radius.md
                    color: root.focusedRow === 0
                        ? Theme.alpha(Theme.accent, 0.08)
                        : "transparent"
                    border.width: root.focusedRow === 0 ? 2 : 0
                    border.color: Theme.accent
                    Behavior on border.width { NumberAnimation { duration: 120 } }
                    Behavior on color { ColorAnimation { duration: 120 } }
                }

                RowLayout {
                    id: bgRow
                    anchors.fill: parent
                    anchors.margins: 8
                    spacing: 14

                    SvgIcon {
                        name: "imagesmode"
                        size: 22
                        color: Theme.icon
                        Layout.alignment: Qt.AlignVCenter
                    }

                    Text {
                        text: qsTr("Background")
                        color: Theme.text
                        font.pixelSize: 14
                        font.weight: Font.Medium
                        Layout.preferredWidth: 110
                        Layout.alignment: Qt.AlignVCenter
                    }

                    M3Dropdown {
                        id: bgDropdown
                        Layout.fillWidth: true
                        options: root._options
                        currentIndex: {
                            for (let i = 0; i < root._options.length; i++) {
                                if (root._options[i].value === root.currentBackground) return i
                            }
                            return 0
                        }
                        onSelected: (value) => root.backgroundSelected(value)
                    }
                }
            }
        }
    }
}
