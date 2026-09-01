pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import Qt5Compat.GraphicalEffects
import "../controls"
import "../primitives"


Item {
    id: root

    property var selectedGame: null
    property bool hasSelection: false
    property bool isRunning: false
    property bool runnerUpdating: false
    property bool isLaunching: false
    property int launchShowDelay: 120

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
        let kindWord = downloadActivity.kind === "update" ? qsTr("Updating") : qsTr("Installing")
        if (s === "Paused") return qsTr("Paused")
        if (s === "Queued") return qsTr("%1 · Queued").arg(kindWord)
        if (s === "Extracting") return qsTr("Extracting")
        if (s === "Patching") return qsTr("Patching")
        let pct = Math.round(downloadActivity.progress || 0)
        return qsTr("%1 · %2%").arg(kindWord).arg(pct)
    }

    function playtimeLabel() {
        let h = displayedGame ? displayedGame.playtime : 0
        return h >= 1 ? Math.floor(h) + "h " + Math.round((h % 1) * 60) + "m"
                      : Math.round(h * 60) + "m"
    }

    property var displayedGame: null
    property bool displayedIsRunning: false
    property bool displayedIsLaunching: false
    property bool displayedRunnerUpdating: false
    property var displayedActivity: null
    readonly property bool displayedHasActivity:
        displayedActivity !== null && displayedActivity !== undefined

    property bool _launchVisible: false
    property bool _barHidden: true
    // prevents the 150ms button crossfade from playing visibly through the 200ms bar fade-in
    property bool _suppressButtonAnim: false

    readonly property string buttonState: displayedIsLaunching ? "launching"
                                        : displayedIsRunning ? "stop"
                                        : displayedHasActivity ? "activity"
                                        : "play"

    component Dot: Rectangle {
        width: 4; height: 4; radius: 2
        color: Theme.dot
        anchors.verticalCenter: parent.verticalCenter
    }

    component Stat: Row {
        id: stat

        property string icon: ""
        property string label: ""
        property color tint: Theme.textMuted

        spacing: 6
        anchors.verticalCenter: parent.verticalCenter

        SvgIcon {
            name: stat.icon
            size: 14
            color: stat.tint
            anchors.verticalCenter: parent.verticalCenter
        }

        Text {
            text: stat.label
            color: stat.tint
            font.pixelSize: Theme.type.caption.size
            anchors.verticalCenter: parent.verticalCenter
        }
    }

    component ButtonSlot: Item {
        id: slot

        property string forState: ""

        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        width: 100
        height: 40
        opacity: root.buttonState === slot.forState ? 1 : 0
        visible: opacity > 0.001

        Behavior on opacity {
            enabled: !root._suppressButtonAnim
            NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
        }
    }

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
            displayedRunnerUpdating = runnerUpdating
            _adoptLaunchState()
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
    function _canSync() {
        return hasSelection && !crossfadeTimer.running
    }
    function _syncIsRunning() {
        if (_canSync()) displayedIsRunning = isRunning
    }
    function _syncIsLaunching() {
        if (_canSync()) displayedIsLaunching = _launchVisible
    }
    function _adoptLaunchState() {
        launchShowTimer.stop()
        _launchVisible = isLaunching
        displayedIsLaunching = _launchVisible
    }
    function _syncActivity() {
        if (_canSync()) displayedActivity = downloadActivity
    }
    function _syncRunnerUpdating() {
        if (_canSync()) displayedRunnerUpdating = runnerUpdating
    }
    onIsRunningChanged: Qt.callLater(_syncIsRunning)
    onDownloadActivityChanged: Qt.callLater(_syncActivity)
    onRunnerUpdatingChanged: Qt.callLater(_syncRunnerUpdating)

    onIsLaunchingChanged: {
        if (isLaunching) {
            launchShowTimer.restart()
            return
        }
        launchShowTimer.stop()
        _launchVisible = false
        Qt.callLater(_syncIsLaunching)
    }

    Timer {
        id: launchShowTimer
        interval: root.launchShowDelay
        onTriggered: {
            root._launchVisible = true
            root._syncIsLaunching()
        }
    }

    Timer {
        id: crossfadeTimer
        interval: 100
        onTriggered: {
            root.displayedGame = root.selectedGame
            root.displayedIsRunning = root.isRunning
            root.displayedRunnerUpdating = root.runnerUpdating
            root._adoptLaunchState()
            root.displayedActivity = root.downloadActivity
            barContent.opacity = 1
        }
    }

    height: 56

    RectangularGlow {
        anchors.fill: bar
        anchors.topMargin: 4
        anchors.bottomMargin: -4
        glowRadius: 20
        spread: 0.06
        color: Qt.rgba(0, 0, 0, 0.45)
        cornerRadius: Theme.radius.lg + 20
        opacity: bar.opacity
        visible: bar.visible
    }

    Squircle {
        id: bar
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 14
        width: parent.width - 32
        height: 56
        radius: Theme.radius.lg
        fillColor: Theme.barBg
        opacity: root.hasSelection ? 1 : 0
        visible: opacity > 0

        Behavior on opacity {
            NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
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
                        color: Theme.text
                        font.pixelSize: Theme.type.body.size
                        font.weight: Font.DemiBold
                        elide: Text.ElideRight
                        // bar.width not leftWrap.width, leftWrap is momentarily 0 during init while rightCluster resolves
                        width: Math.min(implicitWidth, Math.max(100, bar.width * 0.4))
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Dot {}

                    Stat {
                        icon: "schedule"
                        label: root.playtimeLabel()
                        tint: Theme.textMuted
                    }

                    Dot {}

                    Stat {
                        icon: "calendar_month"
                        label: root.displayedGame ? root.displayedGame.lastPlayed : ""
                        tint: Theme.textSubtle
                    }

                    Dot {}

                    Text {
                        text: root.displayedGame ? root.displayedGame.runner : ""
                        color: Theme.textFaint
                        font.pixelSize: Theme.type.caption.size
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
                width: root.buttonState === "activity" ? 150 : 100
                height: 40
                anchors.verticalCenter: parent.verticalCenter

                Behavior on width {
                    NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                }

                ButtonSlot {
                    forState: "launching"

                    M3Button {
                        anchors.fill: parent
                        variant: "filled"
                        enabled: false
                        text: qsTr("Starting")
                    }
                }

                ButtonSlot {
                    forState: "stop"

                    M3Button {
                        anchors.fill: parent
                        variant: "filled"
                        danger: true
                        text: qsTr("Stop")
                        onClicked: root.stopClicked()
                    }
                }

                ButtonSlot {
                    forState: "activity"
                    width: parent.width

                    Squircle {
                        anchors.fill: parent
                        radius: Theme.radius.lg
                        fillColor: Theme.alpha(Theme.text, 0.08)

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
                            color: Theme.alpha(Theme.accent, 0.15)
                        }

                        Row {
                            anchors.centerIn: parent
                            spacing: 6

                            SvgIcon {
                                anchors.verticalCenter: parent.verticalCenter
                                name: "schedule"
                                size: 14
                                color: Theme.accent
                            }

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: root.activityLabel()
                                color: Theme.text
                                font.pixelSize: Theme.type.micro.size
                                font.weight: Font.DemiBold
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.downloadActivityClicked()
                        }
                    }
                }

                ButtonSlot {
                    forState: "play"

                    M3Button {
                        anchors.fill: parent
                        variant: "filled"
                        enabled: !root.displayedRunnerUpdating
                        text: qsTr("Play")
                        onClicked: root.playClicked()
                    }
                }
            }
        }
    }
}
