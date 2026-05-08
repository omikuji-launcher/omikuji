import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects

import "../widgets"

Item {
    id: dialog

    property var defaults: null
    property var gameModel: null

    property var sectionLabels: ({
        "wine": "Wine (version, architecture)",
        "sync": "Sync (esync, fsync, ntsync)",
        "translation_layers": "Translation Layers (DXVK, VKD3D, …)",
        "compatibility": "Compatibility (BattlEye, EAC, FSR)",
        "display": "Display (DPI scaling)",
        "drivers": "Drivers (audio, graphics)",
        "dll_overrides": "DLL Overrides",
        "launch": "Launch (command prefix)",
        "environment": "Environment Variables",
        "graphics": "Graphics (MangoHUD, GPU)",
        "gamescope": "Gamescope",
        "performance": "Performance (gamemode, CPU limit)",
        "audio": "Audio (Pulse latency)",
        "power": "Power (prevent sleep)"
    })

    property var availableSections: []
    property var checkedSections: []
    property bool replaceMaps: false

    visible: false
    z: 2100

    function show() {
        if (!defaults) return
        try { dialog.availableSections = JSON.parse(defaults.populatedSectionsJson()) }
        catch (e) { dialog.availableSections = [] }
        dialog.checkedSections = dialog.availableSections.slice()
        dialog.replaceMaps = false
        dialog.visible = true
    }

    function hide() { dialog.visible = false }

    function _toggle(sec) {
        let cur = dialog.checkedSections.slice()
        let idx = cur.indexOf(sec)
        if (idx === -1) cur.push(sec)
        else cur.splice(idx, 1)
        dialog.checkedSections = cur
    }

    function _apply() {
        if (!gameModel || dialog.checkedSections.length === 0) {
            dialog.hide()
            return
        }
        let csv = dialog.checkedSections.join(",")
        gameModel.applyDefaultsToExistingGames(csv, dialog.replaceMaps)
        dialog.hide()
    }

    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, 0.55)
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.AllButtons
            onClicked: (mouse) => { if (mouse.button === Qt.LeftButton) dialog.hide() }
            onWheel: (wheel) => wheel.accepted = true
            cursorShape: Qt.ArrowCursor
        }
    }

    Rectangle {
        id: card
        anchors.centerIn: parent
        width: Math.min(parent.width - 80, 520)
        height: Math.min(parent.height - 120, contentCol.implicitHeight + actionRow.implicitHeight + 56)
        radius: 22
        color: theme.surface
        border.width: 1
        border.color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.08)

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

        Flickable {
            id: cardScroll
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: actionRow.top
            anchors.margins: 22
            anchors.bottomMargin: 12
            contentWidth: width
            contentHeight: contentCol.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            interactive: contentHeight > height
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

        ColumnLayout {
            id: contentCol
            width: cardScroll.width
            spacing: 12

            Text {
                text: "Apply defaults to existing games"
                color: theme.text
                font.pixelSize: 17
                font.weight: Font.DemiBold
            }

            Text {
                Layout.fillWidth: true
                text: "Sections you tick will be written to every game's TOML, overwriting their current values for those fields. Untouched sections stay as they are per-game."
                color: theme.textMuted
                font.pixelSize: 12
                wrapMode: Text.Wrap
                lineHeight: 1.35
            }

            Text {
                Layout.fillWidth: true
                text: "Nothing to apply — set some fields in the Defaults tab first."
                color: theme.textSubtle
                font.pixelSize: 13
                wrapMode: Text.Wrap
                visible: dialog.availableSections.length === 0
            }

            Flickable {
                Layout.fillWidth: true
                Layout.preferredHeight: Math.min(secList.height, 360)
                contentHeight: secList.height
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                interactive: contentHeight > height
                visible: dialog.availableSections.length > 0

                Column {
                    id: secList
                    width: parent.width
                    spacing: 4

                    Repeater {
                        model: dialog.availableSections

                        Item {
                            required property var modelData

                            width: parent.width
                            height: 40

                            readonly property bool selected: dialog.checkedSections.indexOf(modelData) !== -1

                            Rectangle {
                                anchors.fill: parent
                                radius: 8
                                color: rowHover.containsMouse
                                    ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                                    : "transparent"
                                Behavior on color { ColorAnimation { duration: 100 } }
                            }

                            Row {
                                anchors.left: parent.left
                                anchors.leftMargin: 10
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 12

                                SvgIcon {
                                    anchors.verticalCenter: parent.verticalCenter
                                    name: selected ? "check_box" : "check_box_outline_blank"
                                    size: 20
                                    color: selected ? theme.accent : Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.55)
                                }

                                Text {
                                    text: dialog.sectionLabels[modelData] || modelData
                                    color: theme.text
                                    font.pixelSize: 14
                                    anchors.verticalCenter: parent.verticalCenter
                                }
                            }

                            MouseArea {
                                id: rowHover
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: dialog._toggle(modelData)
                            }
                        }
                    }
                }
            }

            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 40
                visible: dialog.availableSections.length > 0
                    && (dialog.checkedSections.indexOf("environment") !== -1
                     || dialog.checkedSections.indexOf("dll_overrides") !== -1)

                Rectangle {
                    anchors.fill: parent
                    radius: 8
                    color: replaceHover.containsMouse
                        ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                        : "transparent"
                    Behavior on color { ColorAnimation { duration: 100 } }
                }

                Row {
                    anchors.left: parent.left
                    anchors.leftMargin: 10
                    anchors.right: parent.right
                    anchors.rightMargin: 10
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 12

                    SvgIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: dialog.replaceMaps ? "check_box" : "check_box_outline_blank"
                        size: 20
                        color: dialog.replaceMaps ? theme.accent : Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.55)
                    }

                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 1
                        Text {
                            text: "Replace env / DLL tables"
                            color: theme.text
                            font.pixelSize: 13
                        }
                        Text {
                            text: dialog.replaceMaps
                                ? "wipes the game's keys, then writes the global ones"
                                : "merges global keys into the game (game keys win on conflict)"
                            color: theme.textSubtle
                            font.pixelSize: 11
                        }
                    }
                }

                MouseArea {
                    id: replaceHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: dialog.replaceMaps = !dialog.replaceMaps
                }
            }
        }
        }

        RowLayout {
            id: actionRow
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 22
            spacing: 10

            Text {
                Layout.fillWidth: true
                text: dialog.gameModel ? "Affects " + dialog.gameModel.count + " game" + (dialog.gameModel.count === 1 ? "" : "s") : ""
                color: theme.textSubtle
                font.pixelSize: 12
            }

            Item {
                implicitWidth: 90
                implicitHeight: 36

                Rectangle {
                    anchors.fill: parent
                    radius: 18
                    color: cancelHover.containsPress
                        ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.12)
                        : cancelHover.containsMouse
                            ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                            : "transparent"
                    Behavior on color { ColorAnimation { duration: 100 } }
                }
                Text {
                    anchors.centerIn: parent
                    text: "Cancel"
                    color: theme.text
                    font.pixelSize: 13
                    font.weight: Font.Medium
                }
                MouseArea {
                    id: cancelHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: dialog.hide()
                }
            }

            Item {
                implicitWidth: 100
                implicitHeight: 36
                opacity: dialog.checkedSections.length > 0 ? 1.0 : 0.5
                enabled: dialog.checkedSections.length > 0

                Rectangle {
                    anchors.fill: parent
                    radius: 18
                    color: theme.accent
                    opacity: applyHover.containsPress ? 0.8
                        : applyHover.containsMouse ? 0.95 : 0.9
                    scale: applyHover.containsPress ? 0.97 : 1.0
                    Behavior on opacity { NumberAnimation { duration: 100 } }
                    Behavior on scale { NumberAnimation { duration: 100 } }
                }
                Text {
                    anchors.centerIn: parent
                    text: "Apply"
                    color: theme.accentOn
                    font.pixelSize: 13
                    font.weight: Font.DemiBold
                }
                MouseArea {
                    id: applyHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: dialog._apply()
                }
            }
        }
    }
}
