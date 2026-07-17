import QtQuick

import "."
import "../controls"
import "../primitives"

Item {
    id: root

    property var ofudaBridge: null
    property var prefixes: []

    signal openRequested(var prefix)
    signal createRequested()

    function refresh() {
        if (!ofudaBridge) return
        try {
            prefixes = JSON.parse(ofudaBridge.listJson()) || []
        } catch (e) {
            prefixes = []
        }
    }

    onOfudaBridgeChanged: {
        if (!ofudaBridge) return
        ofudaBridge.watch()
        refresh()
    }

    Connections {
        target: ofudaBridge
        enabled: ofudaBridge !== null
        function onChanged() { root.refresh() }
    }

    implicitHeight: content.height

    Column {
        id: content
        width: parent.width
        spacing: theme.space.xxl

        SettingsSection {
            label: "Ofuda"
            width: parent.width
            action: M3Button {
                text: qsTr("New prefix")
                variant: "tonal"
                onClicked: root.createRequested()
            }

            Text {
                text: qsTr("Wine prefixes omikuji knows about. Each game lives in one; an orphan is a prefix no game uses anymore.")
                color: theme.textSubtle
                font.pixelSize: theme.type.caption.size
                width: parent.width
                wrapMode: Text.WordWrap
                bottomPadding: 8
            }

            Column {
                width: parent.width
                spacing: 6

                Repeater {
                    model: root.prefixes

                    delegate: Item {
                        id: rowItem
                        required property var modelData
                        width: parent.width
                        height: 62

                        Squircle {
                            anchors.fill: parent
                            radius: theme.radius.md
                            fillColor: theme.cardBg
                        }

                        Column {
                            anchors.left: parent.left
                            anchors.leftMargin: 16
                            anchors.right: rightCluster.left
                            anchors.rightMargin: 14
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 3

                            Text {
                                text: rowItem.modelData.name
                                color: theme.text
                                font.pixelSize: theme.type.body.size
                                font.weight: Font.DemiBold
                                width: parent.width
                                elide: Text.ElideRight
                            }
                            Text {
                                text: rowItem.modelData.gameCount === 0
                                    ? qsTr("Orphan")
                                    : rowItem.modelData.gameCount === 1
                                        ? qsTr("1 game")
                                        : qsTr("%1 games").arg(rowItem.modelData.gameCount)
                                color: theme.textSubtle
                                font.pixelSize: theme.type.caption.size
                                width: parent.width
                                elide: Text.ElideRight
                            }
                        }

                        M3Button {
                            id: rightCluster
                            anchors.right: parent.right
                            anchors.rightMargin: 14
                            anchors.verticalCenter: parent.verticalCenter
                            text: qsTr("Manage")
                            variant: "tonal"
                            onClicked: root.openRequested(rowItem.modelData)
                        }
                    }
                }

                Text {
                    visible: root.prefixes.length === 0
                    text: qsTr("No prefixes yet.")
                    color: theme.textSubtle
                    font.pixelSize: theme.type.caption.size
                    width: parent.width
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
