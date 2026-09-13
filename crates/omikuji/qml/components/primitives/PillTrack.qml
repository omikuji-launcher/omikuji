import QtQuick

Item {
    id: root

    property real value: 0
    property color fillColor
    property color trackColor

    implicitWidth: 160
    implicitHeight: 24

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width
        height: Math.round(root.height * 0.5)
        radius: height / 2
        color: root.trackColor

        Rectangle {
            width: Math.max(height, parent.width * root.value)
            height: parent.height
            radius: height / 2
            color: root.fillColor
            visible: root.value > 0
        }
    }
}
