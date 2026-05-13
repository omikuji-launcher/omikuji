import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects

import "../widgets"

Item {
    id: root

    property var gameModel: null
    property var gogModel: null

    property int runnersVersion: 0
    onRunnersVersionChanged: if (opened) loadRunners()

    property int gameIndex: -1

    signal installEnqueued(string downloadId)
    signal cancelled()

    property bool opened: false
    visible: opacity > 0.001
    opacity: opened ? 1.0 : 0.0
    Behavior on opacity {
        NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
    }
    z: 1000

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

    property real existingInstallBytes: 0
    property bool hasResumeState: false

    function hasEnoughSpace() {
        if (freeSpaceBytes < 0) return false
        if (installBytes < 0) return false
        if (existingInstallBytes > 0 || hasResumeState) return true
        // gogdl reports 0 for goodies/stubs/linux-natives, gogdl itself errors if truly uninstallable
        if (installBytes === 0) return true
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
        let raw = gameModel.gog_check_existing_install(gameData.appName, effectiveInstallPath)
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
        let id = "gog-" + Date.now().toString(36) + "-" + Math.random().toString(36).substring(2, 8)
        _sizeRequestId = id
        gameModel.fetch_gog_install_size(id, gameData.appName)
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

    function formatBytes(bytes) {
        if (bytes <= 0) return ""
        let gb = bytes / (1024 * 1024 * 1024)
        if (gb >= 1) return gb.toFixed(1) + " GB"
        let mb = bytes / (1024 * 1024)
        return mb.toFixed(0) + " MB"
    }

    onInstallPathChanged: refreshFreeSpace()
    onEffectiveInstallPathChanged: refreshExistingInstall()

    readonly property bool isImportMode: gameData && gameData.isInstalled === true

    readonly property string effectiveInstallPath: {
        if (!gameData) return ""
        if (isImportMode) return gameData.installPath || ""
        let safe = (gameData.title || "Game").replace(/[\\/:*?"<>|]/g, "").trim()
        let base = (installPath || "").trim().replace(/\/+$/, "")
        if (base === "" || safe === "") return ""
        return base + "/" + safe
    }

    function show() {
        if (!gogModel || gameIndex < 0) return
        gameData = gogModel.get_game_at(gameIndex)
        let gogGameDir = gameData && gameData.installPath ? gameData.installPath : ""
        if (gameData && gameData.isInstalled === true && gogGameDir !== "") {
            let slash = gogGameDir.lastIndexOf("/")
            installPath = slash > 0 ? gogGameDir.substring(0, slash) : gogGameDir
        } else if (installPath === "") {
            installPath = defaultInstallPath(gameData ? gameData.title : "")
        }
        if (defaults) prefixPath = defaults.getConfig()["wine.prefix"] || ""
        loadRunners()
        refreshFreeSpace()
        refreshInstallSize()
        refreshExistingInstall()
        opened = true
        forceActiveFocus()
    }

    function hide() {
        opened = false
    }

    onVisibleChanged: {
        if (!visible) {
            gameIndex = -1
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
        }
    }

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

        let pick = -1
        for (let i = opts.length - 1; i >= 0; i--) {
            let v = opts[i].value
            if (v.indexOf("GE-Proton") !== -1 || v.indexOf("Proton-GE") !== -1) { pick = i; break }
        }
        if (pick < 0) {
            for (let i = opts.length - 1; i >= 0; i--) {
                if (opts[i].value.indexOf("wine-ge") !== -1) { pick = i; break }
            }
        }
        runnerIndex = pick >= 0 ? pick : 0
    }

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

    // popups parent here not on the card, card's layer.enabled would clip them
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
                    name: "gog"
                    size: 20
                    color: theme.textMuted
                    Layout.alignment: Qt.AlignVCenter
                }

                Text {
                    Layout.fillWidth: true
                    text: "Install " + (root.gameData ? root.gameData.title : "")
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

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 96
                radius: 12
                color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.04)
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
                            radius: 12
                        }
                    }
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4

                M3FileField {
                    Layout.fillWidth: true
                    label: root.isImportMode ? "Install path" : "Library folder"
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
                            let label = "Found existing files"
                            if (root.existingInstallBytes > 0) label += " · " + formatBytes(root.existingInstallBytes)
                            if (root.hasResumeState) label += " · resume state"
                            parts.push(label)
                        }
                        if (root.downloadBytes === -2) {
                            parts.push("Calculating size…")
                        } else if (root.sizeError !== "") {
                            parts.push("Size unavailable")
                        } else if (root.installBytes === 0 && root.downloadBytes === 0) {
                            parts.push("Size unknown")
                        } else if (root.installBytes >= 0) {
                            parts.push(formatBytes(root.installBytes) + " install")
                            if (root.downloadBytes > 0) {
                                parts.push(formatBytes(root.downloadBytes) + " download")
                            }
                        }
                        if (root.freeSpaceBytes >= 0) {
                            parts.push(formatBytes(root.freeSpaceBytes) + " free")
                        }
                        return parts.join(" · ")
                    }
                    color: (root.existingInstallBytes > 0 || root.hasResumeState)
                        ? theme.accent
                        : (root.hasEnoughSpace() ? theme.textFaint : "#e06060")
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
                    property bool canInstall: root.installPath.trim().length > 0 && root.hasEnoughSpace()

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
                        text: {
                            if (root.gameData && root.gameData.isInstalled) {
                                return root.gameData.hasLibraryEntry ? "Repair" : "Import"
                            }
                            if (root.existingInstallBytes > 0 || root.hasResumeState) return "Resume"
                            return "Install"
                        }
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
                                : ""
                            let id = root.gogModel.enqueue_install(
                                root.gameIndex,
                                root.effectiveInstallPath,
                                root.prefixPath,
                                runner,
                                root.isImportMode
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
