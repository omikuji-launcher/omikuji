import QtQuick
import omikuji 1.0


Item {
    id: root

    property string label: ""
    property string description: ""
    // 120 for per-game pages, wider for global settings where labels are longer
    property int labelWidth: 120
    property int contentRightMargin: 98
    default property alias content: contentSlot.children

    readonly property var _content: contentSlot.children.length > 0 ? contentSlot.children[0] : null

    implicitWidth: parent ? parent.width : 400
    implicitHeight: Math.max(labelCol.implicitHeight, _content ? _content.implicitHeight : 0)

    Column {
        id: labelCol
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        width: Math.max(root.labelWidth, contentSlot.x - Theme.space.xl)
        spacing: 2

        Text {
            id: labelText
            text: root.label
            color: Theme.text
            font.pixelSize: Theme.type.subtitle.size
        }

        Text {
            text: root.description
            color: Theme.textSubtle
            font.pixelSize: Theme.type.label.size
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.description !== ""
        }
    }

    Item {
        id: contentSlot
        anchors.right: parent.right
        anchors.rightMargin: root.contentRightMargin
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: root._content ? root._content.width : 0
    }
}
