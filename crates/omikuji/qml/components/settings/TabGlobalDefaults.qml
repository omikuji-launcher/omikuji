import QtQuick
import QtQuick.Layouts

import "."
import "../widgets"

Item {
    id: root

    property var defaults: null
    property var gameModel: null

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
                ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.10)
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
        spacing: 20

        Item {
            width: parent.width
            height: 32

            Item {
                id: applyBtn
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: applyRow.implicitWidth + 24
                height: 32

                Rectangle {
                    anchors.fill: parent
                    radius: 8
                    color: applyMouse.containsPress
                        ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.14)
                        : (applyMouse.containsMouse
                            ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.08)
                            : "transparent")
                    border.width: 1
                    border.color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.18)
                    Behavior on color { ColorAnimation { duration: 100 } }
                }

                Row {
                    id: applyRow
                    anchors.centerIn: parent
                    spacing: 6

                    SvgIcon {
                        name: "sync"
                        size: 14
                        color: theme.text
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Text {
                        text: "Apply to existing games"
                        color: theme.text
                        font.pixelSize: 12
                        font.weight: Font.Medium
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }

                MouseArea {
                    id: applyMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.applyToExistingRequested()
                }
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
                    label: "Version"
                    width: parent.width - 32
                    options: {
                        if (!root.gameModel) return [{ label: "System default", value: "" }]
                        let base = [{ label: "System default", value: "" }]
                        let runners = JSON.parse(root.gameModel.list_runners())
                        return base.concat(runners.map(v => {
                            let label = v.startsWith("steam:") ? v.substring(6) + " (steam)" : v
                            return { label: label, value: v }
                        }))
                    }
                    currentIndex: {
                        let v = root.cfg["wine.version"] || ""
                        if (!root.gameModel) return 0
                        let runners = JSON.parse(root.gameModel.list_runners())
                        let idx = runners.indexOf(v)
                        return idx >= 0 ? idx + 1 : 0
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
                    label: "Prefix"
                    placeholder: "empty = auto-create per game"
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
                    label: "Architecture"
                    width: parent.width - 32
                    options: [
                        { label: "64-bit (win64)", value: "win64" },
                        { label: "32-bit (win32)", value: "win32" }
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
            label: "Sync"
            icon: "sync"
            width: parent.width

            ToggleRow { fieldKey: "wine.esync"; toggleLabel: "Esync" }
            ToggleRow { fieldKey: "wine.fsync"; toggleLabel: "Fsync" }
            ToggleRow {
                fieldKey: "wine.ntsync"
                toggleLabel: "NTSync"
                toggleDescription: !root.isProtonWine
                    ? "Only applied when the default Wine version is Proton"
                    : ""
                toggleEnabled: root.isProtonWine
            }
        }

        SettingsSection {
            label: "Translation Layers"
            icon: "layers"
            width: parent.width

            ToggleRow { fieldKey: "wine.dxvk"; toggleLabel: "DXVK" }
            ToggleRow { fieldKey: "wine.vkd3d"; toggleLabel: "VKD3D" }
            ToggleRow { fieldKey: "wine.d3d_extras"; toggleLabel: "D3D Extras" }
            ToggleRow { fieldKey: "wine.dxvk_nvapi"; toggleLabel: "DXVK-NVAPI"; toggleDescription: "Nvidia DLSS support" }
        }

        SettingsSection {
            label: "Compatibility"
            icon: "verified"
            width: parent.width

            ToggleRow { fieldKey: "wine.battleye"; toggleLabel: "BattlEye" }
            ToggleRow { fieldKey: "wine.easyanticheat"; toggleLabel: "EasyAntiCheat" }
            ToggleRow { fieldKey: "wine.fsr"; toggleLabel: "FSR"; toggleDescription: "AMD FidelityFX Super Resolution" }
        }

        SettingsSection {
            label: "Display"
            icon: "desktop_windows"
            width: parent.width

            ToggleRow { fieldKey: "wine.dpi_scaling"; toggleLabel: "DPI Scaling" }

            Row {
                width: parent.width
                spacing: 8
                visible: root.cfg["wine.dpi_scaling"] === true
                M3Slider {
                    id: dpiSlider
                    label: "DPI"
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
            label: "Drivers"
            icon: "headphones"
            width: parent.width

            Row {
                width: parent.width
                spacing: 8
                M3Dropdown {
                    id: audioDd
                    label: "Audio Driver"
                    width: parent.width - 32
                    options: [
                        { label: "Default", value: "" },
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
                    label: "Graphics Driver"
                    width: parent.width - 32
                    options: [
                        { label: "Default", value: "" },
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
            label: "DLL Overrides"
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
                    addLabel: "Add override"
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
            label: "Launch"
            icon: "terminal"
            width: parent.width

            Row {
                width: parent.width
                spacing: 8
                M3TextField {
                    id: cmdPrefixTf
                    label: "Command Prefix"
                    placeholder: "prepended to every game's command"
                    text: root.cfg["launch.command_prefix"] || ""
                    width: parent.width - 32
                    onTextEdited: root.update("launch.command_prefix", text)
                }
                ResetBadge {
                    anchors.verticalCenter: cmdPrefixTf.verticalCenter
                    fieldKey: "launch.command_prefix"
                }
            }
        }

        SettingsSection {
            label: "Environment"
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
                    addLabel: "Add variable"
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
            label: "Graphics"
            icon: "fullscreen"
            width: parent.width

            ToggleRow { fieldKey: "graphics.mangohud"; toggleLabel: "MangoHUD"; toggleDescription: "FPS overlay" }

            Row {
                width: parent.width
                spacing: 8
                M3Dropdown {
                    id: gpuDd
                    label: "GPU"
                    width: parent.width - 32
                    options: root.gameModel ? JSON.parse(root.gameModel.list_gpus()).map(g => ({ label: g[0], value: g[1] })) : [{ label: "Default", value: "" }]
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

            ToggleRow { fieldKey: "graphics.gamescope.enabled"; toggleLabel: "Enable"; toggleDescription: "run every game inside gamescope" }

            Column {
                width: parent.width
                spacing: 12
                visible: root.cfg["graphics.gamescope.enabled"] === true

                ToggleRow { fieldKey: "graphics.gamescope.fullscreen"; toggleLabel: "Fullscreen" }
                ToggleRow { fieldKey: "graphics.gamescope.borderless"; toggleLabel: "Borderless" }
                ToggleRow { fieldKey: "graphics.gamescope.integer_scaling"; toggleLabel: "Integer Scaling" }
                ToggleRow { fieldKey: "graphics.gamescope.hdr"; toggleLabel: "HDR" }

                Row {
                    width: parent.width
                    spacing: 8
                    M3Slider {
                        id: fpsSlider
                        label: "FPS Limit"
                        from: 0
                        to: 360
                        stepSize: 5
                        value: root.cfg["graphics.gamescope.fps"] || 0
                        width: parent.width - 32
                        onMoved: (val) => root.update("graphics.gamescope.fps", Math.round(val))
                    }
                    ResetBadge {
                        anchors.verticalCenter: fpsSlider.verticalCenter
                        fieldKey: "graphics.gamescope.fps"
                    }
                }

                Row {
                    width: parent.width
                    spacing: 8
                    M3Dropdown {
                        id: filterDd
                        label: "Filter"
                        width: parent.width - 32
                        options: [
                            { label: "None", value: "" },
                            { label: "Nearest", value: "nearest" },
                            { label: "Linear", value: "linear" },
                            { label: "FSR", value: "fsr" },
                            { label: "NIS", value: "nis" },
                            { label: "Pixel", value: "pixel" }
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
                        label: "FSR Sharpness"
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
            label: "Performance"
            icon: "speed"
            width: parent.width

            ToggleRow { fieldKey: "system.gamemode"; toggleLabel: "GameMode"; toggleDescription: "Feral GameMode (gamemoderun)" }

            SettingsRow {
                label: "CPU Cores"
                description: "0 = no limit"
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
            label: "Audio"
            icon: "volume_up"
            width: parent.width

            ToggleRow { fieldKey: "system.pulse_latency"; toggleLabel: "Reduce Pulse Latency" }
        }

        SettingsSection {
            label: "Power"
            icon: "power_settings_new"
            width: parent.width

            ToggleRow { fieldKey: "system.prevent_sleep"; toggleLabel: "Prevent Sleep"; toggleDescription: "inhibit screensaver and sleep" }
        }
    }
}
