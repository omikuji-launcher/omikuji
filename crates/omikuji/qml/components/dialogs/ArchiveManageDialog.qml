pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Controls
import "../controls"
import "../primitives"
import "../lib/ArchiveAssets.js" as AA


DialogCard {
    sizeKey: "archive_manage"
    id: root

    property var archiveManager: null
    property var activeInstalls: ({})

    property string category: ""
    property string sourceName: ""
    property string sourceKind: ""

    property var versions: []
    property var installedDirs: ({})

    property string errorMessage: ""
    property bool fetching: false

    signal closed()
    signal versionDeleted(string category, string sourceName, string tag)
    signal removeSourceRequested(string category, string sourceName)
    signal moveToSteamRequested(string sourceName, string tag)

    maxWidth: 720
    scrollable: false
    fillHeight: true
    title: ""

    function show(cat, name, kind) {
        category = cat
        sourceName = name
        sourceKind = kind
        versions = []
        installedDirs = ({})
        errorMessage = ""
        refreshInstalled()
        open()
        fetchVersionsNow()
    }

    function hide() {
        root.closed()
        close()
    }

    function refreshInstalled() {
        if (!archiveManager || sourceName === "") return
        try {
            let raw = archiveManager.listInstalled(category, sourceName)
            let list = JSON.parse(raw) || []
            let map = ({})
            for (let i = 0; i < list.length; i++) map[list[i]] = true
            installedDirs = map
        } catch (e) {
            console.warn("installedDirs parse failed:", e)
            installedDirs = ({})
        }
    }

    function fetchVersionsNow() {
        if (!archiveManager || sourceName === "") return
        fetching = true
        errorMessage = ""
        archiveManager.fetchVersions(category, sourceName)
    }

    onCloseRequested: { root.closed(); root.close() }

    footerLeft: M3Button {
        text: qsTr("Remove source")
        variant: "tonal"
        danger: true
        onClicked: root.removeSourceRequested(root.category, root.sourceName)
    }

    actions: Row {
        M3Button {
            text: qsTr("Close")
            variant: "tonal"
            onClicked: { root.closed(); root.close() }
        }
    }

    Connections {
        target: root.archiveManager
        enabled: root.shown && root.archiveManager !== null

        function onVersionsReady(cat, name, json) {
            if (cat !== root.category || name !== root.sourceName) return
            root.fetching = false
            try {
                root.versions = JSON.parse(json) || []
            } catch (e) {
                root.versions = []
                root.errorMessage = qsTr("Couldn't parse versions response.")
            }
        }
        function onVersionsFailed(cat, name, err) {
            if (cat !== root.category || name !== root.sourceName) return
            root.fetching = false
            root.errorMessage = err
        }
        function onInstallCompleted(cat, name, tag, installDir) {
            if (cat !== root.category || name !== root.sourceName) return
            root.refreshInstalled()
        }
        function onInstallFailed(cat, name, tag, err) {
            if (cat !== root.category || name !== root.sourceName) return
            root.errorMessage = err
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

                Row {
                    spacing: 10
                    Text {
                        text: root.sourceName
                        color: Theme.text
                        font.pixelSize: Theme.type.headline.size
                        font.weight: Font.DemiBold
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Rectangle {
                        height: 18
                        width: kindLabel.width + 14
                        radius: 9
                        color: Theme.alpha(Theme.accent, 0.15)
                        anchors.verticalCenter: parent.verticalCenter
                        Text {
                            id: kindLabel
                            anchors.centerIn: parent
                            text: root.sourceKind
                            color: Theme.accent
                            font.pixelSize: Theme.type.micro.size
                            font.weight: Font.Medium
                            font.capitalization: Font.AllUppercase
                            font.letterSpacing: 0.6
                        }
                    }
                }

                Text {
                    text: root.fetching ? qsTr("Fetching versions…")
                        : root.versions.length > 0 ? qsTr("%1 versions available").arg(root.versions.length)
                        : root.errorMessage !== "" ? root.errorMessage
                        : qsTr("No versions loaded yet")
                    color: root.errorMessage !== "" ? Theme.error : Theme.textSubtle
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
            model: root.versions
            spacing: 0

            ScrollBar.vertical: ThinScrollBar {}

            Text {
                anchors.centerIn: parent
                visible: list.count === 0
                text: root.fetching ? qsTr("Loading…")
                    : root.errorMessage !== "" ? qsTr("Couldn't load versions.")
                    : qsTr("No versions available.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.label.size
            }

            delegate: Item {
                id: versionRow
                required property int index
                required property var modelData

                readonly property string tag: modelData.tag || ""
                readonly property string publishedAt: modelData.published_at || ""
                readonly property var assets: modelData.assets || []
                property int assetIndex: {
                    for (var i = 0; i < assets.length; i++)
                        if (assets[i].name === modelData.asset_name) return i
                    return 0
                }
                readonly property var chosenAsset: assets[assetIndex] || null
                readonly property int assetSize: chosenAsset ? (chosenAsset.size || 0) : (modelData.asset_size || 0)
                readonly property var assetStems: AA.stems(assets)
                readonly property var assetLabels: AA.labels(assets)
                readonly property bool installed: root.installedDirs[assetStems[assetIndex]] === true
                readonly property bool busy:
                    root.activeInstalls[root.category + "/" + root.sourceName + "/" + tag] !== undefined

                width: ListView.view.width
                height: 64

                Rectangle {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space.sm
                    anchors.rightMargin: Theme.space.sm
                    anchors.topMargin: 3
                    anchors.bottomMargin: 3
                    radius: Theme.radius.sm
                    color: rowMouse.containsMouse
                        ? Theme.alpha(Theme.text, 0.05)
                        : "transparent"
                    Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
                }

                MouseArea {
                    id: rowMouse
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
                        text: versionRow.tag
                        color: Theme.text
                        font.pixelSize: Theme.type.label.size
                        font.weight: Font.Medium
                        font.family: "monospace"
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        text: {
                            var parts = []
                            if (versionRow.assetSize > 0) parts.push((versionRow.assetSize / (1024 * 1024)).toFixed(1) + " MB")
                            if (versionRow.publishedAt.length >= 10) parts.push(versionRow.publishedAt.substring(0, 10))
                            return parts.join("  ·  ")
                        }
                        color: Theme.textSubtle
                        font.pixelSize: Theme.type.caption.size
                        font.family: "monospace"
                        elide: Text.ElideRight
                    }
                }

                Row {
                    id: actionSlot
                    anchors.right: parent.right
                    anchors.rightMargin: 20
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 14

                    IconButton {
                        visible: versionRow.installed && root.category === "runners" && root.sourceKind === "proton"
                        anchors.verticalCenter: parent.verticalCenter
                        icon: "steam"
                        size: 28
                        rounded: true
                        onClicked: root.moveToSteamRequested(root.sourceName, versionRow.assetStems[versionRow.assetIndex])
                    }

                    M3Dropdown {
                        visible: versionRow.assets.length > 1
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.min(versionRow.width - 380, assetMetrics.width + 56)
                        fieldHeight: 30
                        options: versionRow.assetLabels.map((l, i) => ({ label: l, value: i }))
                        currentIndex: versionRow.assetIndex
                        onSelected: (v) => versionRow.assetIndex = v

                        TextMetrics {
                            id: assetMetrics
                            font.pixelSize: Theme.type.body.size
                            text: versionRow.assetLabels[versionRow.assetIndex] || ""
                        }
                    }

                    Item {
                        width: 132
                        height: 30
                        anchors.verticalCenter: parent.verticalCenter

                        M3Button {
                            anchors.centerIn: parent
                            visible: !versionRow.installed && !versionRow.busy
                            text: qsTr("Install")
                            variant: "filled"
                            onClicked: {
                                var payload = JSON.parse(JSON.stringify(versionRow.modelData))
                                if (versionRow.chosenAsset) {
                                    payload.asset_name = versionRow.chosenAsset.name
                                    payload.asset_url = versionRow.chosenAsset.url
                                    payload.asset_size = versionRow.chosenAsset.size
                                }
                                root.archiveManager.installVersion(
                                    root.category,
                                    root.sourceName,
                                    JSON.stringify(payload)
                                )
                            }
                        }

                        Row {
                            anchors.centerIn: parent
                            visible: versionRow.installed && !versionRow.busy
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
                                onClicked: {
                                    root.archiveManager.deleteVersion(root.category, root.sourceName, versionRow.assetStems[versionRow.assetIndex])
                                    root.refreshInstalled()
                                    root.versionDeleted(root.category, root.sourceName, versionRow.assetStems[versionRow.assetIndex])
                                }
                            }
                        }

                        Text {
                            anchors.centerIn: parent
                            visible: versionRow.busy
                            text: qsTr("Working…")
                            color: Theme.textMuted
                            font.pixelSize: Theme.type.caption.size
                        }
                    }
                }

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: Theme.separator
                    visible: versionRow.index < (list.count - 1)
                }
            }
        }
    }
}
