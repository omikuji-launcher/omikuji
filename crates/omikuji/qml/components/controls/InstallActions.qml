pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property bool installed: false
    property bool busy: false

    signal installRequested()
    signal deleteRequested()

    implicitWidth: 132
    implicitHeight: 30

    M3Button {
        anchors.centerIn: parent
        visible: !root.installed && !root.busy
        text: qsTr("Install")
        variant: "filled"
        onClicked: root.installRequested()
    }

    Row {
        anchors.centerIn: parent
        visible: root.installed && !root.busy
        spacing: 8

        M3Button {
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Installed")
            variant: "tonal"
            enabled: false
            opacity: 0.75
        }

        IconButton {
            anchors.verticalCenter: parent.verticalCenter
            icon: "close"
            size: 28
            rounded: false
            danger: true
            onClicked: root.deleteRequested()
        }
    }

    Text {
        anchors.centerIn: parent
        visible: root.busy
        text: qsTr("Working…")
        color: Theme.textMuted
        font.pixelSize: Theme.type.caption.size
    }
}
