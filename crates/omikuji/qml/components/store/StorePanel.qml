import QtQuick
import omikuji 1.0

Rectangle {
    id: panel

    property string viewName: ""
    property string currentView: ""
    property bool unloadIdle: true
    property alias sourceComponent: loader.sourceComponent

    // M3Dropdown walks up looking for this flag to reparent its popup
    property bool isDropdownHost: true

    signal activated()
    signal deactivated()
    signal idleUnloaded()

    anchors.fill: parent
    color: Theme.surface
    radius: Theme.radius.md
    visible: opacity > 0
    opacity: panelActive ? 1 : 0

    readonly property bool panelActive: viewName === currentView
    property bool keepAlive: panelActive

    Behavior on opacity {
        NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
    }

    // corner mask so the rounded bottom-right doesnt let content bleed
    Rectangle {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: parent.radius
        height: parent.radius
        color: parent.color
        visible: parent.visible
    }

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
