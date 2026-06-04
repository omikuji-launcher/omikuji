import QtQuick
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects

import "../widgets"

Item {
    id: root

    property var selectedGame: null
    property bool hasSelection: false
    property bool isRunning: false

    // non-null when theres an active download, launching mid-patch reads files the patcher is rewriting
    property var downloadActivity: null

    signal settingsClicked()
    signal playClicked()
    signal stopClicked()
    signal downloadActivityClicked()
    signal wineToolsClicked()

    // exposed so callers can anchor popups to the wine icon witout window-coord math
    readonly property alias wineToolsAnchor: wineToolsBtn

    readonly property bool hasActivity: downloadActivity !== null && downloadActivity !== undefined

    function activityLabel() {
        if (!hasActivity) return ""
        let s = downloadActivity.status || ""
        let kindWord = downloadActivity.kind === "update" ? "Updating" : "Installing"
        if (s === "Paused") return "Paused"
        if (s === "Queued") return kindWord + " · Queued"
        if (s === "Extracting") return "Extracting"
        if (s === "Patching") return "Patching"
        let pct = Math.round(downloadActivity.progress || 0)
        return kindWord + " · " + pct + "%"
    }

    property var displayedGame: null
    property bool displayedIsRunning: false
    property var displayedActivity: null
    readonly property bool displayedHasActivity:
        displayedActivity !== null && displayedActivity !== undefined

    property bool _barHidden: true
    // prevents the 150ms button crossfade from playing visibly through the 200ms bar fade-in
    property bool _suppressButtonAnim: false

    onSelectedGameChanged: {
        if (!selectedGame || !selectedGame.name) {
            // keep displayed* intact so the close animation shows the previous state
            _barHidden = true
            return
        }
        if (_barHidden || !displayedGame || !displayedGame.name) {
            _suppressButtonAnim = true
            displayedGame = selectedGame
            displayedIsRunning = isRunning
            displayedActivity = downloadActivity
            barContent.opacity = 1
            _barHidden = false
            // re-enable on the next tick so subsequent state flips animate normally
            Qt.callLater(function() { root._suppressButtonAnim = false })
            return
        }
        barContent.opacity = 0
        crossfadeTimer.start()
    }

    // Qt.callLater lets bindings settle, deselect can fire isRunning false before selectedGame null
    function _syncIsRunning() {
        if (hasSelection) displayedIsRunning = isRunning
    }
    function _syncActivity() {
        if (hasSelection) displayedActivity = downloadActivity
    }
    onIsRunningChanged: Qt.callLater(_syncIsRunning)
    onDownloadActivityChanged: Qt.callLater(_syncActivity)

    Timer {
        id: crossfadeTimer
        interval: 100
        onTriggered: {
            root.displayedGame = root.selectedGame
            root.displayedIsRunning = root.isRunning
            root.displayedActivity = root.downloadActivity
            barContent.opacity = 1
        }
    }

    height: 56

    Rectangle {
        id: bar
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 14
        width: parent.width - 32
        height: 56
        radius: 16
        color: theme.barBg
        border.width: 1
        border.color: theme.barBorder
        opacity: root.hasSelection ? 1 : 0
        visible: opacity > 0

        Behavior on opacity {
            NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
        }

        layer.enabled: true
        layer.effect: DropShadow {
            transparentBorder: true
            horizontalOffset: 0
            verticalOffset: 4
            radius: 20
            samples: 41
            color: Qt.rgba(0, 0, 0, 0.45)
        }

        Item {
            id: barContent
            anchors.fill: parent
            opacity: 1

            Behavior on opacity {
                NumberAnimation { duration: 100 }
            }

            Item {
                id: leftWrap
                // explicit width because a forward anchor to rightCluster resolves to 0 on init and clip eat everything
                anchors.left: parent.left
                anchors.leftMargin: 20
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: Math.max(0, parent.width - 20 - rightCluster.width - 28)
                clip: true

                Row {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 16

                    Text {
                        text: root.displayedGame ? root.displayedGame.name : ""
                        color: theme.text
                        font.pixelSize: 14
                        font.weight: Font.DemiBold
                        elide: Text.ElideRight
                        // bar.width not leftWrap.width, leftWrap is momentarily 0 during init while rightCluster resolves
                        width: Math.min(implicitWidth, Math.max(100, bar.width * 0.4))
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Rectangle {
                        width: 4; height: 4; radius: 2
                        color: theme.dot
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Row {
                        spacing: 6
                        anchors.verticalCenter: parent.verticalCenter

                        SvgIcon {
                            name: "schedule"
                            size: 14
                            color: theme.textMuted
                            anchors.verticalCenter: parent.verticalCenter
                        }

                        Text {
                            property real hours: root.displayedGame ? root.displayedGame.playtime : 0
                            text: hours >= 1 ? Math.floor(hours) + "h " + Math.round((hours % 1) * 60) + "m"
                                             : Math.round(hours * 60) + "m"
                            color: theme.textMuted
                            font.pixelSize: 12
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }

                    Rectangle {
                        width: 4; height: 4; radius: 2
                        color: theme.dot
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Row {
                        spacing: 6
                        anchors.verticalCenter: parent.verticalCenter

                        SvgIcon {
                            name: "calendar_month"
                            size: 14
                            color: theme.textSubtle
                            anchors.verticalCenter: parent.verticalCenter
                        }

                        Text {
                            text: root.displayedGame ? root.displayedGame.lastPlayed : ""
                            color: theme.textSubtle
                            font.pixelSize: 12
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }

                    Rectangle {
                        width: 4; height: 4; radius: 2
                        color: theme.dot
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Text {
                        text: root.displayedGame ? root.displayedGame.runner : ""
                        color: theme.textFaint
                        font.pixelSize: 11
                        font.family: "monospace"
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }
        }

        Row {
            id: rightCluster
            anchors.right: parent.right
            anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            spacing: 8

            IconButton {
                id: wineToolsBtn
                icon: "wine_bar"
                size: 40
                rounded: true
                visible: !root.displayedGame || (root.displayedGame.runnerType !== "native" && root.displayedGame.runnerType !== "flatpak")
                anchors.verticalCenter: parent.verticalCenter
                onClicked: root.wineToolsClicked()
            }

            IconButton {
                icon: "settings"
                size: 40
                rounded: true
                anchors.verticalCenter: parent.verticalCenter
                onClicked: root.settingsClicked()
            }

            Item {
                width: root.displayedHasActivity ? 150 : 100
                height: 40
                anchors.verticalCenter: parent.verticalCenter

                Behavior on width {
                    NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                }

                Item {
                    id: stopSlot
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    width: 100
                    height: 40
                    opacity: root.displayedIsRunning ? 1 : 0
                    visible: opacity > 0.001

                    Behavior on opacity {
                        enabled: !root._suppressButtonAnim
                        NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                    }

                    Rectangle {
                        id: stopBtn
                        anchors.fill: parent
                        radius: 12
                        color: Qt.rgba(0.9, 0.25, 0.25, 1.0)
                        opacity: stopMouse.containsPress ? 0.8 : (stopMouse.containsHover ? 0.95 : 0.9)
                        scale: stopMouse.containsPress ? 0.97 : 1.0

                        Behavior on opacity {
                            NumberAnimation { duration: 100 }
                        }
                        Behavior on scale {
                            NumberAnimation { duration: 100; easing.type: Easing.OutCubic }
                        }

                        Text {
                            anchors.centerIn: parent
                            text: "Stop"
                            color: "white"
                            font.pixelSize: 14
                            font.weight: Font.DemiBold
                        }

                        MouseArea {
                            id: stopMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.stopClicked()
                        }
                    }
                }

                Rectangle {
                    id: activityBtn
                    anchors.fill: parent
                    radius: 12
                    color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.08)
                    border.width: 1
                    border.color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.6)
                    opacity: (!root.displayedIsRunning && root.displayedHasActivity) ? 1 : 0
                    visible: opacity > 0

                    Behavior on opacity {
                        enabled: !root._suppressButtonAnim
                        NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                    }

                    // no width Behavior, it raced the opacity fade and painted outside the rounded bounds (XDDDDDDDDDDDDDDZ))IS)D(ISDJ(SJD))
                    clip: true
                    Rectangle {
                        anchors.left: parent.left
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        anchors.margins: 1
                        radius: 11
                        width: {
                            if (!root.displayedHasActivity) return 0
                            let pct = (root.displayedActivity.progress || 0) / 100
                            return Math.max(0, Math.min((parent.width - 2) * pct, parent.width - 2))
                        }
                        color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.15)
                    }

                    Row {
                        anchors.centerIn: parent
                        spacing: 6

                        SvgIcon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: "schedule"
                            size: 14
                            color: theme.accent
                        }

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: root.activityLabel()
                            color: theme.text
                            font.pixelSize: 12
                            font.weight: Font.DemiBold
                        }
                    }

                    MouseArea {
                        id: activityMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.downloadActivityClicked()
                    }
                }

                Item {
                    id: playSlot
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    width: 100
                    height: 40
                    opacity: (!root.displayedIsRunning && !root.displayedHasActivity) ? 1 : 0
                    visible: opacity > 0.001

                    Behavior on opacity {
                        enabled: !root._suppressButtonAnim
                        NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                    }

                    Rectangle {
                        id: playBtnBg
                        anchors.fill: parent
                        radius: 12
                        color: theme.accent
                        opacity: playMouse.containsPress ? 0.8 : (playMouse.containsHover ? 0.95 : 0.9)
                        scale: playMouse.containsPress ? 0.97 : 1.0

                        Behavior on opacity {
                            NumberAnimation { duration: 100 }
                        }
                        Behavior on scale {
                            NumberAnimation { duration: 100; easing.type: Easing.OutCubic }
                        }

                        Text {
                            anchors.centerIn: parent
                            text: "Play"
                            color: theme.accentOn
                            font.pixelSize: 14
                            font.weight: Font.DemiBold
                        }

                        MouseArea {
                            id: playMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.playClicked()
                        }
                    }
                }
            }
        }
    }
}
