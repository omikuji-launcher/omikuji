import QtQuick
import QtQuick.Controls

ScrollBar {
    id: control

    policy: ScrollBar.AsNeeded
    padding: 2
    minimumSize: 0.1
    visible: size > 0 && size < 1

    onActiveChanged: if (!active) hideTimer.restart()
    Timer { id: hideTimer; interval: 1400 }

    background: Item {}

    contentItem: Rectangle {
        implicitWidth: 4
        implicitHeight: 4
        radius: Math.min(width, height) / 2
        color: theme.alpha(theme.text, control.pressed ? 0.5 : (control.hovered ? 0.4 : 0.25))
        opacity: control.active || control.hovered || hideTimer.running ? 1 : 0

        Behavior on opacity { NumberAnimation { duration: theme.dur.med } }
        Behavior on color { ColorAnimation { duration: theme.dur.fast } }
    }
}
