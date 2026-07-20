import QtQuick

import "."
import "../controls"
import "../primitives"

Item {
    id: root

    property var ofudaBridge: null
    property var uiSettings: null
    property var prefixes: []
    property var steamPrefixes: []

    signal openRequested(var prefix)
    signal createRequested()

    readonly property bool showSteam: uiSettings ? uiSettings.showSteamPrefixes : false

    function refresh() {
        if (!ofudaBridge) return
        try {
            prefixes = JSON.parse(ofudaBridge.listJson()) || []
        } catch (e) {
            prefixes = []
        }
        try {
            steamPrefixes = JSON.parse(ofudaBridge.listSteamJson()) || []
        } catch (e) {
            steamPrefixes = []
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

    component PrefixRow: Item {
        id: row

        property var prefix: ({})
        property string detail: ""

        width: parent.width
        height: rowText.height + theme.space.lg + theme.space.xs

        Squircle {
            anchors.fill: parent
            radius: theme.radius.md
            fillColor: theme.cardBg
        }

        Column {
            id: rowText
            anchors.left: parent.left
            anchors.leftMargin: theme.space.lg
            anchors.right: manageBtn.left
            anchors.rightMargin: theme.space.md
            anchors.verticalCenter: parent.verticalCenter
            spacing: 6

            Item {
                width: parent.width
                height: nameText.implicitHeight

                Text {
                    id: nameText
                    text: row.prefix.name || ""
                    color: theme.text
                    font.pixelSize: theme.type.subtitle.size
                    font.weight: Font.DemiBold
                    elide: Text.ElideRight
                    width: Math.min(implicitWidth, parent.width - (detailText.visible ? detailText.implicitWidth + theme.space.sm : 0))
                }
                Text {
                    id: detailText
                    visible: row.detail !== ""
                    anchors.left: nameText.right
                    anchors.leftMargin: theme.space.sm
                    anchors.baseline: nameText.baseline
                    text: "· " + row.detail
                    color: theme.textSubtle
                    font.pixelSize: theme.type.caption.size
                }
            }

            Text {
                width: parent.width
                text: row.prefix.path || ""
                color: theme.accent
                font.pixelSize: theme.type.caption.size
                font.family: "monospace"
                elide: Text.ElideMiddle
            }
        }

        M3Button {
            id: manageBtn
            anchors.right: parent.right
            anchors.rightMargin: theme.space.md
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Manage")
            variant: "tonal"
            onClicked: root.openRequested(row.prefix)
        }
    }

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

                    delegate: PrefixRow {
                        required property var modelData
                        prefix: modelData
                        detail: modelData.gameCount === 0
                            ? qsTr("Orphan")
                            : modelData.gameCount === 1
                                ? qsTr("1 game")
                                : qsTr("%1 games").arg(modelData.gameCount)
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

        SettingsSection {
            label: qsTr("Steam")
            width: parent.width
            action: M3Switch {
                checked: root.showSteam
                onToggled: (val) => { if (root.uiSettings) root.uiSettings.applyShowSteamPrefixes(val) }
            }

            Text {
                text: qsTr("Prefixes of Steam games in the library.")
                color: theme.textSubtle
                font.pixelSize: theme.type.caption.size
                width: parent.width
                wrapMode: Text.WordWrap
                bottomPadding: 8
            }

            Column {
                visible: root.showSteam
                width: parent.width
                spacing: 6

                Repeater {
                    model: root.steamPrefixes

                    delegate: PrefixRow {
                        required property var modelData
                        prefix: modelData
                    }
                }

                Text {
                    visible: root.steamPrefixes.length === 0
                    text: qsTr("No Steam game in the library has a prefix yet. Steam creates one on first launch.")
                    color: theme.textSubtle
                    font.pixelSize: theme.type.caption.size
                    width: parent.width
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
