pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts

import "."
import "../lib/RunnerGrouping.js" as RG
import "../controls"

Item {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var gameModel: null
    property var openDllSets: function() {}

    // list_runners is a function call not a reactive property, bumping this forces re-evaluation
    property int runnersVersion: 0

    implicitHeight: content.height

    property string runnerType: config["runner.type"] || ""
    property bool isWine: runnerType === "" || runnerType === "wine"
    property bool isProtonWine: isProtonVersion(config["wine.version"] || "")

    function isProtonVersion(version) {
        return gameModel.runner_is_proton(String(version || ""))
    }

    component DllVersionPicker: M3Dropdown {
        required property string kind
        required property string fieldKey
        required property var gameModel
        required property var config
        required property var apply
        width: parent.width
        readonly property var _versions: gameModel ? (JSON.parse(gameModel.dll_versions_for_kind(kind)) || []) : []
        options: [{ label: qsTr("Default (global)"), value: "" }].concat(_versions.map(v => ({ label: v, value: v })))
        currentIndex: Math.max(0, options.findIndex(o => o.value === (config[fieldKey] || "")))
        onSelected: (v) => apply(fieldKey, v)
    }

    Column {
        id: content
        width: parent.width
        spacing: 20

        Column {
            width: parent.width
            spacing: 20
            visible: root.isWine

            SettingsSection {
                label: qsTr("Executable")
                icon: "terminal"
                width: parent.width

                M3FileField {
                    label: qsTr("Path")
                    text: root.config["meta.exe"] || ""
                    width: parent.width
                    gameModel: root.gameModel
                    expandHint: false
                    onTextEdited: (t) => root.updateField("meta.exe", t)
                }

                M3FileField {
                    label: qsTr("Working Directory")
                    placeholder: qsTr("empty = executable's parent directory")
                    text: root.config["launch.working_dir"] || ""
                    selectFolder: true
                    width: parent.width
                    gameModel: root.gameModel
                    onTextEdited: (t) => root.updateField("launch.working_dir", t)
                }

                M3TextField {
                    label: qsTr("Arguments")
                    placeholder: '--skip-intro --windowed --name "John Doe"'
                    text: root.config["launch.args"] || ""
                    width: parent.width
                    gameModel: root.gameModel
                    onTextEdited: (t) => root.updateField("launch.args", t)
                }

                M3TextField {
                    label: qsTr("Command Prefix")
                    placeholder: qsTr("prepended to command (e.g. custom wrapper)")
                    text: root.config["launch.command_prefix"] || ""
                    width: parent.width
                    onTextEdited: (t) => root.updateField("launch.command_prefix", t)
                }
            }

            SettingsSection {
                label: "Wine"
                icon: "wine_bar"
                width: parent.width

                M3Dropdown {
                    label: qsTr("Version")
                    width: parent.width
                    options: {
                        // touch runnersVersion so QML re-evaluates the binding after install/delete
                        void root.runnersVersion
                        if (!root.gameModel) return [{ label: qsTr("Loading..."), value: "" }]
                        return RG.groupRunners(JSON.parse(root.gameModel.list_runners()))
                    }
                    currentIndex: {
                        void root.runnersVersion
                        let v = root.config["wine.version"] || ""
                        let idx = RG.indexOfValue(options, v)
                        if (idx >= 0) return idx
                        let first = RG.firstNonHeader(options)
                        return first >= 0 ? first : 0
                    }
                    onSelected: (val) => root.updateField("wine.version", val)
                }

                M3FileField {
                    label: qsTr("Prefix")
                    placeholder: root.config["wine.prefix.resolved"] || qsTr("empty = auto-create per game")
                    text: root.config["wine.prefix"] || ""
                    selectFolder: true
                    width: parent.width
                    gameModel: root.gameModel
                    onTextEdited: (t) => root.updateField("wine.prefix", t)
                }

                M3Dropdown {
                    label: qsTr("Architecture")
                    width: parent.width
                    options: [
                        { label: qsTr("64-bit (win64)"), value: "win64" },
                        { label: qsTr("32-bit (win32)"), value: "win32" }
                    ]
                    currentIndex: root.config["wine.prefix_arch"] === "win32" ? 1 : 0
                    onSelected: (val) => root.updateField("wine.prefix_arch", val)
                }
            }

            SettingsSection {
                label: qsTr("Sync")
                icon: "sync"
                width: parent.width

                GridLayout {
                    columns: 2
                    columnSpacing: 96
                    rowSpacing: 12

                    LabeledSwitch {
                        label: qsTr("Esync")
                        checked: root.config["wine.esync"] === true
                        onToggled: (val) => root.updateField("wine.esync", val)
                    }

                    LabeledSwitch {                        label: qsTr("Fsync")
                        checked: root.config["wine.fsync"] === true
                        onToggled: (val) => root.updateField("wine.fsync", val)
                    }

                    LabeledSwitch {
                        label: qsTr("NTSync")
                        enabled: root.isProtonWine
                        checked: root.config["wine.ntsync"] === true
                        onToggled: (val) => root.updateField("wine.ntsync", val)
                    }

                    Text {
                        text: qsTr("NTSync is only applied when the selected Wine version is Proton.")
                        color: Theme.textSubtle
                        font.pixelSize: Theme.type.label.size
                        visible: !root.isProtonWine
                        Layout.columnSpan: 2
                        wrapMode: Text.WordWrap
                    }
                }
            }

            SettingsSection {
                label: qsTr("Translation Layers")
                icon: "layers"
                width: parent.width

                GridLayout {
                    columns: 2
                    columnSpacing: 96
                    rowSpacing: 12

                    LabeledSwitch {
                        label: "DXVK"
                        checked: root.config["wine.dxvk"] === true
                        onToggled: (val) => root.updateField("wine.dxvk", val)
                    }

                    LabeledSwitch {                        label: "VKD3D"
                        checked: root.config["wine.vkd3d"] === true
                        onToggled: (val) => root.updateField("wine.vkd3d", val)
                    }

                    LabeledSwitch {
                        label: qsTr("D3D Extras")
                        checked: root.config["wine.d3d_extras"] === true
                        onToggled: (val) => root.updateField("wine.d3d_extras", val)
                    }

                    LabeledSwitch {                        label: "DXVK-NVAPI"
                        checked: root.config["wine.dxvk_nvapi"] === true
                        onToggled: (val) => root.updateField("wine.dxvk_nvapi", val)
                    }
                }

                DllVersionPicker {
                    kind: "dxvk"
                    fieldKey: "wine.dxvk_version"
                    label: qsTr("DXVK version")
                    gameModel: root.gameModel
                    config: root.config
                    apply: root.updateField
                    visible: root.config["wine.dxvk"] === true
                }

                DllVersionPicker {
                    kind: "vkd3d"
                    fieldKey: "wine.vkd3d_version"
                    label: qsTr("VKD3D version")
                    gameModel: root.gameModel
                    config: root.config
                    apply: root.updateField
                    visible: root.config["wine.vkd3d"] === true
                }

                DllVersionPicker {
                    kind: "dxvk_nvapi"
                    fieldKey: "wine.dxvk_nvapi_version"
                    label: qsTr("DXVK-NVAPI version")
                    gameModel: root.gameModel
                    config: root.config
                    apply: root.updateField
                    visible: root.config["wine.dxvk_nvapi"] === true
                }

                Text {
                    width: parent.width
                    visible: root.isProtonWine && (root.config["wine.dxvk"] === true || root.config["wine.vkd3d"] === true || root.config["wine.dxvk_nvapi"] === true)
                    text: qsTr("These may not work with Proton. Tsk.")
                    color: Theme.warning
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.WordWrap
                }
            }

            SettingsSection {
                label: qsTr("Compatibility")
                icon: "verified"
                width: parent.width

                GridLayout {
                    columns: 2
                    columnSpacing: 96
                    rowSpacing: 12

                    LabeledSwitch {
                        label: "BattlEye"
                        checked: root.config["wine.battleye"] === true
                        onToggled: (val) => root.updateField("wine.battleye", val)
                    }

                    LabeledSwitch {                        label: "EasyAntiCheat"
                        checked: root.config["wine.easyanticheat"] === true
                        onToggled: (val) => root.updateField("wine.easyanticheat", val)
                    }

                    LabeledSwitch {
                        label: "FSR"
                        checked: root.config["wine.fsr"] === true
                        onToggled: (val) => root.updateField("wine.fsr", val)
                    }
                }
            }

            SettingsSection {
                label: qsTr("Display")
                icon: "desktop_windows"
                width: parent.width

                LabeledSwitch {
                    label: qsTr("DPI Scaling")
                    checked: root.config["wine.dpi_scaling"] === true
                    onToggled: (val) => root.updateField("wine.dpi_scaling", val)
                }

                M3Slider {
                    label: qsTr("DPI")
                    from: 72
                    to: 288
                    stepSize: 12
                    value: root.config["wine.dpi"] || 96
                    width: parent.width
                    visible: root.config["wine.dpi_scaling"] === true
                    onMoved: (val) => root.updateField("wine.dpi", Math.round(val))
                }
            }

            SettingsSection {
                label: qsTr("Drivers")
                icon: "headphones"
                width: parent.width

                M3Dropdown {
                    label: qsTr("Audio Driver")
                    width: parent.width
                    options: [
                        { label: qsTr("Default"), value: "" },
                        { label: "PulseAudio", value: "pulse" },
                        { label: "ALSA", value: "alsa" }
                    ]
                    currentIndex: {
                        let d = root.config["wine.audio_driver"] || ""
                        if (d === "pulse") return 1
                        if (d === "alsa") return 2
                        return 0
                    }
                    onSelected: (val) => root.updateField("wine.audio_driver", val)
                }

                M3Dropdown {
                    label: qsTr("Graphics Driver")
                    width: parent.width
                    options: [
                        { label: qsTr("Default"), value: "" },
                        { label: "X11", value: "x11" },
                        { label: "Wayland", value: "wayland" }
                    ]
                    currentIndex: {
                        let d = root.config["wine.graphics_driver"] || ""
                        if (d === "x11") return 1
                        if (d === "wayland") return 2
                        return 0
                    }
                    onSelected: (val) => root.updateField("wine.graphics_driver", val)
                }
            }

            SettingsSection {
                label: qsTr("DLL Overrides")
                icon: "build"
                width: parent.width

                KeyValueTable {
                    width: parent.width
                    json: root.config["wine.dll_overrides"] || "{}"
                    keyPlaceholder: "dll_name"
                    valuePlaceholder: "n,b"
                    addLabel: qsTr("Add override")
                    onChanged: (j) => root.updateField("wine.dll_overrides", j)
                }

                M3Button {
                    text: {
                        let n = 0
                        try { n = JSON.parse(root.config["wine.dll_override_sets"] || "[]").length } catch (e) {}
                        return n > 0 ? qsTr("Sets · %1 synced").arg(n) : qsTr("Sets")
                    }
                    variant: "tonal"
                    icon: "view_list"
                    onClicked: root.openDllSets()
                }
            }
        }

        // the more i add the more i aks myself why im doing this. Electron was the real answer all along...
        SettingsSection {
            label: qsTr("Native")
            icon: "terminal"
            width: parent.width
            visible: root.runnerType === "native"

            M3FileField {
                label: qsTr("Executable")
                text: root.config["meta.exe"] || ""
                width: parent.width
                gameModel: root.gameModel
                expandHint: false
                onTextEdited: (t) => root.updateField("meta.exe", t)
            }

            M3FileField {
                label: qsTr("Working Directory")
                placeholder: qsTr("empty = executable's parent directory")
                text: root.config["launch.working_dir"] || ""
                selectFolder: true
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => root.updateField("launch.working_dir", t)
            }

            M3TextField {
                label: qsTr("Arguments")
                placeholder: '--skip-intro --windowed'
                text: root.config["launch.args"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => root.updateField("launch.args", t)
            }

            M3TextField {
                label: qsTr("Command Prefix")
                placeholder: qsTr("prepended to command (e.g. custom wrapper)")
                text: root.config["launch.command_prefix"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("launch.command_prefix", t)
            }
        }

        SettingsSection {
            label: "Steam"
            icon: "steam"
            width: parent.width
            visible: root.runnerType === "steam"

            M3TextField {
                label: qsTr("Application ID")
                placeholder: "e.g. 235320"
                text: root.config["source.app_id"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("source.app_id", t)
            }

            M3TextField {
                label: qsTr("Arguments")
                placeholder: '--skip-intro --windowed'
                text: root.config["launch.args"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("launch.args", t)
            }
        }

        SettingsSection {
            label: "Flatpak"
            icon: "sports_esports"
            width: parent.width
            visible: root.runnerType === "flatpak"

            M3TextField {
                label: qsTr("Application ID")
                placeholder: "e.g. com.valvesoftware.Steam"
                text: root.config["source.app_id"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("source.app_id", t)
            }

            M3TextField {
                label: qsTr("Arguments")
                placeholder: qsTr("passed to the application")
                text: root.config["launch.args"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("launch.args", t)
            }
        }
    }
}
