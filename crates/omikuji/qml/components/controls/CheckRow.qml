import QtQuick
import QtQuick.Layouts
import omikuji 1.0

Item {
    id: root

    property bool checked: false
    property bool showCheck: true
    property string icon: ""
    property string text: ""
    property string hint: ""
    property real minHeight: 40
    default property alias content: layout.data

    signal toggled()

    implicitWidth: layout.implicitWidth + 20
    implicitHeight: Math.max(minHeight, layout.implicitHeight + Theme.space.sm)

    Rectangle {
        anchors.fill: parent
        radius: Theme.radius.sm
        color: press.containsMouse ? Theme.stateHover : "transparent"

        Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
    }

    PressArea {
        id: press
        anchors.fill: parent
        hoverEnabled: true
        onActivated: root.toggled()
    }

    RowLayout {
        id: layout
        anchors.fill: parent
        anchors.leftMargin: 10
        anchors.rightMargin: 10
        spacing: Theme.space.md

        M3Checkbox {
            visible: root.showCheck
            checked: root.checked
        }

        SvgIcon {
            visible: root.icon !== ""
            name: root.icon
            size: 18
            color: Theme.icon
        }

        Column {
            visible: root.text !== ""
            Layout.fillWidth: true
            spacing: 1

            Text {
                width: parent.width
                text: root.text
                color: Theme.text
                font.pixelSize: Theme.type.body.size
                elide: Text.ElideRight
            }

            Text {
                visible: root.hint !== ""
                width: parent.width
                text: root.hint
                color: Theme.textSubtle
                font.pixelSize: Theme.type.micro.size
                wrapMode: Text.Wrap
            }
        }
    }
}
