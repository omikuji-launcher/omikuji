import QtQuick
import QtQuick.Layouts

import "."
import "../lib/RunnerGrouping.js" as RG
import "../controls"
import "../primitives"

Item {
    id: root

    property var defaults: null
    property var gameModel: null

    signal applyToExistingRequested()
    signal manageSetsRequested(string kind)

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
        let v = String(version || "").toLowerCase()
        return v.indexOf("proton") !== -1
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
        target: defaults
        function onChanged() { root.refresh() }
    }

    component ResetBadge: Item {
        property string fieldKey: ""
        property bool active: root.isSet(fieldKey)
        width: 24
        height: 24
        opacity: active ? 1 : 0
        enabled: active
        Behavior on opacity { NumberAnimation { duration: 120 } }

        Rectangle {
            anchors.fill: parent
            radius: width / 2
            color: iconArea.containsMouse
                ? theme.alpha(theme.text, 0.10)
                : "transparent"
            Behavior on color { ColorAnimation { duration: 100 } }
        }

        SvgIcon {
            anchors.centerIn: parent
            size: 14
            name: "sync"
            color: iconArea.containsMouse ? theme.accent : theme.textSubtle
        }

        MouseArea {
            id: iconArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.reset(fieldKey)
        }
    }

    component ToggleRow: SettingsRow {
        id: toggleRow
        property string fieldKey: ""
        property string toggleLabel: ""
        property string toggleDescription: ""
        property bool toggleEnabled: true
        label: toggleLabel
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

    Column {
        id: content
        width: parent.width
        spacing: theme.space.xxl

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
            label: "Wine"
            icon: "wine_bar"
            width: parent.width

            Row {
                width: parent.width
                spacing: 8
                M3Dropdown {
                    id: versionDd
                    label: qsTr("Version")
                    width: parent.width - 32
                    options: {
                        if (!root.gameModel) return [{ label: qsTr("System default"), value: "" }]
                        let runners = JSON.parse(root.gameModel.list_runners())
                        return RG.groupRunners(runners, { includeSystemDefault: true })
                    }
                    currentIndex: {
                        let v = root.cfg["wine.version"] || ""
                        let idx = RG.indexOfValue(options, v)
                        return idx >= 0 ? idx : 0
                    }
                    onSelected: (val) => root.update("wine.version", val)
                }
                ResetBadge {
                    anchors.verticalCenter: versionDd.verticalCenter
                    fieldKey: "wine.version"
                }
            }

            Row {
                width: parent.width
                spacing: 8
                M3FileField {
                    id: prefixField
                    label: qsTr("Prefix")
                    placeholder: qsTr("empty = auto-create per game")
                    text: root.cfg["wine.prefix"] || ""
                    selectFolder: true
                    width: parent.width - 32
                    gameModel: root.gameModel
                    onTextEdited: (t) => root.update("wine.prefix", t)
                    onAccepted: (p) => root.update("wine.prefix", p)
                }
                ResetBadge {
                    anchors.verticalCenter: prefixField.verticalCenter
                    fieldKey: "wine.prefix"
                }
            }

            Row {
                width: parent.width
                spacing: 8
                M3Dropdown {
                    id: archDd
                    label: qsTr("Architecture")
                    width: parent.width - 32
                    options: [
                        { label: qsTr("64-bit (win64)"), value: "win64" },
                        { label: qsTr("32-bit (win32)"), value: "win32" }
                    ]
                    currentIndex: root.cfg["wine.prefix_arch"] === "win32" ? 1 : 0
                    onSelected: (val) => root.update("wine.prefix_arch", val)
                }
                ResetBadge {
                    anchors.verticalCenter: archDd.verticalCenter
                    fieldKey: "wine.prefix_arch"
                }
            }
        }

        SettingsSection {
            label: qsTr("Sync")
            icon: "sync"
            width: parent.width

            ToggleRow { fieldKey: "wine.esync"; toggleLabel: qsTr("Esync") }
            ToggleRow { fieldKey: "wine.fsync"; toggleLabel: qsTr("Fsync") }
            ToggleRow {
                fieldKey: "wine.ntsync"
                toggleLabel: qsTr("NTSync")
                toggleDescription: !root.isProtonWine
                    ? qsTr("Only applied when the default Wine version is Proton")
                    : ""
                toggleEnabled: root.isProtonWine
            }
        }

        SettingsSection {
            label: qsTr("Translation Layers")
            icon: "layers"
            width: parent.width

            ToggleRow { fieldKey: "wine.dxvk"; toggleLabel: "DXVK" }
            ToggleRow { fieldKey: "wine.vkd3d"; toggleLabel: "VKD3D" }
            ToggleRow { fieldKey: "wine.d3d_extras"; toggleLabel: qsTr("D3D Extras") }
            ToggleRow { fieldKey: "wine.dxvk_nvapi"; toggleLabel: "DXVK-NVAPI"; toggleDescription: qsTr("Nvidia DLSS support") }
        }

        SettingsSection {
            label: qsTr("Compatibility")
            icon: "verified"
            width: parent.width

            ToggleRow { fieldKey: "wine.battleye"; toggleLabel: "BattlEye" }
            ToggleRow { fieldKey: "wine.easyanticheat"; toggleLabel: "EasyAntiCheat" }
            ToggleRow { fieldKey: "wine.fsr"; toggleLabel: "FSR"; toggleDescription: "AMD FidelityFX Super Resolution" }
        }

        SettingsSection {
            label: qsTr("Display")
            icon: "desktop_windows"
            width: parent.width

            ToggleRow { fieldKey: "wine.dpi_scaling"; toggleLabel: qsTr("DPI Scaling") }

            Row {
                width: parent.width
                spacing: 8
                visible: root.cfg["wine.dpi_scaling"] === true
                M3Slider {
                    id: dpiSlider
                    label: qsTr("DPI")
                    from: 72
                    to: 288
                    stepSize: 12
                    value: root.cfg["wine.dpi"] || 96
                    width: parent.width - 32
                    onMoved: (val) => root.update("wine.dpi", Math.round(val))
                }
                ResetBadge {
                    anchors.verticalCenter: dpiSlider.verticalCenter
                    fieldKey: "wine.dpi"
                }
            }
        }

        SettingsSection {
            label: qsTr("Drivers")
            icon: "headphones"
            width: parent.width

            Row {
                width: parent.width
                spacing: 8
                M3Dropdown {
                    id: audioDd
                    label: qsTr("Audio Driver")
                    width: parent.width - 32
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
                    anchors.verticalCenter: audioDd.verticalCenter
                    fieldKey: "wine.audio_driver"
                }
            }

            Row {
                width: parent.width
                spacing: 8
                M3Dropdown {
                    id: gfxDd
                    label: qsTr("Graphics Driver")
                    width: parent.width - 32
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
                    anchors.verticalCenter: gfxDd.verticalCenter
                    fieldKey: "wine.graphics_driver"
                }
            }
        }

        SettingsSection {
            label: qsTr("DLL Overrides")
            icon: "build"
            width: parent.width

            Row {
                width: parent.width
                spacing: 8
                KeyValueTable {
                    id: dllKvt
                    width: parent.width - 32
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
            label: qsTr("Launch")
            icon: "terminal"
            width: parent.width

            Row {
                width: parent.width
                spacing: 8
                M3TextField {
                    id: cmdPrefixTf
                    label: qsTr("Command Prefix")
                    placeholder: qsTr("prepended to every game's command")
                    text: root.cfg["launch.command_prefix"] || ""
                    width: parent.width - 32
                    onTextEdited: (t) => root.update("launch.command_prefix", t)
                }
                ResetBadge {
                    anchors.verticalCenter: cmdPrefixTf.verticalCenter
                    fieldKey: "launch.command_prefix"
                }
            }
        }

        SettingsSection {
            label: qsTr("Environment")
            icon: "tune"
            width: parent.width

            Row {
                width: parent.width
                spacing: 8
                KeyValueTable {
                    id: envKvt
                    width: parent.width - 32
                    json: root.cfg["launch.env"] || "{}"
                    keyPlaceholder: "VAR_NAME"
                    valuePlaceholder: "value"
                    addLabel: qsTr("Add variable")
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
            label: qsTr("Environment Sets")
            icon: "view_list"
            width: parent.width

            RowLayout {
                width: parent.width

                Text {
                    Layout.fillWidth: true
                    text: qsTr("Create and edit reusable env sets, applied or copied per-game.")
                    color: theme.textSubtle
                    font.pixelSize: 13
                    wrapMode: Text.WordWrap
                }

                M3Button {
                    text: qsTr("Manage")
                    variant: "tonal"
                    onClicked: root.manageSetsRequested("env")
                }
            }
        }

        SettingsSection {
            label: qsTr("DLL Override Sets")
            icon: "view_list"
            width: parent.width

            RowLayout {
                width: parent.width

                Text {
                    Layout.fillWidth: true
                    text: qsTr("Create and edit reusable DLL override sets, applied or copied per-game.")
                    color: theme.textSubtle
                    font.pixelSize: 13
                    wrapMode: Text.WordWrap
                }

                M3Button {
                    text: qsTr("Manage")
                    variant: "tonal"
                    onClicked: root.manageSetsRequested("dll")
                }
            }
        }

        SettingsSection {
            label: qsTr("Graphics")
            icon: "fullscreen"
            width: parent.width

            ToggleRow { fieldKey: "graphics.mangohud"; toggleLabel: "MangoHUD"; toggleDescription: qsTr("FPS overlay") }

            Row {
                width: parent.width
                spacing: 8
                M3Dropdown {
                    id: gpuDd
                    label: qsTr("GPU")
                    width: parent.width - 32
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
                    anchors.verticalCenter: gpuDd.verticalCenter
                    fieldKey: "graphics.gpu"
                }
            }
        }

        SettingsSection {
            label: "Gamescope"
            icon: "monitor"
            width: parent.width

            ToggleRow { fieldKey: "graphics.gamescope.enabled"; toggleLabel: qsTr("Enable"); toggleDescription: qsTr("run every game inside gamescope") }

            Column {
                width: parent.width
                spacing: 12
                visible: root.cfg["graphics.gamescope.enabled"] === true

                ToggleRow { fieldKey: "graphics.gamescope.fullscreen"; toggleLabel: qsTr("Fullscreen") }
                ToggleRow { fieldKey: "graphics.gamescope.borderless"; toggleLabel: qsTr("Borderless") }
                ToggleRow { fieldKey: "graphics.gamescope.integer_scaling"; toggleLabel: qsTr("Integer Scaling") }
                ToggleRow { fieldKey: "graphics.gamescope.hdr"; toggleLabel: qsTr("HDR") }

                Row {
                    width: parent.width
                    spacing: 8
                    SettingsRow {
                        label: qsTr("FPS Limit")
                        width: parent.width - 32
                        M3SpinBox {
                            id: fpsSpinBox
                            from: 0
                            to: 999
                            stepSize: 1
                            value: root.cfg["graphics.gamescope.fps"] || 0
                            zeroPlaceholder: "—"
                            onValueChanged: root.update("graphics.gamescope.fps", value)
                        }
                    }
                    ResetBadge {
                        anchors.verticalCenter: parent.verticalCenter
                        fieldKey: "graphics.gamescope.fps"
                    }
                }

                Row {
                    width: parent.width
                    spacing: 8
                    SettingsRow {
                        label: qsTr("Refresh Rate")
                        width: parent.width - 32
                        M3SpinBox {
                            id: refreshRateSpinBox
                            from: 0
                            to: 999
                            stepSize: 1
                            value: root.cfg["graphics.gamescope.refresh_rate"] || 0
                            zeroPlaceholder: "—"
                            onValueChanged: root.update("graphics.gamescope.refresh_rate", value)
                        }
                    }
                    ResetBadge {
                        anchors.verticalCenter: parent.verticalCenter
                        fieldKey: "graphics.gamescope.refresh_rate"
                    }
                }

                Row {
                    width: parent.width
                    spacing: 8
                    M3Dropdown {
                        id: filterDd
                        label: qsTr("Filter")
                        width: parent.width - 32
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
                        anchors.verticalCenter: filterDd.verticalCenter
                        fieldKey: "graphics.gamescope.filter"
                    }
                }

                Row {
                    width: parent.width
                    spacing: 8
                    visible: (root.cfg["graphics.gamescope.filter"] || "") === "fsr"
                    M3Slider {
                        id: sharpSlider
                        label: qsTr("FSR Sharpness")
                        from: 0
                        to: 20
                        stepSize: 1
                        value: root.cfg["graphics.gamescope.fsr_sharpness"] || 0
                        width: parent.width - 32
                        onMoved: (val) => root.update("graphics.gamescope.fsr_sharpness", Math.round(val))
                    }
                    ResetBadge {
                        anchors.verticalCenter: sharpSlider.verticalCenter
                        fieldKey: "graphics.gamescope.fsr_sharpness"
                    }
                }
            }
        }

        SettingsSection {
            label: qsTr("Performance")
            icon: "speed"
            width: parent.width

            ToggleRow { fieldKey: "system.gamemode"; toggleLabel: "GameMode"; toggleDescription: qsTr("Feral GameMode (gamemoderun)") }

            SettingsRow {
                label: qsTr("CPU Cores")
                description: qsTr("0 = no limit")
                width: parent.width
                contentRightMargin: 52
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
            label: qsTr("Audio")
            icon: "volume_up"
            width: parent.width

            ToggleRow { fieldKey: "system.pulse_latency"; toggleLabel: qsTr("Reduce Pulse Latency") }
        }

        SettingsSection {
            label: qsTr("Power")
            icon: "power_settings_new"
            width: parent.width

            ToggleRow { fieldKey: "system.prevent_sleep"; toggleLabel: qsTr("Prevent Sleep"); toggleDescription: qsTr("inhibit screensaver and sleep") }
        }
    }
}
