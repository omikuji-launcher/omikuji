import QtQuick
import QtQuick.Layouts

import "."
import "../widgets"
import "../widgets/RunnerGrouping.js" as RG

Item {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var gameModel: null

    // list_runners is a function call not a reactive property, bumping this forces re-evaluation
    property int runnersVersion: 0

    implicitHeight: content.height

    property string runnerType: config["runner.type"] || ""
    property bool isWine: runnerType === "" || runnerType === "wine"
    property bool isProtonWine: isProtonVersion(config["wine.version"] || "")

    function isProtonVersion(version) {
        let v = String(version || "").toLowerCase()
        return v.indexOf("proton") !== -1
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
                label: "Executable"
                icon: "terminal"
                width: parent.width

                M3FileField {
                    label: "Path"
                    text: config["meta.exe"] || ""
                    width: parent.width
                    gameModel: root.gameModel
                    onTextEdited: (t) => updateField("meta.exe", t)
                }

                M3FileField {
                    label: "Working Directory"
                    placeholder: "empty = executable's parent directory"
                    text: config["launch.working_dir"] || ""
                    selectFolder: true
                    width: parent.width
                    gameModel: root.gameModel
                    onTextEdited: (t) => updateField("launch.working_dir", t)
                }

                M3TextField {
                    label: "Arguments"
                    placeholder: '--skip-intro --windowed --name "John Doe"'
                    text: config["launch.args"] || ""
                    width: parent.width
                    onTextEdited: (t) => updateField("launch.args", t)
                }

                M3TextField {
                    label: "Command Prefix"
                    placeholder: "prepended to command (e.g. custom wrapper)"
                    text: config["launch.command_prefix"] || ""
                    width: parent.width
                    onTextEdited: (t) => updateField("launch.command_prefix", t)
                }
            }

            SettingsSection {
                label: "Wine"
                icon: "wine_bar"
                width: parent.width

                M3Dropdown {
                    label: "Version"
                    width: parent.width
                    options: {
                        // touch runnersVersion so QML re-evaluates the binding after install/delete
                        void root.runnersVersion
                        if (!gameModel) return [{ label: "Loading...", value: "" }]
                        return RG.groupRunners(JSON.parse(gameModel.list_runners()))
                    }
                    currentIndex: {
                        void root.runnersVersion
                        let v = config["wine.version"] || ""
                        let idx = RG.indexOfValue(options, v)
                        if (idx >= 0) return idx
                        let first = RG.firstNonHeader(options)
                        return first >= 0 ? first : 0
                    }
                    onSelected: (val) => updateField("wine.version", val)
                }

                M3FileField {
                    label: "Prefix"
                    placeholder: "empty = auto-create per game"
                    text: config["wine.prefix"] || ""
                    selectFolder: true
                    width: parent.width
                    gameModel: root.gameModel
                    onTextEdited: (t) => updateField("wine.prefix", t)
                }

                M3Dropdown {
                    label: "Architecture"
                    width: parent.width
                    options: [
                        { label: "64-bit (win64)", value: "win64" },
                        { label: "32-bit (win32)", value: "win32" }
                    ]
                    currentIndex: config["wine.prefix_arch"] === "win32" ? 1 : 0
                    onSelected: (val) => updateField("wine.prefix_arch", val)
                }
            }

            SettingsSection {
                label: "Sync"
                icon: "sync"
                width: parent.width

                GridLayout {
                    columns: 2
                    columnSpacing: 96
                    rowSpacing: 12

                    LabeledSwitch {
                        label: "Esync"
                        checked: config["wine.esync"] === true
                        onToggled: (val) => updateField("wine.esync", val)
                    }

                    LabeledSwitch {                        label: "Fsync"
                        checked: config["wine.fsync"] === true
                        onToggled: (val) => updateField("wine.fsync", val)
                    }

                    LabeledSwitch {
                        label: "NTSync"
                        enabled: root.isProtonWine
                        checked: config["wine.ntsync"] === true
                        onToggled: (val) => updateField("wine.ntsync", val)
                    }

                    Text {
                        text: "NTSync is only applied when the selected Wine version is Proton."
                        color: theme.textSubtle
                        font.pixelSize: 13
                        visible: !root.isProtonWine
                        Layout.columnSpan: 2
                        wrapMode: Text.WordWrap
                    }
                }
            }

            SettingsSection {
                label: "Translation Layers"
                icon: "layers"
                width: parent.width

                GridLayout {
                    columns: 2
                    columnSpacing: 96
                    rowSpacing: 12

                    LabeledSwitch {
                        label: "DXVK"
                        checked: config["wine.dxvk"] === true
                        onToggled: (val) => updateField("wine.dxvk", val)
                    }

                    LabeledSwitch {                        label: "VKD3D"
                        checked: config["wine.vkd3d"] === true
                        onToggled: (val) => updateField("wine.vkd3d", val)
                    }

                    LabeledSwitch {
                        label: "D3D Extras"
                        checked: config["wine.d3d_extras"] === true
                        onToggled: (val) => updateField("wine.d3d_extras", val)
                    }

                    LabeledSwitch {                        label: "DXVK-NVAPI"
                        checked: config["wine.dxvk_nvapi"] === true
                        onToggled: (val) => updateField("wine.dxvk_nvapi", val)
                    }
                }
            }

            SettingsSection {
                label: "Compatibility"
                icon: "verified"
                width: parent.width

                GridLayout {
                    columns: 2
                    columnSpacing: 96
                    rowSpacing: 12

                    LabeledSwitch {
                        label: "BattlEye"
                        checked: config["wine.battleye"] === true
                        onToggled: (val) => updateField("wine.battleye", val)
                    }

                    LabeledSwitch {                        label: "EasyAntiCheat"
                        checked: config["wine.easyanticheat"] === true
                        onToggled: (val) => updateField("wine.easyanticheat", val)
                    }

                    LabeledSwitch {
                        label: "FSR"
                        checked: config["wine.fsr"] === true
                        onToggled: (val) => updateField("wine.fsr", val)
                    }
                }
            }

            SettingsSection {
                label: "Display"
                icon: "desktop_windows"
                width: parent.width

                LabeledSwitch {
                    label: "DPI Scaling"
                    checked: config["wine.dpi_scaling"] === true
                    onToggled: (val) => updateField("wine.dpi_scaling", val)
                }

                M3Slider {
                    label: "DPI"
                    from: 72
                    to: 288
                    stepSize: 12
                    value: config["wine.dpi"] || 96
                    width: parent.width
                    visible: config["wine.dpi_scaling"] === true
                    onMoved: (val) => updateField("wine.dpi", Math.round(val))
                }
            }

            SettingsSection {
                label: "Drivers"
                icon: "headphones"
                width: parent.width

                M3Dropdown {
                    label: "Audio Driver"
                    width: parent.width
                    options: [
                        { label: "Default", value: "" },
                        { label: "PulseAudio", value: "pulse" },
                        { label: "ALSA", value: "alsa" }
                    ]
                    currentIndex: {
                        let d = config["wine.audio_driver"] || ""
                        if (d === "pulse") return 1
                        if (d === "alsa") return 2
                        return 0
                    }
                    onSelected: (val) => updateField("wine.audio_driver", val)
                }

                M3Dropdown {
                    label: "Graphics Driver"
                    width: parent.width
                    options: [
                        { label: "Default", value: "" },
                        { label: "X11", value: "x11" },
                        { label: "Wayland", value: "wayland" }
                    ]
                    currentIndex: {
                        let d = config["wine.graphics_driver"] || ""
                        if (d === "x11") return 1
                        if (d === "wayland") return 2
                        return 0
                    }
                    onSelected: (val) => updateField("wine.graphics_driver", val)
                }
            }

            SettingsSection {
                label: "DLL Overrides"
                icon: "build"
                width: parent.width

                KeyValueTable {
                    width: parent.width
                    json: config["wine.dll_overrides"] || "{}"
                    keyPlaceholder: "dll_name"
                    valuePlaceholder: "n,b"
                    addLabel: "Add override"
                    onChanged: (j) => updateField("wine.dll_overrides", j)
                }
            }
        }

        // the more i add the more i aks myself why im doing this. Electron was the real answer all along...
        SettingsSection {
            label: "Native"
            icon: "terminal"
            width: parent.width
            visible: runnerType === "native"

            M3FileField {
                label: "Executable"
                text: config["meta.exe"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("meta.exe", t)
            }

            M3FileField {
                label: "Working Directory"
                placeholder: "empty = executable's parent directory"
                text: config["launch.working_dir"] || ""
                selectFolder: true
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("launch.working_dir", t)
            }

            M3TextField {
                label: "Arguments"
                placeholder: '--skip-intro --windowed'
                text: config["launch.args"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("launch.args", t)
            }

            M3TextField {
                label: "Command Prefix"
                placeholder: "prepended to command (e.g. custom wrapper)"
                text: config["launch.command_prefix"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("launch.command_prefix", t)
            }
        }

        SettingsSection {
            label: "Steam"
            icon: "steam"
            width: parent.width
            visible: runnerType === "steam"

            M3TextField {
                label: "Application ID"
                placeholder: "e.g. 235320"
                text: config["source.app_id"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("source.app_id", t)
            }

            M3TextField {
                label: "Arguments"
                placeholder: '--skip-intro --windowed'
                text: config["launch.args"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("launch.args", t)
            }
        }

        SettingsSection {
            label: "Flatpak"
            icon: "sports_esports"
            width: parent.width
            visible: runnerType === "flatpak"

            M3TextField {
                label: "Application ID"
                placeholder: "e.g. com.valvesoftware.Steam"
                text: config["source.app_id"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("source.app_id", t)
            }

            M3TextField {
                label: "Arguments"
                placeholder: "passed to the application"
                text: config["launch.args"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("launch.args", t)
            }
        }
    }
}
