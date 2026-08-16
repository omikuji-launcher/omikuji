pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import omikuji 1.0
import "../controls"
import "../primitives"

DialogCard {
    id: root
    sizeKey: "found_runners"

    property var archiveManager: null
    property var runners: []

    signal manageRunnerRequested(string name, string kind, string dir, string origin)
    signal deleteRunnerRequested(string name, string dir)

    maxWidth: 720
    scrollable: false
    fillHeight: true
    title: ""

    function show() {
        refresh()
        open()
    }

    function refresh() {
        try { runners = JSON.parse(archiveManager.foundRunners()) } catch (e) { runners = [] }
    }

    onCloseRequested: close()

    actions: Row {
        M3Button {
            text: qsTr("Close")
            variant: "tonal"
            onClicked: root.close()
        }
    }

    body: Item {
        width: parent.width
        height: parent.height

        Item {
            id: bodyHeader
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 64

            Column {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2

                Text {
                    text: qsTr("Found runners")
                    color: Theme.text
                    font.pixelSize: Theme.type.headline.size
                    font.weight: Font.DemiBold
                }

                Text {
                    text: root.runners.length > 0
                        ? qsTr("%1 installed on disk").arg(root.runners.length)
                        : qsTr("Nothing installed yet")
                    color: Theme.textSubtle
                    font.pixelSize: Theme.type.caption.size
                }
            }
        }

        Rectangle {
            id: bodyDivider
            anchors.top: bodyHeader.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: 1
            color: Theme.separator
        }

        ListView {
            id: list
            anchors.top: bodyDivider.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: root.runners
            spacing: 0

            ScrollBar.vertical: ThinScrollBar {}

            Text {
                anchors.centerIn: parent
                visible: list.count === 0
                text: qsTr("No runners found on disk.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.label.size
            }

            delegate: Item {
                id: rr
                required property int index
                required property var modelData

                width: ListView.view.width
                height: 64

                Rectangle {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space.sm
                    anchors.rightMargin: Theme.space.sm
                    anchors.topMargin: 3
                    anchors.bottomMargin: 3
                    radius: Theme.radius.sm
                    color: rrMouse.containsMouse
                        ? Theme.alpha(Theme.text, 0.05)
                        : "transparent"
                    Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
                }

                MouseArea {
                    id: rrMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.NoButton
                }

                Column {
                    anchors.left: parent.left
                    anchors.leftMargin: 24
                    anchors.right: actionSlot.left
                    anchors.rightMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 3

                    Text {
                        width: parent.width
                        text: rr.modelData.name
                        color: Theme.text
                        font.pixelSize: Theme.type.label.size
                        font.weight: Font.Medium
                        font.family: "monospace"
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        text: rr.modelData.origin
                        color: Theme.textSubtle
                        font.pixelSize: Theme.type.caption.size
                        elide: Text.ElideRight
                    }
                }

                Row {
                    id: actionSlot
                    anchors.right: parent.right
                    anchors.rightMargin: 20
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 10

                    IconButton {
                        visible: rr.modelData.kind === "proton"
                        anchors.verticalCenter: parent.verticalCenter
                        icon: "tune"
                        size: 28
                        rounded: true
                        onClicked: root.manageRunnerRequested(rr.modelData.name, rr.modelData.kind, rr.modelData.path, rr.modelData.origin)
                    }

                    IconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        icon: "close"
                        size: 28
                        rounded: false
                        danger: true
                        onClicked: root.deleteRunnerRequested(rr.modelData.name, rr.modelData.path)
                    }
                }

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: Theme.separator
                    visible: rr.index < (list.count - 1)
                }
            }
        }
    }
}
