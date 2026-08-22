import QtQuick
import omikuji 1.0

Item {
    id: root

    property string label: ""
    property string description: ""
    property bool checked: false
    property int labelWidth: 240

    signal toggled(bool val)

    implicitWidth: root.labelWidth + 8 + sw.implicitWidth
    implicitHeight: Math.max(sw.implicitHeight, labelCol.implicitHeight)

    Column {
        id: labelCol
        anchors.left: parent.left
        anchors.right: sw.left
        anchors.rightMargin: 8
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Text {
            width: parent.width
            text: root.label
            color: root.enabled ? Theme.text : Theme.textSubtle
            font.pixelSize: Theme.type.subtitle.size
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            visible: root.description !== ""
            text: root.description
            color: Theme.textSubtle
            font.pixelSize: Theme.type.label.size
            wrapMode: Text.WordWrap
        }
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
