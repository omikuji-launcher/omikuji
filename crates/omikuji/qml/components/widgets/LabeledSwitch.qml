import QtQuick
import "."

Item {
    id: root

    property string label: ""
    property bool checked: false
    property int labelWidth: 240

    signal toggled(bool val)

    implicitWidth: root.labelWidth + 8 + sw.implicitWidth
    implicitHeight: sw.implicitHeight

    Text {
        id: labelText
        anchors.left: parent.left
        anchors.right: sw.left
        anchors.rightMargin: 8
        anchors.verticalCenter: parent.verticalCenter
        text: root.label
        color: root.enabled ? theme.text : theme.textSubtle
        font.pixelSize: 15
        elide: Text.ElideRight
    }

    M3Switch {
        id: sw
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        opacity: root.enabled ? 1 : 0.45
        checked: root.checked
        onToggled: (val) => root.toggled(val)
    }
}
