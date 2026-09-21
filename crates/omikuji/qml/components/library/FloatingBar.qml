pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Effects
import QtQuick.Shapes
import "../lib/PlayState.js" as PlayState


Item {
    id: root

    property string barStyle: "floating"

    // the hug cuts the surface corner, so it has to match the radius that drew it
    property real surfaceRadius: Theme.radius.md

    property int barHeight: 56
    property bool barOpaque: false

    readonly property var styleDefaults: ({
        sideInset: 16,
        bottomGap: 14,
        radius: Theme.radius.lg,
        shadow: true,
        hug: false
    })

    readonly property var styleOverrides: ({
        docked: {
            sideInset: 0,
            bottomGap: 0,
            radius: 0,
            shadow: false,
            hug: true
        }
    })

    readonly property var spec: Object.assign({}, root.styleDefaults,
        root.styleOverrides[root.barStyle] || ({}))

    property var selectedGame: null
    property bool hasSelection: false
    property bool isRunning: false
    property bool runnerUpdating: false
    property bool isLaunching: false
    property int launchShowDelay: 120

    // non-null when theres an active download, launching mid-patch reads files the patcher is rewriting
    property var downloadActivity: null

    property var actions: null

    signal settingsClicked()
    signal downloadActivityClicked()
    signal wineToolsClicked()

    // exposed so callers can anchor popups to the wine icon witout window-coord math
    readonly property alias wineToolsAnchor: wineToolsBtn

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

    readonly property int buttonState: PlayState.stateFor(
        root.displayedIsLaunching, root.displayedIsRunning, root.displayedHasActivity)

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

    height: root.barHeight

    RectangularShadow {
        anchors.fill: bar
        anchors.topMargin: 4
        anchors.bottomMargin: -4
        blur: 20
        radius: bar.radius
        color: Qt.rgba(0, 0, 0, 0.45)
        opacity: bar.opacity
        visible: bar.visible && root.spec.shadow
    }

    Squircle {
        id: bar
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: root.spec.bottomGap
        width: parent.width - root.spec.sideInset * 2
        height: root.barHeight
        radius: root.spec.radius
        fillColor: root.barOpaque ? Theme.bg : Theme.barBg
        opacity: root.hasSelection ? 1 : 0
        visible: opacity > 0

        Behavior on opacity {
            NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
        }

        Repeater {
            model: root.spec.hug ? [0, 1] : []

            Item {
                id: patch
                required property int modelData
                readonly property real r: root.surfaceRadius

                x: modelData === 0 ? 0 : bar.width - r
                y: -r
                width: r
                height: r
                transform: Scale { xScale: patch.modelData === 0 ? 1 : -1; origin.x: patch.r / 2 }

                Shape {
                    anchors.fill: parent
                    antialiasing: true
                    preferredRendererType: Shape.CurveRenderer

                    ShapePath {
                        fillColor: bar.fillColor
                        strokeColor: "transparent"
                        startX: 0
                        startY: 0

                        PathLine { x: 0; y: patch.r }
                        PathLine { x: patch.r; y: patch.r }
                        PathAngleArc {
                            centerX: patch.r
                            centerY: 0
                            radiusX: patch.r
                            radiusY: patch.r
                            startAngle: 90
                            sweepAngle: 90
                        }
                    }
                }
            }
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

                GameStatsRow {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    game: root.displayedGame
                    // bar.width not leftWrap.width, leftWrap is momentarily 0 during init while rightCluster resolves
                    nameMaxWidth: Math.max(100, bar.width * 0.4)
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
                icon: "wine"
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

            GameActionButton {
                anchors.verticalCenter: parent.verticalCenter
                actions: root.actions
                playState: root.buttonState
                activity: root.displayedActivity
                runnerUpdating: root.displayedRunnerUpdating
                suppressAnim: root._suppressButtonAnim
                onActivityClicked: root.downloadActivityClicked()
            }
        }
    }
}
