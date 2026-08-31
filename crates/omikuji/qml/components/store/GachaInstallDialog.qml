pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts
import "../lib/RunnerGrouping.js" as RG
import "../lib/ArchiveAssets.js" as AA
import "../controls"
import "../dialogs"
import "../primitives"
import "../lib/Format.js" as Format

DialogCard {
    sizeKey: "gacha_install"
    id: root

    property var gameModel: null
    property var downloadModel: null
    property var archiveManager: null

    property var advised: null
    property int assetIndex: 0
    property bool runnerInstalling: false
    property string runnerPhase: ""
    property real runnerPercent: 0
    property string runnerError: ""

    readonly property bool hasAdvised: advised !== null && advised !== undefined
    readonly property var advisedAssets: hasAdvised ? advised.assets : []
    readonly property var advisedStems: AA.stems(advisedAssets)
    readonly property var advisedLabels: AA.labels(advisedAssets)
    readonly property string chosenStem: advisedStems[assetIndex] || ""
    readonly property bool advisedInstalled:
        hasAdvised && chosenStem !== "" && advised.installedDirs.indexOf(chosenStem) >= 0
    readonly property bool advisedSelected:
        hasAdvised && runnerOptions.length > 0
        && runnerOptions[runnerIndex] && runnerOptions[runnerIndex].isAdvisedEntry === true
    readonly property color advisedTint: advisedInstalled ? Theme.success : Theme.warning

    property int runnersVersion: 0
    onRunnersVersionChanged: if (root.shown) loadRunners()

    property string manifestId: ""
    property var manifest: null

    readonly property var companion: manifest && manifest.alongside ? manifest.alongside : null
    property bool companionAccepted: false

    signal installEnqueued(string downloadId)
    signal imported(string gameId)
    signal cancelled()

    property int editionIndex: 0
    property string installPath: ""
    property string prefixPath: ""
    property string tempPath: ""
    property var defaults: null
    property var runnerOptions: []
    property int runnerIndex: 0

    property var voiceChecks: []

    property real installFreeBytes: -1
    property real tempFreeBytes: -1
    property real downloadBytes: -1
    property real installBytes: -1
    property string sizeError: ""
    property string _sizeRequestId: ""

    property real existingTempBytes: 0
    property int existingTempSegments: 0
    property bool existingInstall: false
    property string existingVersion: ""
    property string detectedEdition: ""
    property string importDir: ""
    readonly property bool directImport: existingInstall && importDir !== "" && importDir === (installPath || "").trim()

    readonly property string displayName: root.manifest ? root.manifest.display_name : ""
    readonly property string installFolderName:
        root.manifest ? (root.manifest.install_folder_name || "") : ""
    readonly property var editions:
        root.manifest && root.manifest.editions ? root.manifest.editions : []
    readonly property var voiceLocales:
        root.manifest && root.manifest.voice_locales ? root.manifest.voice_locales : []
    readonly property bool usesTempDir:
        root.manifest ? (root.manifest.uses_temp_dir !== false) : true

    readonly property string editionId: {
        if (!editions.length) return ""
        let idx = Math.max(0, Math.min(editionIndex, editions.length - 1))
        return editions[idx].id || ""
    }
    readonly property string appIdPrefix:
        root.manifest ? (root.manifest.app_id_prefix || "") : ""
    readonly property string appId: {
        if (appIdPrefix === "" || editionId === "") return ""
        return appIdPrefix + ":" + editionId
    }

    readonly property string effectiveInstallPath: {
        let folder = (installFolderName || "").replace(/[\\/:*?"<>|]/g, "").trim()
        let base = (installPath || "").trim().replace(/\/+$/, "")
        if (base === "" || folder === "") return ""
        return base + "/" + folder
    }

    maxWidth: 480

    function effectiveTempPath() {
        return tempPath.trim() !== "" ? tempPath.trim() : installPath.trim()
    }

    function voicesSelected() {
        let out = []
        for (let i = 0; i < voiceLocales.length; i++) {
            if (voiceChecks[i]) out.push(voiceLocales[i].id)
        }
        return out
    }

    function hasEnoughSpace() {
        if (existingInstall) return true
        if (installFreeBytes < 0) return false
        if (downloadBytes < 0 || installBytes < 0) return false
        if (!usesTempDir) {
            let resuming = existingInstall
            if (!resuming && installFreeBytes < installBytes) return false
            return true
        }
        if (tempFreeBytes < 0) return false
        let tempNeeded = Math.max(0, downloadBytes - existingTempBytes)
        if (tempFreeBytes < tempNeeded) return false
        let resuming = existingTempSegments > 0 || existingInstall
        if (!resuming && installFreeBytes < installBytes) return false
        return true
    }

    onInstallPathChanged: { refreshFreeSpace(); refreshExisting() }
    onEffectiveInstallPathChanged: refreshExisting()
    onTempPathChanged: { refreshFreeSpace(); refreshExisting() }
    onEditionIndexChanged: {
        if (root.shown) sizeFetchDebounce.restart()
        refreshExisting()
    }
    onVoiceChecksChanged: if (root.shown) sizeFetchDebounce.restart()

    Timer {
        id: sizeFetchDebounce
        interval: 200
        repeat: false
        onTriggered: root.refreshInstallSize()
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
    }

    function resetState() {
        manifest = null
        editionIndex = 0
        installPath = ""
        prefixPath = ""
        tempPath = ""
        runnerOptions = []
        runnerIndex = 0
        voiceChecks = []
        installFreeBytes = -1
        tempFreeBytes = -1
        downloadBytes = -1
        installBytes = -1
        sizeError = ""
        _sizeRequestId = ""
        existingTempBytes = 0
        existingTempSegments = 0
        existingInstall = false
        existingVersion = ""
        detectedEdition = ""
        importDir = ""
        advised = null
        assetIndex = 0
        runnerInstalling = false
        runnerPhase = ""
        runnerPercent = 0
        runnerError = ""
        companionAccepted = false
        sizeFetchDebounce.stop()
    }

    function show() {
        if (!gameModel || manifestId === "") return
        let raw = gameModel.get_gacha_manifest(manifestId)
        if (!raw || raw.length === 0) {
            console.warn("[GachaInstallDialog] unknown manifest:", manifestId)
            return
        }
        let m = null
        try { m = JSON.parse(raw) } catch (e) { m = null }
        if (!m) {
            console.warn("[GachaInstallDialog] failed to parse manifest:", manifestId)
            return
        }

        resetState()
        manifest = m

        let vs = []
        for (let i = 0; i < voiceLocales.length; i++) vs.push(i === 0)
        voiceChecks = vs

        installPath = defaultInstallPath()
        if (defaults) prefixPath = defaults.getConfig()["wine.prefix"] || ""
        loadRunners()
        loadAdvised()
        refreshFreeSpace()
        refreshInstallSize()
        refreshExisting()
        open()
        forceActiveFocus()
    }

    function hide() { close() }

    onVisibleChanged: if (!visible) { manifestId = ""; resetState() }

    onCloseRequested: { root.cancelled(); root.close() }

    function defaultInstallPath() {
        if (!gameModel || !manifest) return ""
        let home = gameModel.home_dir() || ""
        let tpl = manifest.default_library_template || "{home}/Games"
        return tpl.replace("{home}", home)
    }

    function loadRunners() {
        if (!gameModel) return
        let raw = gameModel.list_runners()
        let arr = []
        try { arr = JSON.parse(raw) || [] } catch (e) { arr = [] }
        let opts = RG.groupRunners(arr)
        if (opts.length === 0) opts = [{ label: "System Wine", value: "system" }]

        if (hasAdvised) {
            opts = [{
                label: advised.tag,
                value: "__advised__",
                isAdvisedEntry: true,
                tint: advisedTint
            }].concat(opts)
            runnerOptions = opts
            runnerIndex = 0
            return
        }

        runnerOptions = opts
        let def = defaults ? (defaults.getConfig()["wine.version"] || "") : ""
        runnerIndex = RG.preferredIndex(opts, def, [])
    }

    function loadAdvised() {
        advised = null
        assetIndex = 0
        runnerError = ""
        let link = manifest ? (manifest.runner || "") : ""
        if (!archiveManager || link === "") return
        archiveManager.fetchAdvisedRunner(link)
    }

    function selectedRunner() {
        if (advisedSelected && chosenStem !== "") return chosenStem
        return runnerOptions.length > 0 ? runnerOptions[runnerIndex].value : "system"
    }

    function startInstall() {
        if (advisedSelected && !advisedInstalled) {
            runnerError = ""
            runnerPhase = ""
            runnerPercent = 0
            runnerInstalling = true
            archiveManager.installAdvisedRunner(
                manifest.runner, advisedAssets[assetIndex].name
            )
            return
        }
        commitInstall()
    }

    function commitInstall() {
        let runner = selectedRunner()
        let importing = existingInstall
        if (importing && !downloadModel.gacha_supports_import(manifestId)) {
            let gid = gameModel.gacha_import_after_install(
                manifestId, editionId, displayName, importDir, runner, prefixPath,
                companionAccepted
            )
            imported(gid || "")
            close()
            return
        }
        let id = downloadModel.enqueue_gacha(
            manifestId, editionId, voicesSelected().join(","),
            displayName, importing ? importDir : effectiveInstallPath,
            runner, prefixPath, tempPath, importing, companionAccepted
        )
        if (id && id.length > 0) installEnqueued(id)
        close()
    }

    Connections {
        target: root.archiveManager

        function onAdvisedRunnerReady(json, error) {
            if (error && error.length > 0) {
                root.advised = null
                root.runnerError = error
                return
            }
            try { root.advised = JSON.parse(json) } catch (e) { root.advised = null }
            root.assetIndex = 0
            root.loadRunners()
        }
    }

    Connections {
        target: root.archiveManager
        enabled: root.runnerInstalling

        function onInstallProgress(category, source, tag, phase, percent) {
            if (!root.hasAdvised || tag !== root.advised.tag) return
            root.runnerPhase = phase
            root.runnerPercent = percent
        }
        function onInstallCompleted(category, source, tag, dir) {
            if (!root.hasAdvised || tag !== root.advised.tag) return
            root.runnerInstalling = false
            root.commitInstall()
        }
        function onInstallFailed(category, source, tag, error) {
            if (!root.hasAdvised || tag !== root.advised.tag) return
            root.runnerInstalling = false
            root.runnerError = error
        }
    }

    function refreshFreeSpace() {
        if (!gameModel || installPath.trim() === "") {
            installFreeBytes = -1; tempFreeBytes = -1; return
        }
        let rawInstall = gameModel.disk_free_space(installPath.trim())
        installFreeBytes = parseInt(rawInstall)
        if (isNaN(installFreeBytes)) installFreeBytes = -1

        let tp = effectiveTempPath()
        if (tp === installPath.trim()) {
            tempFreeBytes = installFreeBytes
        } else {
            let rawTemp = gameModel.disk_free_space(tp)
            tempFreeBytes = parseInt(rawTemp)
            if (isNaN(tempFreeBytes)) tempFreeBytes = -1
        }
    }

    function refreshInstallSize() {
        if (!gameModel || !manifest) {
            downloadBytes = -1; installBytes = -1; sizeError = ""; return
        }
        downloadBytes = -2
        installBytes = -2
        sizeError = ""
        let id = "gacha-" + Date.now().toString(36) + "-" + Math.random().toString(36).substring(2, 8)
        _sizeRequestId = id
        gameModel.fetch_gacha_install_size(id, manifestId, editionId, voicesSelected().join(","))
    }

    function _adoptDetectedEdition(dir) {
        detectedEdition = gameModel.gacha_detect_edition(manifestId, dir) || ""
        if (detectedEdition === "") return
        let idx = editions.findIndex(e => e.id === detectedEdition)
        if (idx >= 0 && idx !== editionIndex) editionIndex = idx
    }

    function _inspect(path, temp) {
        let out = {}
        try { out = JSON.parse(gameModel.gacha_check_existing_install(manifestId, editionId, path, temp) || "{}") || {} }
        catch (e) { out = {} }
        return out
    }

    function refreshExisting() {
        if (!gameModel || !manifest) {
            existingTempBytes = 0; existingTempSegments = 0; existingInstall = false; existingVersion = ""; importDir = ""; return
        }
        let rawPath = (installPath || "").trim()
        if (rawPath !== "") {
            let direct = _inspect(rawPath, "")
            if (direct.has_install === true) {
                existingTempBytes = 0; existingTempSegments = 0
                existingInstall = true
                existingVersion = (typeof direct.installed_version === "string") ? direct.installed_version : ""
                importDir = rawPath
                _adoptDetectedEdition(rawPath)
                return
            }
        }
        if (effectiveInstallPath === "") {
            existingTempBytes = 0; existingTempSegments = 0; existingInstall = false; existingVersion = ""; importDir = ""; return
        }
        let nested = _inspect(effectiveInstallPath, tempPath.trim())
        existingTempBytes = parseInt(nested.bytes) || 0
        existingTempSegments = parseInt(nested.segments) || 0
        existingInstall = nested.has_install === true
        existingVersion = (typeof nested.installed_version === "string") ? nested.installed_version : ""
        importDir = existingInstall ? effectiveInstallPath : ""
        if (existingInstall) _adoptDetectedEdition(importDir)
        else detectedEdition = ""
    }

    body: ColumnLayout {
        width: parent.width
        spacing: Theme.space.lg

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space.md

            SvgIcon {
                name: "local_activity"
                size: 20
                color: Theme.textMuted
                Layout.alignment: Qt.AlignVCenter
            }

            Text {
                Layout.fillWidth: true
                text: root.displayName ? qsTr("Install %1").arg(root.displayName) : qsTr("Install")
                color: Theme.text
                font.pixelSize: Theme.type.headline.size
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }
        }

        DialogSection {
            Layout.fillWidth: true
            label: qsTr("Edition")
            visible: root.editions.length > 1

            RowLayout {
                width: parent.width
                spacing: Theme.space.sm

                Repeater {
                    model: root.editions

                    Item {
                        id: editionRow
                        required property var modelData
                        required property int index

                        Layout.fillWidth: true
                        implicitHeight: 36

                        Rectangle {
                            anchors.fill: parent
                            radius: Theme.radius.pill
                            color: editionRow.index === root.editionIndex
                                ? Theme.alpha(Theme.accent, 0.15)
                                : edBtnHover.containsMouse
                                    ? Theme.alpha(Theme.text, 0.06)
                                    : "transparent"
                            border.width: editionRow.index === root.editionIndex ? 1 : 0
                            border.color: Theme.alpha(Theme.accent, 0.3)

                            Behavior on color { ColorAnimation { duration: 100 } }
                        }

                        Text {
                            anchors.centerIn: parent
                            text: editionRow.modelData.label
                            color: editionRow.index === root.editionIndex ? Theme.accent : Theme.text
                            font.pixelSize: Theme.type.label.size
                            font.weight: editionRow.index === root.editionIndex ? Font.DemiBold : Font.Normal
                        }

                        MouseArea {
                            id: edBtnHover
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.editionIndex = editionRow.index
                        }
                    }
                }
            }
        }

        DialogSection {
            Layout.fillWidth: true
            label: qsTr("Voice Packs")
            visible: root.voiceLocales.length > 0

            GridLayout {
                width: parent.width
                columns: 2
                columnSpacing: Theme.space.md
                rowSpacing: Theme.space.sm

                Repeater {
                    model: root.voiceLocales

                    RowLayout {
                        id: localeRow
                        required property var modelData
                        required property int index

                        Layout.fillWidth: true
                        spacing: Theme.space.sm

                        M3Switch {
                            checked: root.voiceChecks[localeRow.index] === true
                            onToggled: {
                                let copy = root.voiceChecks.slice()
                                copy[localeRow.index] = !copy[localeRow.index]
                                root.voiceChecks = copy
                            }
                        }

                        Text {
                            text: localeRow.modelData.label
                            color: Theme.text
                            font.pixelSize: Theme.type.label.size
                            Layout.fillWidth: true
                        }
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
                placeholder: root.defaultInstallPath()
                selectFolder: true
                gameModel: root.gameModel
                text: root.installPath
                trailingHint: root.directImport || !root.installFolderName ? "" : "/" + root.installFolderName
                onTextEdited: (t) => root.installPath = t
                onAccepted: (p) => root.installPath = p
            }

            Text {
                text: {
                    let parts = []
                    if (!root.existingInstall) {
                        if (root.downloadBytes === -2) {
                            parts.push(qsTr("Calculating size…"))
                        } else if (root.sizeError !== "") {
                            parts.push(qsTr("Size unavailable"))
                        } else if (root.installBytes >= 0) {
                            parts.push(qsTr("%1 install").arg(Format.formatBytesShort(root.installBytes)))
                        }
                    }
                    if (root.installFreeBytes >= 0) {
                        parts.push(qsTr("%1 free").arg(Format.formatBytesShort(root.installFreeBytes)))
                    }
                    if (root.existingInstall) {
                        if (root.existingVersion !== "") {
                            parts.push(qsTr("existing install detected · v%1").arg(root.existingVersion))
                        } else {
                            parts.push(qsTr("Unknown version"))
                        }
                    }
                    return parts.join(" · ")
                }
                color: root.existingInstall
                    ? Theme.accent
                    : (root.installBytes >= 0 && root.installFreeBytes >= 0
                        && root.installFreeBytes < root.installBytes
                        && !root.existingInstall
                        ? "#e06060" : Theme.textFaint)
                font.pixelSize: Theme.type.micro.size
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                Layout.leftMargin: 4
                visible: text !== ""
            }

            NoteChip {
                Layout.fillWidth: true
                Layout.topMargin: 4
                visible: root.existingInstall && root.editions.length > 1
                icon: root.detectedEdition !== "" ? "info" : "warning"
                tone: root.detectedEdition !== "" ? Theme.accent : Theme.warning
                text: {
                    if (root.detectedEdition === "") {
                        return qsTr("Select the region this folder holds. Importing the wrong one will not match these files.")
                    }
                    let hit = root.editions.find(e => e.id === root.detectedEdition)
                    return qsTr("This folder holds the %1 version, and it has been selected for you, baby.")
                        .arg(hit ? hit.label : root.detectedEdition)
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            visible: root.usesTempDir

            M3FileField {
                Layout.fillWidth: true
                label: qsTr("Temp path (optional)")
                placeholder: qsTr("auto — next to install path")
                selectFolder: true
                gameModel: root.gameModel
                text: root.tempPath
                onTextEdited: (t) => root.tempPath = t
                onAccepted: (p) => root.tempPath = p
            }

            Text {
                text: {
                    let parts = []
                    if (root.existingTempSegments > 0) {
                        parts.push(qsTr("Found existing files · %1").arg(Format.formatBytesShort(root.existingTempBytes)))
                    }
                    if (root.downloadBytes >= 0) {
                        parts.push(qsTr("%1 download").arg(Format.formatBytesShort(root.downloadBytes)))
                    }
                    if (root.tempFreeBytes >= 0) {
                        parts.push(qsTr("%1 free").arg(Format.formatBytesShort(root.tempFreeBytes)))
                    }
                    return parts.join(" · ")
                }
                color: root.existingTempSegments > 0
                    ? Theme.accent
                    : (root.downloadBytes >= 0 && root.tempFreeBytes >= 0
                        && root.tempFreeBytes < root.downloadBytes
                        ? "#e06060" : Theme.textFaint)
                font.pixelSize: Theme.type.micro.size
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                Layout.leftMargin: 4
                visible: text !== ""
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: Theme.space.md

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
                labelSuffix: root.advisedSelected
                    ? (root.advisedInstalled
                        ? qsTr("This runner is adviced to play this game, and is installed.")
                        : qsTr("This runner is adviced to play this game."))
                    : ""
                labelSuffixColor: root.advisedTint
                enabled: !root.runnerInstalling
                options: root.runnerOptions
                currentIndex: root.runnerIndex
                onSelected: (v) => {
                    for (let i = 0; i < root.runnerOptions.length; i++) {
                        if (root.runnerOptions[i].value === v) { root.runnerIndex = i; break }
                    }
                }
            }

            M3Dropdown {
                Layout.fillWidth: true
                visible: root.advisedSelected && root.advisedAssets.length > 1
                label: qsTr("Release file")
                enabled: !root.runnerInstalling
                options: root.advisedLabels.map((l, i) => ({ label: l, value: i }))
                currentIndex: root.assetIndex
                onSelected: (v) => root.assetIndex = v
            }

            ColumnLayout {
                Layout.fillWidth: true
                visible: root.runnerInstalling || root.runnerError !== ""
                spacing: Theme.space.xs

                Text {
                    Layout.fillWidth: true
                    text: root.runnerError !== ""
                        ? root.runnerError
                        : qsTr("Installing the runner... %1").arg(root.runnerPhase)
                    color: root.runnerError !== "" ? Theme.error : Theme.textMuted
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.WordWrap
                }

                WavyProgressBar {
                    Layout.fillWidth: true
                    visible: root.runnerInstalling
                    value: root.runnerPercent / 100
                    fillColor: Theme.accent
                    trackColor: Theme.alpha(Theme.text, 0.16)
                }
            }
        }

        DialogSection {
            Layout.fillWidth: true
            label: qsTr("Misc")
            visible: root.companion !== null

            Item {
                width: parent.width
                height: 40

                Rectangle {
                    anchors.fill: parent
                    radius: Theme.radius.sm
                    color: companionHover.containsMouse
                        ? Theme.alpha(Theme.text, 0.06)
                        : "transparent"
                    Behavior on color { ColorAnimation { duration: 100 } }
                }

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: 10
                    anchors.right: companionBox.left
                    anchors.rightMargin: Theme.space.md
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.companion ? (root.companion.label || root.companion.name) : ""
                    color: Theme.text
                    font.pixelSize: Theme.type.label.size
                    elide: Text.ElideRight
                }

                M3Checkbox {
                    id: companionBox
                    anchors.right: parent.right
                    anchors.rightMargin: 10
                    anchors.verticalCenter: parent.verticalCenter
                    checked: root.companionAccepted
                }

                MouseArea {
                    id: companionHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.companionAccepted = !root.companionAccepted
                }
            }
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "text"
            onClicked: { root.cancelled(); root.close() }
        }
        M3Button {
            text: root.existingInstall
                ? qsTr("Import")
                : (root.existingTempSegments > 0 ? qsTr("Resume") : qsTr("Install"))
            variant: "filled"
            enabled: root.manifest !== null
                && root.installPath.trim().length > 0
                && root.hasEnoughSpace()
                && !root.runnerInstalling
            onClicked: root.startInstall()
        }
    }
}
