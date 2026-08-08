import QtQuick
import omikuji 1.0
import "../primitives"

Item {
    id: root

    property string icon: ""
    property int iconSize: 20
    property bool primary: true

    signal clicked()

    width: 28
    height: 28

    // prevents hover tint staying latched after click opens a modal (which suppresses mouse-exited)
    property bool _suppressHover: false

    Rectangle {
        anchors.fill: parent
        radius: width / 2
        visible: root.primary
        color: hoverArea.containsPress
            ? Theme.alpha(Theme.accent, 0.3)
            : (hoverArea.containsMouse && !root._suppressHover)
                ? Theme.alpha(Theme.accent, 0.2)
                : "transparent"
        Behavior on color { ColorAnimation { duration: 100 } }
    }

    SvgIcon {
        anchors.centerIn: parent
        name: root.icon
        size: root.iconSize
        color: Theme.accent
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: root.primary
        cursorShape: root.primary ? Qt.PointingHandCursor : Qt.ArrowCursor
        acceptedButtons: root.primary ? Qt.LeftButton : Qt.NoButton
        onClicked: {
            root.clicked()
            root._suppressHover = true
        }
        onPositionChanged: root._suppressHover = false
        onEntered: root._suppressHover = false
        onExited: root._suppressHover = false
    }
}
