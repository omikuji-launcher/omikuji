pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

import "../../lib/RunnerGrouping.js" as RG
import "../../lib/Nav.js" as Nav

Item {
    id: root

    property var defaults: null
    property var gameModel: null
    property var appSettings: null

    signal applyToExistingRequested()

    property var cfg: ({})
    property var setKeys: []

    implicitHeight: content.height

    onDefaultsChanged: refresh()

    function refresh() {
        if (!defaults) return
        cfg = defaults.getConfig()
        try { setKeys = JSON.parse(defaults.setKeysJson()) } catch(e) { setKeys = [] }
    }

    function isSet(key) { return setKeys.indexOf(key) !== -1 }

    property bool isProtonWine: isProtonVersion(cfg["wine.version"] || "")

    function isProtonVersion(version) {
        return gameModel ? gameModel.runner_is_proton(String(version || "")) : false
    }

    function update(key, value) {
        if (!defaults) return
        defaults.updateField(key, String(value))
        refresh()
    }

    function reset(key) {
        if (!defaults) return
        defaults.resetField(key)
        refresh()
    }

    Connections {
        target: root.defaults
        function onChanged() { root.refresh() }
    }

    readonly property int badgeSize: 32
    readonly property int badgeGap: 8
    readonly property int badgeSlot: badgeSize + badgeGap

    component ResetBadge: IconButton {
        id: badge
        property string fieldKey: ""
        function navFallback() { return Nav.collect(badge.parent)[0] || null }
        readonly property bool active: root.isSet(fieldKey)
        size: root.badgeSize
        icon: "sync"
        opacity: active ? 1 : 0
        enabled: active
        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
        onClicked: root.reset(fieldKey)
    }

    component ToggleRow: SettingsRow {
        id: toggleRow
        property string fieldKey: ""
        property string toggleDescription: ""
        property bool toggleEnabled: true
        label: SettingLabels.label(fieldKey)
        description: toggleDescription
        width: parent.width
        opacity: toggleEnabled ? 1 : 0.65

        Row {
            spacing: 12
            M3Switch {
                anchors.verticalCenter: parent.verticalCenter
                enabled: toggleRow.toggleEnabled
                opacity: toggleRow.toggleEnabled ? 1 : 0.45
                checked: root.cfg[toggleRow.fieldKey] === true
                onToggled: (val) => root.update(toggleRow.fieldKey, val)
            }
            ResetBadge {
                anchors.verticalCenter: parent.verticalCenter
                fieldKey: toggleRow.fieldKey
            }
        }
    }

    component LayerVersionRow: Row {
        id: layerRow
        property string fieldKey: ""
        property string kind: ""

        width: parent.width
        spacing: root.badgeGap

        M3Dropdown {
            id: layerDd
            label: SettingLabels.label(layerRow.fieldKey)
            width: parent.width - root.badgeSlot
            options: {
                let installed = []
                try {
                    installed = root.gameModel ? JSON.parse(root.gameModel.dll_versions_for_kind(layerRow.kind)) || [] : []
                } catch (e) {
                    installed = []
                }
                return [{ label: qsTr("Built-in"), value: "builtin" }]
                    .concat(installed.map(tag => ({ label: tag, value: tag })))
            }
            currentIndex: Math.max(0, RG.indexOfValue(options, root.cfg[layerRow.fieldKey] || "builtin"))
            onSelected: (tag) => root.update(layerRow.fieldKey, tag)
        }

        ResetBadge {
            y: layerDd.boxCenterY - height / 2
            fieldKey: layerRow.fieldKey
        }
    }

    Column {
        id: content
        width: parent.width
        spacing: Theme.space.xxl

        Item {
            width: parent.width
            height: 32

            M3Button {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                small: true
                variant: "tonal"
                icon: "sync"
                text: qsTr("Apply to existing games")
                onClicked: root.applyToExistingRequested()
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("wine")
            icon: "wine"
            width: parent.width

            Row {
                width: parent.width
                spacing: root.badgeGap
                M3Dropdown {
                    id: versionDd
                    label: SettingLabels.label("wine.version")
                    width: parent.width - root.badgeSlot
                    options: {
                        let runners = root.gameModel ? JSON.parse(root.gameModel.list_runners()) : []
                        return RG.runnerOptions(runners, root.cfg["wine.version"] || "", {
                            includeSystemDefault: true,
                            defaultLabel: qsTr("System default"),
                            emptyLabel: qsTr("No runners installed"),
                            tint: Theme.error,
                            missingLabel: qsTr("missing")
                        })
                    }
                    currentIndex: {
                        return RG.selectedIndex(options, root.cfg["wine.version"] || "")
                    }
                    onSelected: (val) => root.update("wine.version", val)
                }
                ResetBadge {
                    y: versionDd.boxCenterY - height / 2
                    fieldKey: "wine.version"
                }
            }

            Row {
                width: parent.width
                spacing: root.badgeGap
                M3FileField {
                    id: prefixField
                    label: SettingLabels.label("wine.prefix")
                    placeholder: qsTr("empty = auto-create per game")
                    text: root.cfg["wine.prefix"] || ""
                    selectFolder: true
                    width: parent.width - root.badgeSlot
                    gameModel: root.gameModel
                    onTextEdited: (t) => root.update("wine.prefix", t)
                    onAccepted: (p) => root.update("wine.prefix", p)
                }
                ResetBadge {
                    y: prefixField.boxCenterY - height / 2
                    fieldKey: "wine.prefix"
                }
            }

            Row {
                width: parent.width
                spacing: root.badgeGap
                M3Dropdown {
                    id: archDd
                    label: SettingLabels.label("wine.prefix_arch")
                    width: parent.width - root.badgeSlot
                    options: [
                        { label: qsTr("64-bit (win64)"), value: "win64" },
                        { label: qsTr("32-bit (win32)"), value: "win32" }
                    ]
                    currentIndex: root.cfg["wine.prefix_arch"] === "win32" ? 1 : 0
                    onSelected: (val) => root.update("wine.prefix_arch", val)
                }
                ResetBadge {
                    y: archDd.boxCenterY - height / 2
                    fieldKey: "wine.prefix_arch"
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("sync")
            icon: "sync"
            width: parent.width

            ToggleRow { fieldKey: "wine.esync" }
            ToggleRow { fieldKey: "wine.fsync" }
            ToggleRow {
                fieldKey: "wine.ntsync"
                toggleDescription: !root.isProtonWine
                    ? qsTr("Only applied when the default Wine version is Proton")
                    : ""
                toggleEnabled: root.isProtonWine
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("translation_layers")
            icon: "layers"
            width: parent.width

            ToggleRow { fieldKey: "wine.dxvk" }
            LayerVersionRow { fieldKey: "wine.dxvk_version"; kind: "dxvk" }

            ToggleRow { fieldKey: "wine.vkd3d" }
            LayerVersionRow { fieldKey: "wine.vkd3d_version"; kind: "vkd3d" }

            ToggleRow { fieldKey: "wine.dxvk_nvapi"; toggleDescription: qsTr("Nvidia DLSS support") }
            LayerVersionRow { fieldKey: "wine.dxvk_nvapi_version"; kind: "dxvk_nvapi" }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("compatibility")
            icon: "verified"
            width: parent.width

            ToggleRow { fieldKey: "wine.battleye" }
            ToggleRow { fieldKey: "wine.easyanticheat" }
            ToggleRow { fieldKey: "wine.fsr"; toggleDescription: "AMD FidelityFX Super Resolution" }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("display")
            icon: "desktop_windows"
            width: parent.width

            ToggleRow { fieldKey: "wine.dpi_scaling" }

            Row {
                width: parent.width
                spacing: root.badgeGap
                visible: root.cfg["wine.dpi_scaling"] === true
                SettingsRow {
                    label: SettingLabels.label("wine.dpi")
                    width: parent.width - root.badgeSlot
                    contentRightMargin: 78
                    M3SpinBox {
                        from: 72
                        to: 288
                        stepSize: 12
                        value: root.cfg["wine.dpi"] || 96
                        onMoved: (val) => root.update("wine.dpi", val)
                    }
                }
                ResetBadge {
                    anchors.verticalCenter: parent.verticalCenter
                    fieldKey: "wine.dpi"
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("drivers")
            icon: "headphones"
            width: parent.width

            Row {
                width: parent.width
                spacing: root.badgeGap
                M3Dropdown {
                    id: audioDd
                    label: SettingLabels.label("wine.audio_driver")
                    width: parent.width - root.badgeSlot
                    options: [
                        { label: qsTr("Default"), value: "" },
                        { label: "PulseAudio", value: "pulse" },
                        { label: "ALSA", value: "alsa" }
                    ]
                    currentIndex: {
                        let d = root.cfg["wine.audio_driver"] || ""
                        if (d === "pulse") return 1
                        if (d === "alsa") return 2
                        return 0
                    }
                    onSelected: (val) => root.update("wine.audio_driver", val)
                }
                ResetBadge {
                    y: audioDd.boxCenterY - height / 2
                    fieldKey: "wine.audio_driver"
                }
            }

            Row {
                width: parent.width
                spacing: root.badgeGap
                M3Dropdown {
                    id: gfxDd
                    label: SettingLabels.label("wine.graphics_driver")
                    width: parent.width - root.badgeSlot
                    options: [
                        { label: qsTr("Default"), value: "" },
                        { label: "X11", value: "x11" },
                        { label: "Wayland", value: "wayland" }
                    ]
                    currentIndex: {
                        let d = root.cfg["wine.graphics_driver"] || ""
                        if (d === "x11") return 1
                        if (d === "wayland") return 2
                        return 0
                    }
                    onSelected: (val) => root.update("wine.graphics_driver", val)
                }
                ResetBadge {
                    y: gfxDd.boxCenterY - height / 2
                    fieldKey: "wine.graphics_driver"
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("dll_overrides")
            icon: "build"
            width: parent.width

            Row {
                width: parent.width
                spacing: root.badgeGap
                KeyValueTable {
                    id: dllKvt
                    width: parent.width - root.badgeSlot
                    json: root.cfg["wine.dll_overrides"] || "{}"
                    keyPlaceholder: "dll_name"
                    valuePlaceholder: "n,b"
                    addLabel: qsTr("Add override")
                    onChanged: (j) => root.update("wine.dll_overrides", j)
                }
                ResetBadge {
                    anchors.top: parent.top
                    anchors.topMargin: 8
                    fieldKey: "wine.dll_overrides"
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("launch")
            icon: "terminal"
            width: parent.width

            Row {
                width: parent.width
                spacing: root.badgeGap
                M3TextField {
                    id: cmdPrefixTf
                    label: SettingLabels.label("launch.command_prefix")
                    placeholder: qsTr("prepended to every game's command")
                    text: root.cfg["launch.command_prefix"] || ""
                    width: parent.width - root.badgeSlot
                    gameModel: root.gameModel
                    onTextEdited: (t) => root.update("launch.command_prefix", t)
                }
                ResetBadge {
                    y: cmdPrefixTf.boxCenterY - height / 2
                    fieldKey: "launch.command_prefix"
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("environment")
            icon: "tune"
            width: parent.width

            Row {
                width: parent.width
                spacing: root.badgeGap
                KeyValueTable {
                    id: envKvt
                    width: parent.width - root.badgeSlot
                    json: root.cfg["launch.env"] || "{}"
                    keyPlaceholder: "VAR_NAME"
                    valuePlaceholder: "value"
                    addLabel: qsTr("Add variable")
                    gameModel: root.gameModel
                    onChanged: (j) => root.update("launch.env", j)
                }
                ResetBadge {
                    anchors.top: parent.top
                    anchors.topMargin: 8
                    fieldKey: "launch.env"
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("graphics")
            icon: "fullscreen"
            width: parent.width

            ToggleRow { fieldKey: "graphics.mangohud"; toggleDescription: qsTr("FPS overlay") }

            Row {
                width: parent.width
                spacing: root.badgeGap
                M3Dropdown {
                    id: gpuDd
                    label: SettingLabels.label("graphics.gpu")
                    width: parent.width - root.badgeSlot
                    options: root.gameModel ? JSON.parse(root.gameModel.list_gpus()).map(g => ({ label: g[0], value: g[1] })) : [{ label: qsTr("Default"), value: "" }]
                    currentIndex: {
                        let v = root.cfg["graphics.gpu"] || ""
                        let gpus = root.gameModel ? JSON.parse(root.gameModel.list_gpus()) : []
                        let idx = gpus.findIndex(g => g[1] === v)
                        return idx >= 0 ? idx : 0
                    }
                    onSelected: (val) => root.update("graphics.gpu", val)
                }
                ResetBadge {
                    y: gpuDd.boxCenterY - height / 2
                    fieldKey: "graphics.gpu"
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("gamescope")
            icon: "monitor"
            width: parent.width

            ToggleRow { fieldKey: "graphics.gamescope.enabled"; toggleDescription: qsTr("run every game inside gamescope") }

            Column {
                width: parent.width
                spacing: 12
                visible: root.cfg["graphics.gamescope.enabled"] === true

                ToggleRow { fieldKey: "graphics.gamescope.fullscreen" }
                ToggleRow { fieldKey: "graphics.gamescope.borderless" }
                ToggleRow { fieldKey: "graphics.gamescope.integer_scaling" }
                ToggleRow { fieldKey: "graphics.gamescope.hdr" }

                Row {
                    width: parent.width
                    spacing: root.badgeGap
                    SettingsRow {
                        label: SettingLabels.label("graphics.gamescope.fps")
                        width: parent.width - root.badgeSlot
                        contentRightMargin: 78
                        M3SpinBox {
                            id: fpsSpinBox
                            from: 0
                            to: 999
                            stepSize: 1
                            value: root.cfg["graphics.gamescope.fps"] || 0
                            zeroPlaceholder: "—"
                            onMoved: (val) => root.update("graphics.gamescope.fps", val)
                        }
                    }
                    ResetBadge {
                        anchors.verticalCenter: parent.verticalCenter
                        fieldKey: "graphics.gamescope.fps"
                    }
                }

                Row {
                    width: parent.width
                    spacing: root.badgeGap
                    SettingsRow {
                        label: SettingLabels.label("graphics.gamescope.refresh_rate")
                        width: parent.width - root.badgeSlot
                        contentRightMargin: 78
                        M3SpinBox {
                            id: refreshRateSpinBox
                            from: 0
                            to: 999
                            stepSize: 1
                            value: root.cfg["graphics.gamescope.refresh_rate"] || 0
                            zeroPlaceholder: "—"
                            onMoved: (val) => root.update("graphics.gamescope.refresh_rate", val)
                        }
                    }
                    ResetBadge {
                        anchors.verticalCenter: parent.verticalCenter
                        fieldKey: "graphics.gamescope.refresh_rate"
                    }
                }

                Row {
                    width: parent.width
                    spacing: root.badgeGap
                    M3Dropdown {
                        id: filterDd
                        label: SettingLabels.label("graphics.gamescope.filter")
                        width: parent.width - root.badgeSlot
                        options: [
                            { label: qsTr("None"), value: "" },
                            { label: qsTr("Nearest"), value: "nearest" },
                            { label: qsTr("Linear"), value: "linear" },
                            { label: "FSR", value: "fsr" },
                            { label: "NIS", value: "nis" },
                            { label: qsTr("Pixel"), value: "pixel" }
                        ]
                        currentIndex: {
                            let f = root.cfg["graphics.gamescope.filter"] || ""
                            let idx = ["", "nearest", "linear", "fsr", "nis", "pixel"].indexOf(f)
                            return idx >= 0 ? idx : 0
                        }
                        onSelected: (val) => root.update("graphics.gamescope.filter", val)
                    }
                    ResetBadge {
                        y: filterDd.boxCenterY - height / 2
                        fieldKey: "graphics.gamescope.filter"
                    }
                }

                Row {
                    width: parent.width
                    spacing: root.badgeGap
                    visible: (root.cfg["graphics.gamescope.filter"] || "") === "fsr"
                    M3Slider {
                        id: sharpSlider
                        label: SettingLabels.label("graphics.gamescope.fsr_sharpness")
                        from: 0
                        to: 20
                        stepSize: 1
                        value: root.cfg["graphics.gamescope.fsr_sharpness"] || 0
                        width: parent.width - root.badgeSlot
                        onMoved: (val) => root.update("graphics.gamescope.fsr_sharpness", Math.round(val))
                    }
                    ResetBadge {
                        y: sharpSlider.boxCenterY - height / 2
                        fieldKey: "graphics.gamescope.fsr_sharpness"
                    }
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("performance")
            icon: "speed"
            width: parent.width

            ToggleRow { fieldKey: "system.gamemode"; toggleDescription: qsTr("Feral GameMode (gamemoderun)") }

            SettingsRow {
                label: SettingLabels.label("system.cpu_limit")
                description: qsTr("0 = no limit")
                width: parent.width
                contentRightMargin: 74
                Row {
                    spacing: 12
                    M3SpinBox {
                        anchors.verticalCenter: parent.verticalCenter
                        from: 0
                        to: root.gameModel ? Math.max(1, root.gameModel.cpuCoreCount()) : 1
                        stepSize: 1
                        value: root.cfg["system.cpu_limit"] || 0
                        zeroPlaceholder: "—"
                        onMoved: (val) => root.update("system.cpu_limit", val)
                    }
                    ResetBadge {
                        anchors.verticalCenter: parent.verticalCenter
                        fieldKey: "system.cpu_limit"
                    }
                }
            }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("audio")
            icon: "volume_up"
            width: parent.width

            ToggleRow { fieldKey: "system.pulse_latency" }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("power")
            icon: "power_settings_new"
            width: parent.width

            ToggleRow { fieldKey: "system.prevent_sleep"; toggleDescription: qsTr("inhibit screensaver and sleep") }
        }

        SettingsSection {
            label: SettingLabels.groupTitle("discord")
            icon: "local_activity"
            width: parent.width

            ToggleRow { fieldKey: "system.discord_rpc"; toggleDescription: qsTr("show the game on your Discord profile while it runs") }
        }
    }
}
