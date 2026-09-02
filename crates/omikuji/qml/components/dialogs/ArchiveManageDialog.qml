pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Controls
import "../controls"
import "../primitives"
import "../lib/ArchiveAssets.js" as AA
import "../lib/Format.js" as Format


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
    property bool fetching: false
    property string latestCategory: "runners_latest"

    readonly property string latestName: sourceName + "-Latest"
    readonly property bool latestInstalled: installedDirs[latestName] === true
    readonly property bool latestBusy: {
        const prefix = latestCategory + "/" + sourceName + "/"
        for (const key in activeInstalls)
            if (key.indexOf(prefix) === 0) return true
        return false
    }

    signal closed()
    signal versionDeleted(string category, string sourceName, string tag)
    signal removeSourceRequested(string category, string sourceName)
    signal editSourceRequested(string category, string sourceName)
    signal moveToSteamRequested(string sourceName, string tag)
    signal latestInstallRequested(string sourceName)

    maxWidth: 840
    preferredHeight: 650
    scrollable: false
    fillHeight: true
    title: ""

    property var sources: []
    property bool showSources: false

    function loadSources() {
        if (!archiveManager || !showSources) {
            sources = []
            return
        }
        try {
            let raw = category === "dll_packs"
                ? archiveManager.listDllPacks()
                : archiveManager.listRunners()
            sources = JSON.parse(raw) || []
        } catch (e) {
            sources = []
        }
    }

    function selectSource(name, kind) {
        sourceName = name
        sourceKind = kind
        versions = []
        installedDirs = ({})
        root.errorText = ""
        refreshInstalled()
        fetchVersionsNow()
    }

    function show(cat, name, kind, withSources) {
        category = cat
        showSources = withSources === true
        loadSources()
        selectSource(name, kind)
        open()
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

    function ownsInstall(cat, name) {
        return name === sourceName && (cat === category || cat === latestCategory)
    }

    function deleteInstalled(dirName) {
        archiveManager.deleteVersion(category, sourceName, dirName)
        refreshInstalled()
        root.versionDeleted(category, sourceName, dirName)
    }

    function fetchVersionsNow() {
        if (!archiveManager || sourceName === "") return
        fetching = true
        root.errorText = ""
        archiveManager.fetchVersions(category, sourceName)
    }

    onCloseRequested: { root.closed(); root.close() }

    footerLeft: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Remove source")
            variant: "tonal"
            danger: true
            onClicked: root.removeSourceRequested(root.category, root.sourceName)
        }

        IconButton {
            icon: "tune"
            size: 40
            anchors.verticalCenter: parent.verticalCenter
            onClicked: root.editSourceRequested(root.category, root.sourceName)
        }
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
                root.errorText = qsTr("Couldn't parse versions response.")
            }
        }
        function onVersionsFailed(cat, name, err) {
            if (cat !== root.category || name !== root.sourceName) return
            root.fetching = false
            root.errorText = err
        }
        function onInstallCompleted(cat, name, tag, installDir) {
            if (!root.ownsInstall(cat, name)) return
            root.refreshInstalled()
        }
        function onInstallFailed(cat, name, tag, err) {
            if (!root.ownsInstall(cat, name)) return
            root.errorText = err
        }
    }

    body: Item {
        width: parent.width
        height: parent.height

        Item {
            id: sourceList
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            visible: root.showSources
            width: root.showSources ? 196 : 0
            clip: true

            readonly property int rowHeight: 42
            readonly property int rowGap: 2
            readonly property int topPad: Theme.space.sm
            readonly property int currentIndex: {
                for (let i = 0; i < root.sources.length; i++) {
                    if (root.sources[i].name === root.sourceName) return i
                }
                return -1
            }

            Squircle {
                readonly property int inset: 3

                x: inset
                width: parent.width - inset * 2
                height: sourceList.rowHeight - inset * 2
                radius: Theme.radius.md
                fillColor: Theme.alpha(Theme.accent, 0.16)
                visible: sourceList.currentIndex >= 0
                y: sourceList.topPad
                    + sourceList.currentIndex * (sourceList.rowHeight + sourceList.rowGap)
                    + inset

                Behavior on y {
                    NumberAnimation {
                        duration: Theme.dur.med
                        easing.type: Theme.ease.emphasized
                        easing.overshoot: Theme.ease.overshoot
                    }
                }
            }

            Column {
                anchors.top: parent.top
                anchors.topMargin: sourceList.topPad
                anchors.left: parent.left
                anchors.right: parent.right
                spacing: sourceList.rowGap

                Repeater {
                    model: root.sources

                    delegate: Item {
                        id: sourceRow
                        required property var modelData

                        readonly property bool current: root.sourceName === sourceRow.modelData.name

                        width: parent.width
                        height: sourceList.rowHeight

                        Squircle {
                            anchors.fill: parent
                            anchors.margins: 3
                            radius: Theme.radius.md
                            fillColor: (!sourceRow.current && sourceHover.containsMouse)
                                ? Theme.alpha(Theme.text, 0.06)
                                : "transparent"
                        }

                        Text {
                            anchors.left: parent.left
                            anchors.leftMargin: Theme.space.md
                            anchors.right: parent.right
                            anchors.rightMargin: Theme.space.sm
                            anchors.verticalCenter: parent.verticalCenter
                            text: sourceRow.modelData.name
                            color: sourceRow.current ? Theme.accent : Theme.text
                            font.pixelSize: Theme.type.label.size
                            font.weight: sourceRow.current ? Font.DemiBold : Font.Normal
                            elide: Text.ElideRight

                            Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
                        }

                        MouseArea {
                            id: sourceHover
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (!sourceRow.current) {
                                    root.selectSource(sourceRow.modelData.name, sourceRow.modelData.kind)
                                }
                            }
                        }
                    }
                }
            }
        }

        Item {
            id: bodyHeader
            anchors.top: parent.top
            anchors.left: sourceList.right
            anchors.leftMargin: root.showSources ? Theme.space.lg : 0
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
                        : root.versions.length > 0 ? qsTr("%n version(s) available", "", root.versions.length)
                        : qsTr("No versions loaded yet")
                    color: Theme.textSubtle
                    font.pixelSize: Theme.type.caption.size
                }
            }
        }

        Rectangle {
            id: bodyDivider
            anchors.top: bodyHeader.bottom
            anchors.left: sourceList.right
            anchors.leftMargin: root.showSources ? Theme.space.lg : 0
            anchors.right: parent.right
            height: 1
            color: Theme.separator
        }

        ListView {
            id: list
            anchors.top: bodyDivider.bottom
            anchors.left: sourceList.right
            anchors.leftMargin: root.showSources ? Theme.space.lg : 0
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
                    : root.errorText !== "" ? qsTr("Installed runners are still listed under Found runners.")
                    : qsTr("No versions available.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.label.size
            }

            header: Item {
                id: latestRow
                width: list.width
                visible: root.category === "runners"
                height: visible ? 64 : 0

                Rectangle {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space.sm
                    anchors.rightMargin: Theme.space.sm
                    anchors.topMargin: 3
                    anchors.bottomMargin: 3
                    radius: Theme.radius.sm
                    color: Theme.alpha(Theme.accent, latestMouse.containsMouse ? 0.12 : 0.07)
                    Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
                }

                MouseArea {
                    id: latestMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.NoButton
                }

                Column {
                    anchors.left: parent.left
                    anchors.leftMargin: 24
                    anchors.right: latestActions.left
                    anchors.rightMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 3

                    Text {
                        width: parent.width
                        text: root.latestName
                        color: Theme.accent
                        font.pixelSize: Theme.type.label.size
                        font.weight: Font.DemiBold
                        font.family: "monospace"
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        text: qsTr("Always updates itself to the newest release")
                        color: Theme.textSubtle
                        font.pixelSize: Theme.type.caption.size
                        elide: Text.ElideRight
                    }
                }

                Row {
                    id: latestActions
                    anchors.right: parent.right
                    anchors.rightMargin: 20
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 14

                    IconButton {
                        visible: root.latestInstalled && root.sourceKind === "proton"
                        anchors.verticalCenter: parent.verticalCenter
                        icon: "steam"
                        size: 32
                        tonal: true
                        squircle: true
                        onClicked: root.moveToSteamRequested(root.sourceName, root.latestName)
                    }

                    InstallActions {
                        anchors.verticalCenter: parent.verticalCenter
                        installed: root.latestInstalled
                        busy: root.latestBusy
                        onInstallRequested: root.latestInstallRequested(root.sourceName)
                        onDeleteRequested: root.deleteInstalled(root.latestName)
                    }
                }

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: Theme.separator
                }
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
                            if (versionRow.assetSize > 0) parts.push(Format.formatBytes(versionRow.assetSize))
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
                        size: 32
                        tonal: true
                        squircle: true
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

                    InstallActions {
                        anchors.verticalCenter: parent.verticalCenter
                        installed: versionRow.installed
                        busy: versionRow.busy
                        onInstallRequested: {
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
                        onDeleteRequested: root.deleteInstalled(versionRow.assetStems[versionRow.assetIndex])
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
