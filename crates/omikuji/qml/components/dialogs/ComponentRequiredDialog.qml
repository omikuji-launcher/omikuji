pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import omikuji 1.0
import "../controls"
import "../primitives"


DialogCard {
    id: root

    sizeKey: "component_required"

    property var componentsBridge: null
    property string componentName: ""
    property int gameIndex: -1
    property bool skipUpdateCheck: false
    property bool installing: false
    property string phase: ""
    property real percent: 0
    property string errorText: ""

    signal launchReady(int idx, bool skip)

    readonly property var blurbs: ({
        "umu-run": qsTr("Proton runners launch through umu-run. It's a one-time download.")
    })
    readonly property string blurb: blurbs[componentName]
        || qsTr("This game's runner needs %1 before it can start.").arg(componentName)

    maxWidth: 460
    scrollable: false
    title: qsTr("%1 is required").arg(componentName)

    function start(idx, skip, name) {
        gameIndex = idx
        skipUpdateCheck = skip
        componentName = name
        installing = false
        phase = ""
        percent = 0
        errorText = ""
        open()
    }

    function install() {
        if (!componentsBridge || installing) return
        errorText = ""
        phase = ""
        percent = 0
        installing = true
        componentsBridge.installComponent(componentName)
    }

    onCloseRequested: if (!installing) close()

    Connections {
        target: root.componentsBridge
        enabled: root.componentsBridge !== null && root.installing

        function onComponentProgress(name, phase, percent) {
            if (name !== root.componentName) return
            root.phase = phase
            root.percent = percent
        }
        function onComponentCompleted(name, version) {
            if (name !== root.componentName) return
            root.close()
            root.launchReady(root.gameIndex, root.skipUpdateCheck)
            root.installing = false
        }
        function onComponentFailed(name, error) {
            if (name !== root.componentName) return
            root.errorText = (error && error.length > 0) ? error : qsTr("Install failed.")
            root.installing = false
        }
    }

    body: ColumnLayout {
        width: parent.width
        spacing: Theme.space.sm

        Text {
            Layout.fillWidth: true
            text: root.blurb
            color: Theme.textMuted
            font.pixelSize: Theme.type.body.size
            wrapMode: Text.WordWrap
        }

        Text {
            Layout.fillWidth: true
            visible: root.installing || root.errorText !== ""
            text: root.errorText !== ""
                ? root.errorText
                : qsTr("Installing %1... %2").arg(root.componentName).arg(root.phase)
            color: root.errorText !== "" ? Theme.error : Theme.textMuted
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        WavyProgressBar {
            Layout.fillWidth: true
            visible: root.installing
            value: root.percent / 100
            fillColor: Theme.accent
            trackColor: Theme.alpha(Theme.text, 0.16)
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Not now")
            variant: "tonal"
            enabled: !root.installing
            onClicked: root.close()
        }

        M3Button {
            text: root.errorText !== "" ? qsTr("Retry") : qsTr("Install")
            variant: "filled"
            enabled: !root.installing
            onClicked: root.install()
        }
    }
}
