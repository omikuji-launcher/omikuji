import QtQuick
import omikuji 1.0

Item {
    id: root

    property string label: ""
    property alias contentSpacing: inner.spacing
    default property alias content: inner.children

    implicitHeight: header.height + (header.visible ? Theme.space.sm : 0) + inner.height

    Item {
        id: header
        width: parent.width
        height: visible ? labelText.implicitHeight : 0
        visible: root.label !== ""

        Text {
            id: labelText
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            text: root.label
            color: Theme.textMuted
            font.pixelSize: Theme.type.micro.size
            font.weight: Font.DemiBold
            font.capitalization: Font.AllUppercase
            font.letterSpacing: 0.6
        }

        Rectangle {
            anchors.left: labelText.right
            anchors.leftMargin: Theme.space.md
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            height: 1
            color: Theme.separator
        }
    }

    Column {
        id: inner
        anchors.top: header.bottom
        anchors.topMargin: header.visible ? Theme.space.sm : 0
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.space.sm
    }
}
