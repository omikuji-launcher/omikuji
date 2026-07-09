import QtQuick


Item {
    id: root

    property string icon: ""
    property string label: ""
    property Component action: null
    default property alias content: sectionContent.children

    implicitWidth: parent ? parent.width : 400
    implicitHeight: header.height + 16 + sectionContent.height

    Item {
        id: header
        width: parent.width
        height: actionLoader.active ? Math.max(28, actionLoader.height) : 28

        Text {
            id: headerLabel
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            text: root.label
            color: theme.textMuted
            font.pixelSize: 12
            font.weight: Font.DemiBold
            font.capitalization: Font.AllUppercase
            font.letterSpacing: 0.6
        }

        Rectangle {
            anchors.left: headerLabel.right
            anchors.leftMargin: 12
            anchors.right: actionLoader.active ? actionLoader.left : parent.right
            anchors.rightMargin: actionLoader.active ? 12 : 0
            anchors.verticalCenter: parent.verticalCenter
            height: 1
            color: theme.separator
        }

        Loader {
            id: actionLoader
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            active: root.action !== null
            sourceComponent: root.action
        }
    }

    Column {
        id: sectionContent
        anchors.top: header.bottom
        anchors.topMargin: 16
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 16
    }
}
