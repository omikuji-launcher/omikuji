pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

import "."
import "../controls"
import "../downloads"
import "../primitives"
import "../lib/Format.js" as Format

Item {
    id: root

    property var ofudaBridge: null
    property var appSettings: null
    property var prefixes: []
    property var steamPrefixes: []
    property var sizes: ({})

    signal openRequested(var prefix)
    signal createRequested()

    readonly property bool showSteam: appSettings ? appSettings.showSteamPrefixes : false

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
        const paths = prefixes.concat(steamPrefixes).map(p => p.path)
        ofudaBridge.scanSizes(JSON.stringify(paths))
    }

    onOfudaBridgeChanged: {
        if (!ofudaBridge) return
        ofudaBridge.watch()
        refresh()
    }

    Connections {
        target: root.ofudaBridge
        enabled: root.ofudaBridge !== null
        function onChanged() { root.refresh() }
        function onSizeReady(path, bytes) {
            root.sizes = Object.assign({}, root.sizes, { [path]: bytes })
        }
    }

    implicitHeight: content.height

    component StatCell: Column {
        id: cell

        property string label: ""
        property string value: ""
        property real minWidth: 0

        visible: value !== ""
        width: Math.max(implicitWidth, minWidth)
        spacing: 2

        CapsLabel {
            width: cell.width
            horizontalAlignment: Text.AlignHCenter
            text: cell.label
            size: Theme.type.caption.size
        }
        Text {
            width: cell.width
            horizontalAlignment: Text.AlignHCenter
            text: cell.value
            color: Theme.text
            font.pixelSize: Theme.type.label.size
            font.weight: Font.DemiBold
        }
    }

    component PrefixRow: Item {
        id: row

        property var prefix: ({})
        property string detail: ""
        property real sizeBytes: -1

        width: parent.width
        readonly property real topBlock: Math.max(iconBox.height, stats.height, nameText.height)

        height: Theme.space.lg * 2 + topBlock + Theme.space.sm + pathText.height

        Squircle {
            anchors.fill: parent
            radius: Theme.radius.md
            fillColor: Theme.cardBg
        }

        Squircle {
            id: iconBox
            anchors.left: parent.left
            anchors.leftMargin: Theme.space.lg
            anchors.verticalCenter: stats.verticalCenter
            width: 34
            height: 34
            radius: Theme.radius.sm
            fillColor: Theme.alpha(Theme.accent, 0.10)

            SvgIcon {
                anchors.centerIn: parent
                name: row.prefix.kind === "steam" ? "steam" : "ofuda"
                size: 18
                color: Theme.accent
            }
        }

        Text {
            id: nameText
            anchors.left: iconBox.right
            anchors.leftMargin: Theme.space.lg
            anchors.right: stats.left
            anchors.rightMargin: Theme.space.md
            anchors.verticalCenter: stats.verticalCenter
            text: row.prefix.name || ""
            color: Theme.text
            font.pixelSize: Theme.type.headline.size
            font.weight: Font.DemiBold
            elide: Text.ElideRight
        }

        Row {
            id: stats
            anchors.right: manageBtn.left
            anchors.rightMargin: Theme.space.xl
            anchors.top: parent.top
            anchors.topMargin: Theme.space.lg
            spacing: Theme.space.xl

            StatCell {
                label: qsTr("Size")
                value: row.sizeBytes < 0 ? "—" : Format.formatBytes(row.sizeBytes)
                minWidth: 76
            }

            StatCell {
                label: qsTr("Games")
                value: row.detail
                minWidth: 62
            }
        }

        M3Button {
            id: manageBtn
            anchors.right: parent.right
            anchors.rightMargin: Theme.space.md
            anchors.verticalCenter: stats.verticalCenter
            text: qsTr("Manage")
            variant: "tonal"
            onClicked: root.openRequested(row.prefix)
        }

        Text {
            id: pathText
            anchors.left: iconBox.right
            anchors.leftMargin: Theme.space.lg
            anchors.right: parent.right
            anchors.rightMargin: Theme.space.lg
            anchors.top: parent.top
            anchors.topMargin: Theme.space.lg + row.topBlock + Theme.space.sm
            text: row.prefix.path || ""
            color: Theme.accent
            font.pixelSize: Theme.type.caption.size
            font.family: "monospace"
            elide: Text.ElideMiddle
        }
    }

    Column {
        id: content
        width: parent.width
        spacing: Theme.space.xxl

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
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
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
                            : String(modelData.gameCount)
                        sizeBytes: root.sizes[modelData.path] !== undefined ? root.sizes[modelData.path] : -1
                    }
                }

                Text {
                    visible: root.prefixes.length === 0
                    text: qsTr("No prefixes yet.")
                    color: Theme.textSubtle
                    font.pixelSize: Theme.type.caption.size
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
                onToggled: (val) => { if (root.appSettings) root.appSettings.applyShowSteamPrefixes(val) }
            }

            Text {
                text: qsTr("Prefixes of Steam games in the library.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
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
                        sizeBytes: root.sizes[modelData.path] !== undefined ? root.sizes[modelData.path] : -1
                    }
                }

                Text {
                    visible: root.steamPrefixes.length === 0
                    text: qsTr("No Steam game in the library has a prefix yet. Steam creates one on first launch.")
                    color: Theme.textSubtle
                    font.pixelSize: Theme.type.caption.size
                    width: parent.width
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
