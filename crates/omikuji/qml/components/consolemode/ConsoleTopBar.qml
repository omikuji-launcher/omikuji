pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import Qt5Compat.GraphicalEffects
import "../primitives"

Row {
    id: bar

    property real uiScale: 1.0
    readonly property real _scale: Math.max(0.85, Math.min(uiScale, 1.5))

    property bool searchExpanded: false
    readonly property string searchText: searchField.text
    property int focusIndex: -1

    spacing: 20 * _scale

    signal searchSubmitted()
    signal searchClosed()
    signal appIconClicked()

    function focusNext()  { if (focusIndex < 1) focusIndex = focusIndex + 1 }
    function focusPrev()  { if (focusIndex > 0) focusIndex = focusIndex - 1 }
    function clearFocus() { focusIndex = -1 }

    function toggleSearch() {
        if (searchExpanded) {
            searchField.text = ""
            searchField.focus = false
            searchExpanded = false
            searchClosed()
        } else {
            searchExpanded = true
            searchField.forceActiveFocus()
        }
    }

    function searchAppendChar(ch) {
        searchField.text += ch
    }

    function searchAddSpace() {
        searchField.text += " "
    }

    function searchBackspace() {
        if (searchField.text.length > 0) {
            searchField.text = searchField.text.slice(0, -1)
        }
    }

    function searchClear() {
        searchField.text = ""
    }

    function submitSearch() {
        searchSubmitted()
        searchField.focus = false
    }

    Rectangle {
        id: searchBox
        anchors.verticalCenter: parent.verticalCenter
        width: bar.searchExpanded ? 320 * bar._scale : 0
        height: 40 * bar._scale
        radius: 12 * bar._scale
        color: Theme.surface
        border.width: searchField.activeFocus ? Math.max(2, 2 * bar._scale) : 1
        border.color: searchField.activeFocus
            ? Theme.accent
            : Theme.alpha(Theme.text, 0.15)
        clip: true
        visible: width > 1
        opacity: bar.searchExpanded ? 1 : 0

        Behavior on width { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
        Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
        Behavior on border.width { NumberAnimation { duration: 120 } }
        Behavior on border.color { ColorAnimation { duration: 120 } }

        TextInput {
            id: searchField
            anchors.fill: parent
            anchors.leftMargin: 16 * bar._scale
            anchors.rightMargin: 16 * bar._scale
            verticalAlignment: TextInput.AlignVCenter
            color: Theme.text
            selectionColor: Theme.accent
            selectedTextColor: Theme.accentOn
            font.pixelSize: 16 * bar._scale
            font.weight: Font.Medium
            clip: true
            selectByMouse: true

            Keys.onReturnPressed: {
                bar.searchSubmitted()
                searchField.focus = false
            }
            Keys.onEnterPressed: {
                bar.searchSubmitted()
                searchField.focus = false
            }
            Keys.onEscapePressed: bar.toggleSearch()
        }

        Text {
            anchors.fill: parent
            anchors.leftMargin: 16 * bar._scale
            anchors.rightMargin: 16 * bar._scale
            verticalAlignment: Text.AlignVCenter
            text: qsTr("Search")
            color: Theme.textMuted
            font.pixelSize: 16 * bar._scale
            font.weight: Font.Medium
            visible: searchField.text.length === 0 && !searchField.activeFocus
        }
    }

    Item {
        id: searchIconWrap
        width: 40 * bar._scale
        height: 40 * bar._scale
        anchors.verticalCenter: parent.verticalCenter
        readonly property bool gamepadFocused: bar.focusIndex === 0

        Rectangle {
            anchors.fill: parent
            color: Theme.surface
            radius: 12 * bar._scale
            border.width: searchIconWrap.gamepadFocused ? 2 : 0
            border.color: Theme.accent

            Behavior on border.width { NumberAnimation { duration: 140 } }
        }

        SvgIcon {
            anchors.centerIn: parent
            name: "search"
            size: 22 * bar._scale
            color: searchIconWrap.gamepadFocused || searchMouse.containsMouse || bar.searchExpanded
                ? Theme.iconHover
                : Theme.icon
        }

        scale: searchIconWrap.gamepadFocused ? 1.12 : (searchMouse.containsPress ? 0.94 : (searchMouse.containsMouse ? 1.05 : 1.0))
        Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }

        MouseArea {
            id: searchMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: bar.toggleSearch()
        }
    }

    Text {
        id: clock
        anchors.verticalCenter: parent.verticalCenter
        text: Qt.formatTime(_now, "HH:mm")
        color: Theme.text
        font.pixelSize: 18 * bar._scale
        font.weight: Font.Medium

        property date _now: new Date()

        Timer {
            interval: 1000
            running: true
            repeat: true
            onTriggered: clock._now = new Date()
        }
    }

    Item {
        id: appIcon
        width: 40 * bar._scale
        height: 40 * bar._scale
        anchors.verticalCenter: parent.verticalCenter
        readonly property bool gamepadFocused: bar.focusIndex === 1

        Rectangle {
            id: appBg
            anchors.fill: parent
            color: Theme.surface
            radius: 12 * bar._scale
            border.width: appIcon.gamepadFocused ? 2 : 0
            border.color: Theme.accent

            Behavior on border.width { NumberAnimation { duration: 140 } }

            layer.enabled: true
            layer.smooth: true
            layer.textureSize: Qt.size(width * 2, height * 2)

            Image {
                anchors.fill: parent
                anchors.margins: 3 * bar._scale
                source: "qrc:/qt/qml/omikuji/qml/icons/app.png"
                fillMode: Image.PreserveAspectFit
                smooth: true
                mipmap: true
            }

            layer.effect: OpacityMask {
                maskSource: Rectangle {
                    width: appBg.width
                    height: appBg.height
                    radius: 12 * bar._scale
                }
            }
        }

        scale: appIcon.gamepadFocused ? 1.12 : (appMouse.containsPress ? 0.94 : (appMouse.containsMouse ? 1.05 : 1.0))
        Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }

        MouseArea {
            id: appMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: bar.appIconClicked()
        }
    }
}
