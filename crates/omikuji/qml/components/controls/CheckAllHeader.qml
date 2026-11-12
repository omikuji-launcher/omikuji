import QtQuick
import omikuji 1.0

Item {
    id: root

    property string title: ""
    property bool checkAllVisible: true
    property bool checked: false
    property bool indeterminate: false
    property alias trailing: trailingSlot.data

    signal checkAllClicked()

    implicitHeight: Math.max(24, trailingSlot.childrenRect.height)

    Text {
        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        text: root.title
        color: Theme.textSubtle
        font.pixelSize: Theme.type.label.size
        font.weight: Font.DemiBold
    }

    Row {
        id: checkAll
        visible: root.checkAllVisible
        anchors.right: parent.right
        anchors.rightMargin: Theme.space.lg
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.space.sm

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Check all")
            color: Theme.textMuted
            font.pixelSize: Theme.type.label.size
        }

        M3Checkbox {
            anchors.verticalCenter: parent.verticalCenter
            checked: root.checked
            indeterminate: !root.checked && root.indeterminate
        }
    }

    Item {
        id: trailingSlot
        anchors.right: root.checkAllVisible ? checkAll.left : parent.right
        anchors.rightMargin: root.checkAllVisible ? Theme.space.lg : 0
        anchors.verticalCenter: parent.verticalCenter
        width: childrenRect.width
        height: childrenRect.height
    }

    PressArea {
        anchors.fill: checkAll
        anchors.margins: -Theme.space.xs
        enabled: root.checkAllVisible
        onActivated: root.checkAllClicked()
    }
}
