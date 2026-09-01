import QtQuick
import omikuji 1.0
import "../primitives"

Item {
    id: root

    property bool checked: false
    property bool indeterminate: false
    property int boxSize: 20

    readonly property bool _on: checked || indeterminate
    readonly property color _outlineOff: Theme.alpha(Theme.text, 0.5)

    implicitWidth: boxSize
    implicitHeight: boxSize
    opacity: enabled ? 1 : 0.45

    Rectangle {
        id: box
        anchors.centerIn: parent
        width: root.boxSize
        height: root.boxSize
        radius: Math.max(2, Math.round(root.boxSize * 0.14))
        color: root._on ? Theme.accent : "transparent"
        border.width: 2
        border.color: root._on ? Theme.accent : root._outlineOff

        Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
        Behavior on border.color { ColorAnimation { duration: Theme.dur.fast } }

        SvgIcon {
            anchors.centerIn: parent
            name: "check"
            size: root.boxSize
            color: Theme.accentOn
            opacity: root.checked && !root.indeterminate ? 1 : 0

            Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
        }

        Rectangle {
            anchors.centerIn: parent
            width: Math.round(root.boxSize * 0.55)
            height: 2
            radius: 1
            color: Theme.accentOn
            opacity: root.indeterminate ? 1 : 0

            Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
        }
    }
}
