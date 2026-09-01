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

    signal deleteRunnerRequested(string name, string dir)
    signal steamActionRequested(string name, string dir, string action)

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

            section.property: "origin"
            section.criteria: ViewSection.FullString
            section.delegate: Item {
                id: originHeader
                required property string section

                width: ListView.view.width
                height: 36

                Text {
                    id: originLabel
                    anchors.left: parent.left
                    anchors.leftMargin: 24
                    anchors.verticalCenter: parent.verticalCenter
                    text: originHeader.section
                    color: Theme.textSubtle
                    font.pixelSize: Theme.type.micro.size
                    font.weight: Font.DemiBold
                    font.capitalization: Font.AllUppercase
                    font.letterSpacing: 0.6
                }

                Rectangle {
                    anchors.left: originLabel.right
                    anchors.leftMargin: Theme.space.sm
                    anchors.right: parent.right
                    anchors.rightMargin: 24
                    anchors.verticalCenter: parent.verticalCenter
                    height: 1
                    color: Theme.separator
                }
            }

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
                height: 48

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

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: 24
                    anchors.right: actionSlot.left
                    anchors.rightMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    text: rr.modelData.name
                    color: Theme.text
                    font.pixelSize: Theme.type.label.size
                    font.weight: Font.Medium
                    font.family: "monospace"
                    elide: Text.ElideRight
                }

                Row {
                    id: actionSlot
                    anchors.right: parent.right
                    anchors.rightMargin: 20
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 10

                    IconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: rr.modelData.steam_action !== "none"
                        icon: "steam"
                        size: 32
                        tonal: true
                        squircle: true
                        onClicked: root.steamActionRequested(rr.modelData.name,
                                                            rr.modelData.path,
                                                            rr.modelData.steam_action)
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
                        && root.runners[rr.index + 1].origin === rr.modelData.origin
                }
            }
        }
    }
}
