import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects

import "../widgets"

Item {
    id: root

    property var gameModel: null
    property var downloadModel: null

    // bumped from Main on runner install so the dropdown reloads witout a close+reopen
    property int runnersVersion: 0
    onRunnersVersionChanged: if (opened) loadRunners()

    property string manifestId: ""
    property var manifest: null

    signal installEnqueued(string downloadId)
    signal imported(string gameId)
    signal cancelled()

    property bool opened: false
    visible: opacity > 0.001
    opacity: opened ? 1.0 : 0.0
    Behavior on opacity {
        NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
    }
    z: 1000

    property int editionIndex: 0
    property string installPath: ""
    property string prefixPath: ""
    property string tempPath: ""
    property var defaults: null
    property var runnerOptions: []
    property int runnerIndex: 0

    // one bool per manifest.voice_locales entry. defaults to [true, false, …]
    property var voiceChecks: []

    // -1 = not checked yet, >=0 = actual bytes free.
    property real installFreeBytes: -1
    property real tempFreeBytes: -1
    // -1 = not fetched, -2 = fetching, >=0 = actual value
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
    // strategies that stream directly into install_path set uses_temp_dir false, hides the temp field and simplifies the space check
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

    function formatBytes(bytes) {
        if (bytes <= 0) return ""
        let gb = bytes / (1024 * 1024 * 1024)
        if (gb >= 1) return gb.toFixed(1) + " GB"
        let mb = bytes / (1024 * 1024)
        return mb.toFixed(0) + " MB"
    }

    onInstallPathChanged: { refreshFreeSpace(); refreshExisting() }
    onEffectiveInstallPathChanged: refreshExisting()
    onTempPathChanged: { refreshFreeSpace(); refreshExisting() }
    onEditionIndexChanged: {
        if (visible) sizeFetchDebounce.restart()
        refreshExisting()
    }
    onVoiceChecksChanged: if (visible) sizeFetchDebounce.restart()

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
        manifest = m

        // first locale on, rest off to match the prior hoyo default
        let vs = []
        for (let i = 0; i < voiceLocales.length; i++) vs.push(i === 0)
        voiceChecks = vs

        if (installPath === "") installPath = defaultInstallPath()
        if (defaults) prefixPath = defaults.getConfig()["wine.prefix"] || ""
        loadRunners()
        refreshFreeSpace()
        refreshInstallSize()
        refreshExisting()
        opened = true
        forceActiveFocus()
    }

    function hide() { opened = false }

    onVisibleChanged: {
        if (!visible) {
            manifest = null
            manifestId = ""
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
    }

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
        let opts = []
        for (let i = 0; i < arr.length; i++) {
            let v = arr[i]
            let label = v
            if (v.indexOf("steam:") === 0) label = v.substring(6) + " (steam)"
            if (v === "system") label = "System Wine"
            opts.push({ label: label, value: v })
        }
        if (opts.length === 0) opts.push({ label: "System Wine", value: "system" })
        runnerOptions = opts

        // first match from the end wins, so GE-Proton-10-34 beats GE-Proton-10-10
        let prefs = (manifest && manifest.runner_preference) ? manifest.runner_preference : []
        let pick = -1
        outer: for (let p = 0; p < prefs.length; p++) {
            let needle = String(prefs[p] || "").toLowerCase()
            if (needle === "") continue
            for (let i = opts.length - 1; i >= 0; i--) {
                if (opts[i].value.toLowerCase().indexOf(needle) !== -1) {
                    pick = i
                    break outer
                }
            }
        }
        runnerIndex = pick >= 0 ? pick : 0
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

    // hoverEnabled true so cards underneath dont stay lit while the dialog is open
    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, 0.55)
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.AllButtons
            onClicked: (mouse) => { if (mouse.button === Qt.LeftButton) root.cancelled() }
            onWheel: (wheel) => wheel.accepted = true
            cursorShape: Qt.ArrowCursor
        }
    }

    // popups parent here not the card, card's layer.enabled would clip them
    property bool isDropdownHost: true

    Rectangle {
        id: card
        anchors.centerIn: parent
        width: Math.min(parent.width - 80, 480)
        height: Math.min(parent.height - 60, contentCol.implicitHeight + 48)
        radius: 24
        color: theme.surface
        border.width: 1
        border.color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.08)

        Behavior on height {
            NumberAnimation { duration: 160; easing.type: Easing.OutCubic }
        }

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            onClicked: {}
            onWheel: (wheel) => wheel.accepted = true
        }

        layer.enabled: true
        layer.effect: DropShadow {
            radius: 24
            samples: 32
            color: Qt.rgba(0, 0, 0, 0.4)
            horizontalOffset: 0
            verticalOffset: 6
        }

        Flickable {
            id: cardScroll
            anchors.fill: parent
            anchors.margins: 24
            contentWidth: width
            contentHeight: contentCol.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            interactive: contentHeight > height
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

        ColumnLayout {
            id: contentCol
            width: cardScroll.width
            spacing: 18

            RowLayout {
                Layout.fillWidth: true
                spacing: 12

                SvgIcon {
                    name: "local_activity"
                    size: 20
                    color: theme.textMuted
                    Layout.alignment: Qt.AlignVCenter
                }

                Text {
                    Layout.fillWidth: true
                    text: root.displayName ? "Install " + root.displayName : "Install"
                    color: theme.text
                    font.pixelSize: 18
                    font.weight: Font.DemiBold
                    elide: Text.ElideRight
                }

                IconButton {
                    icon: "close"
                    size: 28
                    onClicked: root.cancelled()
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 8
                visible: root.editions.length > 1

                Text {
                    text: "Edition"
                    color: theme.textMuted
                    font.pixelSize: 13
                    font.weight: Font.Medium
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8

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
                                    ? Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.15)
                                    : edBtnHover.containsMouse
                                        ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                                        : "transparent"
                                border.width: index === root.editionIndex ? 1 : 0
                                border.color: Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.3)

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
                spacing: 8
                visible: root.voiceLocales.length > 0

                Text {
                    text: "Voice Packs"
                    color: theme.textMuted
                    font.pixelSize: 13
                    font.weight: Font.Medium
                }

                GridLayout {
                    Layout.fillWidth: true
                    columns: 2
                    columnSpacing: 12
                    rowSpacing: 8

                    Repeater {
                        model: root.voiceLocales

                        RowLayout {
                            required property var modelData
                            required property int index

                            Layout.fillWidth: true
                            spacing: 8

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
                    label: "Library folder"
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
                            parts.push("Calculating size…")
                        } else if (root.sizeError !== "") {
                            parts.push("Size unavailable")
                        } else if (root.installBytes >= 0) {
                            parts.push(formatBytes(root.installBytes) + " install")
                        }
                        if (root.installFreeBytes >= 0) {
                            parts.push(formatBytes(root.installFreeBytes) + " free")
                        }
                        if (root.existingInstall) {
                            if (root.existingVersion !== "") {
                                parts.push("existing install detected · v" + root.existingVersion)
                            } else {
                                parts.push("Unknown Version")
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
                    label: "Temp path (optional)"
                    placeholder: "auto — next to install path"
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
                            parts.push("Found existing files · " + formatBytes(root.existingTempBytes))
                        }
                        if (root.downloadBytes >= 0) {
                            parts.push(formatBytes(root.downloadBytes) + " download")
                        }
                        if (root.tempFreeBytes >= 0) {
                            parts.push(formatBytes(root.tempFreeBytes) + " free")
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
                label: "Prefix path (optional)"
                placeholder: "auto — created per game"
                selectFolder: true
                gameModel: root.gameModel
                text: root.prefixPath
                onTextEdited: (t) => root.prefixPath = t
                onAccepted: (p) => root.prefixPath = p
            }

            M3Dropdown {
                Layout.fillWidth: true
                label: "Runner"
                options: root.runnerOptions
                currentIndex: root.runnerIndex
                onSelected: (v) => {
                    for (let i = 0; i < root.runnerOptions.length; i++) {
                        if (root.runnerOptions[i].value === v) { root.runnerIndex = i; break }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: 8
                spacing: 12

                Item { Layout.fillWidth: true }

                Item {
                    implicitWidth: 100
                    implicitHeight: 38

                    Rectangle {
                        anchors.fill: parent
                        radius: 19
                        color: cancelHover.containsPress ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.12)
                            : cancelHover.containsMouse ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                            : "transparent"
                        Behavior on color { ColorAnimation { duration: 100 } }
                    }
                    Text {
                        anchors.centerIn: parent
                        text: "Cancel"
                        color: theme.text
                        font.pixelSize: 13
                        font.weight: Font.Medium
                    }
                    MouseArea {
                        id: cancelHover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.cancelled()
                    }
                }

                Item {
                    id: installBtn
                    implicitWidth: 110
                    implicitHeight: 38
                    property bool canInstall:
                        root.manifest !== null
                        && root.installPath.trim().length > 0
                        && root.hasEnoughSpace()

                    Rectangle {
                        anchors.fill: parent
                        radius: 19
                        color: installBtn.canInstall
                            ? theme.accent
                            : Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.08)
                        opacity: installBtn.canInstall
                            ? (installHover.containsPress ? 0.8 : installHover.containsMouse ? 0.95 : 0.9)
                            : 1.0
                        scale: installBtn.canInstall && installHover.containsPress ? 0.97 : 1.0
                        Behavior on color { ColorAnimation { duration: 120 } }
                        Behavior on opacity { NumberAnimation { duration: 100 } }
                        Behavior on scale { NumberAnimation { duration: 100 } }
                    }
                    Text {
                        anchors.centerIn: parent
                        text: root.existingInstall
                            ? "Import"
                            : (root.existingTempSegments > 0 ? "Resume" : "Install")
                        color: installBtn.canInstall
                            ? theme.accentOn
                            : Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.35)
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                        Behavior on color { ColorAnimation { duration: 120 } }
                    }
                    MouseArea {
                        id: installHover
                        anchors.fill: parent
                        hoverEnabled: installBtn.canInstall
                        enabled: installBtn.canInstall
                        cursorShape: installBtn.canInstall ? Qt.PointingHandCursor : Qt.ForbiddenCursor
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
                        }
                    }
                }
            }
        }
        }
    }
}
