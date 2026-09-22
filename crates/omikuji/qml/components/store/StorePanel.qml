import QtQuick
import omikuji 1.0

FadePanel {
    id: panel

    property string viewName: ""
    property string currentView: ""
    property bool unloadIdle: true
    property alias sourceComponent: loader.sourceComponent

    signal activated()
    signal deactivated()
    signal idleUnloaded()

    shown: panelActive

    readonly property bool panelActive: viewName === currentView
    property bool keepAlive: panelActive

    onPanelActiveChanged: {
        if (panelActive) {
            idleTimer.stop()
            keepAlive = true
            panel.activated()
        } else {
            panel.deactivated()
            if (unloadIdle) idleTimer.restart()
        }
    }

    Timer {
        id: idleTimer
        interval: 10000
        onTriggered: {
            panel.keepAlive = false
            Qt.callLater(() => panel.idleUnloaded())
        }
    }

    Loader {
        id: loader
        anchors.fill: parent
        active: panel.keepAlive
    }
}
