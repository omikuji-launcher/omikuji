import QtQuick
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects
import "../lib/RunnerGrouping.js" as RG
import "../controls"
import "../dialogs"
import "../primitives"
import "../lib/Format.js" as Format

DialogCard {
    sizeKey: "epic_install"
    id: root

    property var gameModel: null
    property var epicModel: null

    property int runnersVersion: 0
    onRunnersVersionChanged: if (root.shown) loadRunners()

    property int gameIndex: -1

    signal installEnqueued(string downloadId)
    signal cancelled()

    property var gameData: null
    property string installPath: ""
    property string prefixPath: ""
    property var defaults: null
    property var runnerOptions: []
    property int runnerIndex: 0

    property real freeSpaceBytes: -1
    property real downloadBytes: -1
    property real installBytes: -1
    property string sizeError: ""
    property string _sizeRequestId: ""

    property var gameDetails: null
    property string _detailsRequestId: ""
    property bool detailsExpanded: false
    readonly property bool _hasDesc: gameDetails !== null && !!gameDetails.description
    readonly property bool _hasReqs: gameDetails !== null && !!gameDetails.reqs && gameDetails.reqs.length > 0

    property real existingInstallBytes: 0
    property bool hasResumeState: false

    readonly property bool isImportMode: gameData && gameData.isInstalled === true
    readonly property bool hasUntrackedInstall: !isImportMode && !hasResumeState && existingInstallBytes > 1048576

    readonly property string effectiveInstallPath: {
        if (!gameData) return ""
        if (isImportMode) return gameData.installPath || ""
        let safe = (gameData.title || "Game").replace(/[\\/:*?"<>|]/g, "").trim()
        let base = (installPath || "").trim().replace(/\/+$/, "")
        if (base === "" || safe === "") return ""
        return base + "/" + safe
    }

    maxWidth: 480
    panelsShown: detailsExpanded
    leftPanel: _hasDesc ? detailsAboutPanel : null
    rightPanel: _hasReqs ? detailsReqsPanel : null

    function hasEnoughSpace() {
        if (freeSpaceBytes < 0) return false
        if (installBytes < 0) return false
        if (existingInstallBytes > 0 || hasResumeState) return true
        return freeSpaceBytes >= installBytes
    }

    function refreshFreeSpace() {
        if (!gameModel || installPath.trim() === "") { freeSpaceBytes = -1; return }
        let raw = gameModel.disk_free_space(installPath.trim())
        freeSpaceBytes = parseInt(raw)
        if (isNaN(freeSpaceBytes)) freeSpaceBytes = -1
    }

    function refreshExistingInstall() {
        if (!gameModel || !gameData || !gameData.appName || effectiveInstallPath === "") {
            existingInstallBytes = 0; hasResumeState = false; return
        }
        let raw = gameModel.epic_check_existing_install(gameData.appName, effectiveInstallPath)
        let p = {}
        try { p = JSON.parse(raw) || {} } catch (e) { p = {} }
        existingInstallBytes = parseInt(p.bytes) || 0
        hasResumeState = p.hasResume === true
    }

    function refreshInstallSize() {
        if (!gameModel || !gameData || !gameData.appName) {
            downloadBytes = -1; installBytes = -1; sizeError = ""; return
        }
        downloadBytes = -2
        installBytes = -2
        sizeError = ""
        let id = "epic-" + Date.now().toString(36) + "-" + Math.random().toString(36).substring(2, 8)
        _sizeRequestId = id
        gameModel.fetch_epic_install_size(id, gameData.appName)
    }

    Connections {
        target: root.gameModel
        function onInstall_size_result(requestId, payload) {
            if (requestId !== root._sizeRequestId) return
            root._sizeRequestId = ""
            let p = {}
            try { p = JSON.parse(payload) || {} } catch (e) { p = {} }
            if (p.error && p.error.length > 0) {
                root.sizeError = p.error
                root.downloadBytes = -1
                root.installBytes = -1
            } else {
                root.sizeError = ""
                root.downloadBytes = parseInt(p.download) || 0
                root.installBytes = parseInt(p.install) || 0
            }
        }
        function onGame_details_result(requestId, payload) {
            if (requestId !== root._detailsRequestId) return
            root._detailsRequestId = ""
            try { root.gameDetails = JSON.parse(payload) } catch (e) { root.gameDetails = null }
        }
    }

    onInstallPathChanged: refreshFreeSpace()
    onEffectiveInstallPathChanged: refreshExistingInstall()

    function resetState() {
        gameData = null
        installPath = ""
        prefixPath = ""
        runnerOptions = []
        runnerIndex = 0
        freeSpaceBytes = -1
        downloadBytes = -1
        installBytes = -1
        sizeError = ""
        _sizeRequestId = ""
        existingInstallBytes = 0
        hasResumeState = false
        gameDetails = null
        _detailsRequestId = ""
        detailsExpanded = false
    }

    function show() {
        if (!epicModel || gameIndex < 0) return
        resetState()
        gameData = epicModel.get_game_at(gameIndex)
        if (gameData && gameData.appName) {
            let did = "epicd-" + Date.now().toString(36) + "-" + Math.random().toString(36).substring(2, 8)
            _detailsRequestId = did
            gameModel.fetch_epic_game_details(did, gameData.appName)
        }
        let legendaryGameDir = gameData && gameData.installPath ? gameData.installPath : ""
        if (gameData && gameData.isInstalled === true && legendaryGameDir !== "") {
            let slash = legendaryGameDir.lastIndexOf("/")
            installPath = slash > 0 ? legendaryGameDir.substring(0, slash) : legendaryGameDir
        } else {
            installPath = defaultInstallPath(gameData ? gameData.title : "")
        }
        if (defaults) prefixPath = defaults.getConfig()["wine.prefix"] || ""
        loadRunners()
        refreshFreeSpace()
        refreshInstallSize()
        refreshExistingInstall()
        open()
        forceActiveFocus()
    }

    function hide() { close() }

    onVisibleChanged: if (!visible) { gameIndex = -1; resetState() }

    onCloseRequested: { root.cancelled(); root.close() }

    function defaultInstallPath(title) {
        if (!gameModel) return ""
        let home = gameModel.home_dir()
        if (!home) return ""
        return home + "/Games"
    }

    function loadRunners() {
        if (!gameModel) return
        let raw = gameModel.list_runners()
        let arr = []
        try { arr = JSON.parse(raw) || [] } catch (e) { arr = [] }
        let opts = RG.groupRunners(arr)
        if (opts.length === 0) opts = [{ label: "System Wine", value: "system" }]
        runnerOptions = opts

        let def = defaults ? (defaults.getConfig()["wine.version"] || "") : ""
        runnerIndex = RG.preferredIndex(opts, def, ["GE-Proton", "Proton-GE", "wine-ge"])
    }

    body: ColumnLayout {
        width: parent.width
        spacing: 18

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.space.md

            SvgIcon {
                name: "shield_moon"
                size: 20
                color: theme.textMuted
                Layout.alignment: Qt.AlignVCenter
            }

            Text {
                Layout.fillWidth: true
                text: root.gameData ? qsTr("Install %1").arg(root.gameData.title) : qsTr("Install")
                color: theme.text
                font.pixelSize: theme.type.headline.size
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 96
            radius: theme.radius.md
            color: theme.alpha(theme.text, 0.04)
            visible: bannerImg.source != ""

            Image {
                id: bannerImg
                anchors.fill: parent
                source: root.gameData ? (root.gameData.banner || root.gameData.coverart || "") : ""
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
                sourceSize.width: 800
                sourceSize.height: 240
                visible: status === Image.Ready
                layer.enabled: true
                layer.effect: OpacityMask {
                    maskSource: Rectangle {
                        width: bannerImg.width
                        height: bannerImg.height
                        radius: theme.radius.md
                    }
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4

            M3FileField {
                Layout.fillWidth: true
                label: qsTr("Installation path")
                placeholder: "/home/you/Games"
                selectFolder: true
                gameModel: root.gameModel
                text: root.installPath
                readOnly: root.isImportMode
                trailingHint: root.isImportMode || !root.gameData || !root.gameData.title
                    ? ""
                    : "/" + (root.gameData.title || "").replace(/[\\/:*?"<>|]/g, "").trim()
                onTextEdited: (t) => root.installPath = t
                onAccepted: (p) => root.installPath = p
            }

            Text {
                text: {
                    let parts = []
                    if (root.existingInstallBytes > 0 || root.hasResumeState) {
                        let label = qsTr("Found existing files")
                        if (root.existingInstallBytes > 0) label += " · " + Format.formatBytesShort(root.existingInstallBytes)
                        if (root.hasResumeState) label += " · " + qsTr("resume state")
                        parts.push(label)
                    }
                    if (root.downloadBytes === -2) {
                        parts.push(qsTr("Calculating size…"))
                    } else if (root.sizeError !== "") {
                        parts.push(qsTr("Size unavailable"))
                    } else if (root.installBytes >= 0) {
                        parts.push(qsTr("%1 install").arg(Format.formatBytesShort(root.installBytes)))
                        if (root.downloadBytes > 0) {
                            parts.push(qsTr("%1 download").arg(Format.formatBytesShort(root.downloadBytes)))
                        }
                    }
                    if (root.freeSpaceBytes >= 0) {
                        parts.push(qsTr("%1 free").arg(Format.formatBytesShort(root.freeSpaceBytes)))
                    }
                    return parts.join(" · ")
                }
                color: (root.existingInstallBytes > 0 || root.hasResumeState)
                    ? theme.accent
                    : (root.hasEnoughSpace() ? theme.textFaint : "#e06060")
                font.pixelSize: theme.type.micro.size
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                Layout.leftMargin: 4
                visible: text !== ""
            }

            Text {
                text: qsTr("Epic Games allows only one install. To reinstall elsewhere, delete the game files first.")
                color: theme.accent
                font.pixelSize: theme.type.micro.size
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                Layout.leftMargin: 4
                visible: root.isImportMode
            }
        }

        M3FileField {
            Layout.fillWidth: true
            label: qsTr("Prefix path (optional)")
            placeholder: qsTr("auto — created per game")
            selectFolder: true
            gameModel: root.gameModel
            text: root.prefixPath
            onTextEdited: (t) => root.prefixPath = t
            onAccepted: (p) => root.prefixPath = p
        }

        M3Dropdown {
            Layout.fillWidth: true
            label: qsTr("Runner")
            options: root.runnerOptions
            currentIndex: root.runnerIndex
            onSelected: (v) => {
                for (let i = 0; i < root.runnerOptions.length; i++) {
                    if (root.runnerOptions[i].value === v) { root.runnerIndex = i; break }
                }
            }
        }
    }

    Component {
        id: detailsAboutPanel
        StoreGameDetails { kind: "about"; details: root.gameDetails }
    }

    Component {
        id: detailsReqsPanel
        StoreGameDetails { kind: "reqs"; details: root.gameDetails }
    }

    footerLeft: M3Button {
        visible: root._hasDesc || root._hasReqs
        variant: "text"
        text: root.detailsExpanded ? qsTr("Hide details") : qsTr("Show details")
        onClicked: root.detailsExpanded = !root.detailsExpanded
    }

    actions: Row {
        spacing: theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "text"
            onClicked: { root.cancelled(); root.close() }
        }
        M3Button {
            text: {
                if (root.gameData && root.gameData.isInstalled) {
                    return root.gameData.hasLibraryEntry ? qsTr("Repair") : qsTr("Import")
                }
                if (root.hasResumeState) return qsTr("Resume")
                if (root.hasUntrackedInstall) return qsTr("Import")
                return qsTr("Install")
            }
            variant: "filled"
            enabled: root.installPath.trim().length > 0 && root.hasEnoughSpace()
            onClicked: {
                let runner = root.runnerOptions.length > 0
                    ? root.runnerOptions[root.runnerIndex].value
                    : ""
                let id = root.epicModel.enqueue_install(
                    root.gameIndex,
                    root.effectiveInstallPath,
                    root.prefixPath,
                    runner,
                    root.isImportMode,
                    root.hasUntrackedInstall
                )
                if (id && id.length > 0) {
                    root.installEnqueued(id)
                }
                root.close()
            }
        }
    }
}
