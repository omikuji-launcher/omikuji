import QtQuick
import QtQuick.Layouts
import "../lib/RunnerGrouping.js" as RG
import "../controls"
import "../dialogs"
import "../primitives"
import "../lib/Format.js" as Format

DialogCard {
    sizeKey: "gacha_install"
    id: root

    property var gameModel: null
    property var downloadModel: null

    property int runnersVersion: 0
    onRunnersVersionChanged: if (root.shown) loadRunners()

    property string manifestId: ""
    property var manifest: null

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
        runnerOptions = opts

        let prefs = (manifest && manifest.runner_preference) ? manifest.runner_preference : []
        let def = defaults ? (defaults.getConfig()["wine.version"] || "") : ""
        runnerIndex = RG.preferredIndex(opts, def, prefs)
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

    function refreshExisting() {
        if (!gameModel || !manifest || effectiveInstallPath === "") {
            existingTempBytes = 0; existingTempSegments = 0; existingInstall = false; existingVersion = ""; return
        }
        let raw = gameModel.gacha_check_existing_install(
            manifestId, editionId, effectiveInstallPath, tempPath.trim()
        )
        let p = {}
        try { p = JSON.parse(raw) || {} } catch (e) { p = {} }
        existingTempBytes = parseInt(p.bytes) || 0
        existingTempSegments = parseInt(p.segments) || 0
        existingInstall = p.has_install === true
        existingVersion = (p.installed_version && typeof p.installed_version === "string") ? p.installed_version : ""
    }

    body: ColumnLayout {
        width: parent.width
        spacing: 18

        RowLayout {
            Layout.fillWidth: true
            spacing: theme.space.md

            SvgIcon {
                name: "local_activity"
                size: 20
                color: theme.textMuted
                Layout.alignment: Qt.AlignVCenter
            }

            Text {
                Layout.fillWidth: true
                text: root.displayName ? qsTr("Install %1").arg(root.displayName) : qsTr("Install")
                color: theme.text
                font.pixelSize: 18
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: theme.space.sm
            visible: root.editions.length > 1

            Text {
                text: qsTr("Edition")
                color: theme.textMuted
                font.pixelSize: 13
                font.weight: Font.Medium
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: theme.space.sm

                Repeater {
                    model: root.editions

                    Item {
                        required property var modelData
                        required property int index

                        Layout.fillWidth: true
                        implicitHeight: 36

                        Rectangle {
                            anchors.fill: parent
                            radius: 18
                            color: index === root.editionIndex
                                ? theme.alpha(theme.accent, 0.15)
                                : edBtnHover.containsMouse
                                    ? theme.alpha(theme.text, 0.06)
                                    : "transparent"
                            border.width: index === root.editionIndex ? 1 : 0
                            border.color: theme.alpha(theme.accent, 0.3)

                            Behavior on color { ColorAnimation { duration: 100 } }
                        }

                        Text {
                            anchors.centerIn: parent
                            text: modelData.label
                            color: index === root.editionIndex ? theme.accent : theme.text
                            font.pixelSize: 13
                            font.weight: index === root.editionIndex ? Font.DemiBold : Font.Normal
                        }

                        MouseArea {
                            id: edBtnHover
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.editionIndex = index
                        }
                    }
                }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: theme.space.sm
            visible: root.voiceLocales.length > 0

            Text {
                text: qsTr("Voice Packs")
                color: theme.textMuted
                font.pixelSize: 13
                font.weight: Font.Medium
            }

            GridLayout {
                Layout.fillWidth: true
                columns: 2
                columnSpacing: theme.space.md
                rowSpacing: theme.space.sm

                Repeater {
                    model: root.voiceLocales

                    RowLayout {
                        required property var modelData
                        required property int index

                        Layout.fillWidth: true
                        spacing: theme.space.sm

                        M3Switch {
                            checked: root.voiceChecks[index] === true
                            onToggled: {
                                let copy = root.voiceChecks.slice()
                                copy[index] = !copy[index]
                                root.voiceChecks = copy
                            }
                        }

                        Text {
                            text: modelData.label
                            color: theme.text
                            font.pixelSize: 13
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
                trailingHint: root.installFolderName ? "/" + root.installFolderName : ""
                onTextEdited: (t) => root.installPath = t
                onAccepted: (p) => root.installPath = p
            }

            Text {
                text: {
                    let parts = []
                    if (root.downloadBytes === -2) {
                        parts.push(qsTr("Calculating size…"))
                    } else if (root.sizeError !== "") {
                        parts.push(qsTr("Size unavailable"))
                    } else if (root.installBytes >= 0) {
                        parts.push(qsTr("%1 install").arg(Format.formatBytesShort(root.installBytes)))
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
                    ? theme.accent
                    : (root.installBytes >= 0 && root.installFreeBytes >= 0
                        && root.installFreeBytes < root.installBytes
                        && !root.existingInstall
                        ? "#e06060" : theme.textFaint)
                font.pixelSize: 11
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                Layout.leftMargin: 4
                visible: text !== ""
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
                    ? theme.accent
                    : (root.downloadBytes >= 0 && root.tempFreeBytes >= 0
                        && root.tempFreeBytes < root.downloadBytes
                        ? "#e06060" : theme.textFaint)
                font.pixelSize: 11
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
                Layout.leftMargin: 4
                visible: text !== ""
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

    actions: Row {
        spacing: theme.space.sm

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
            onClicked: {
                let runner = root.runnerOptions.length > 0
                    ? root.runnerOptions[root.runnerIndex].value
                    : "system"
                if (root.existingInstall) {
                    let gid = root.gameModel.gacha_import_after_install(
                        root.manifestId,
                        root.editionId,
                        root.displayName,
                        root.effectiveInstallPath,
                        runner,
                        root.prefixPath
                    )
                    root.imported(gid || "")
                    root.close()
                    return
                }
                let id = root.downloadModel.enqueue_gacha(
                    root.manifestId,
                    root.editionId,
                    root.voicesSelected().join(","),
                    root.displayName,
                    root.effectiveInstallPath,
                    runner,
                    root.prefixPath,
                    root.tempPath
                )
                if (id && id.length > 0) {
                    root.installEnqueued(id)
                }
                root.close()
            }
        }
    }
}
