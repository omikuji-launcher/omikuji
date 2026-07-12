import QtQuick
import QtQuick.Layouts

import "."
import "../controls"

Item {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var gameModel: null
    property var openEnvSets: function() {}

    implicitHeight: content.height

    Column {
        id: content
        width: parent.width
        spacing: 20

        SettingsSection {
            label: qsTr("Performance")
            icon: "speed"
            width: parent.width

            SettingsRow {
                label: "GameMode"
                description: qsTr("Feral GameMode (gamemoderun)")
                width: parent.width
                M3Switch {
                    checked: config["system.gamemode"] === true
                    onToggled: (val) => updateField("system.gamemode", val)
                }
            }

            SettingsRow {
                label: qsTr("CPU Cores")
                description: qsTr("0 = no limit")
                width: parent.width
                contentRightMargin: 74
                M3SpinBox {
                    from: 0
                    to: gameModel ? Math.max(1, gameModel.cpuCoreCount()) : 1
                    stepSize: 1
                    value: config["system.cpu_limit"] || 0
                    zeroPlaceholder: "—"
                    onMoved: (val) => updateField("system.cpu_limit", val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Display")
            icon: "fullscreen"
            width: parent.width

            SettingsRow {
                label: "MangoHUD"
                description: qsTr("FPS overlay")
                width: parent.width
                M3Switch {
                    checked: config["graphics.mangohud"] === true
                    onToggled: (val) => updateField("graphics.mangohud", val)
                }
            }

            M3Dropdown {
                label: qsTr("GPU")
                width: parent.width
                options: gameModel ? JSON.parse(gameModel.list_gpus()).map(g => ({ label: g[0], value: g[1] })) : [{ label: qsTr("Default"), value: "" }]
                currentIndex: {
                    let v = config["graphics.gpu"] || ""
                    let gpus = gameModel ? JSON.parse(gameModel.list_gpus()) : []
                    let idx = gpus.findIndex(g => g[1] === v)
                    return idx >= 0 ? idx : 0
                }
                onSelected: (val) => updateField("graphics.gpu", val)
            }
        }

        SettingsSection {
            label: "Gamescope"
            icon: "monitor"
            width: parent.width

            SettingsRow {
                label: qsTr("Enable Gamescope")
                description: qsTr("run game inside gamescope compositor")
                width: parent.width
                M3Switch {
                    checked: config["graphics.gamescope.enabled"] === true
                    onToggled: (val) => updateField("graphics.gamescope.enabled", val)
                }
            }

            Column {
                width: parent.width
                spacing: 12
                visible: config["graphics.gamescope.enabled"] === true

                Row {
                    spacing: 12
                    width: parent.width

                    M3TextField {
                        label: qsTr("Output Width")
                        placeholder: qsTr("0 = native")
                        text: (config["graphics.gamescope.width"] || 0) > 0
                            ? String(config["graphics.gamescope.width"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: (t) => updateField("graphics.gamescope.width", t || "0")
                    }

                    M3TextField {
                        label: qsTr("Output Height")
                        placeholder: qsTr("0 = native")
                        text: (config["graphics.gamescope.height"] || 0) > 0
                            ? String(config["graphics.gamescope.height"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: (t) => updateField("graphics.gamescope.height", t || "0")
                    }
                }

                Row {
                    spacing: 12
                    width: parent.width

                    M3TextField {
                        label: qsTr("Game Width")
                        placeholder: qsTr("0 = output")
                        text: (config["graphics.gamescope.game_width"] || 0) > 0
                            ? String(config["graphics.gamescope.game_width"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: (t) => updateField("graphics.gamescope.game_width", t || "0")
                    }

                    M3TextField {
                        label: qsTr("Game Height")
                        placeholder: qsTr("0 = output")
                        text: (config["graphics.gamescope.game_height"] || 0) > 0
                            ? String(config["graphics.gamescope.game_height"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: (t) => updateField("graphics.gamescope.game_height", t || "0")
                    }
                }

                SettingsRow {
                    label: qsTr("FPS Limit")
                    width: parent.width
                    contentRightMargin: 74
                    M3SpinBox {
                        from: 0
                        to: 999
                        stepSize: 1
                        value: config["graphics.gamescope.fps"] || 0
                        zeroPlaceholder: "—"
                        onValueChanged: updateField("graphics.gamescope.fps", value)
                    }
                }

                SettingsRow {
                    label: qsTr("Refresh Rate")
                    width: parent.width
                    contentRightMargin: 74
                    M3SpinBox {
                        from: 0
                        to: 999
                        stepSize: 1
                        value: config["graphics.gamescope.refresh_rate"] || 0
                        zeroPlaceholder: "—"
                        onValueChanged: updateField("graphics.gamescope.refresh_rate", value)
                    }
                }

                SettingsRow {
                    label: qsTr("Fullscreen")
                    width: parent.width
                    M3Switch {
                        checked: config["graphics.gamescope.fullscreen"] === true
                        onToggled: (val) => updateField("graphics.gamescope.fullscreen", val)
                    }
                }

                SettingsRow {
                    label: qsTr("Borderless")
                    width: parent.width
                    M3Switch {
                        checked: config["graphics.gamescope.borderless"] === true
                        onToggled: (val) => updateField("graphics.gamescope.borderless", val)
                    }
                }

                SettingsRow {
                    label: qsTr("Integer Scaling")
                    width: parent.width
                    M3Switch {
                        checked: config["graphics.gamescope.integer_scaling"] === true
                        onToggled: (val) => updateField("graphics.gamescope.integer_scaling", val)
                    }
                }

                SettingsRow {
                    label: qsTr("HDR")
                    width: parent.width
                    M3Switch {
                        checked: config["graphics.gamescope.hdr"] === true
                        onToggled: (val) => updateField("graphics.gamescope.hdr", val)
                    }
                }

                M3Dropdown {
                    label: qsTr("Filter")
                    width: parent.width
                    options: [
                        { label: qsTr("None"), value: "" },
                        { label: qsTr("Nearest"), value: "nearest" },
                        { label: qsTr("Linear"), value: "linear" },
                        { label: "FSR", value: "fsr" },
                        { label: "NIS", value: "nis" },
                        { label: qsTr("Pixel"), value: "pixel" }
                    ]
                    currentIndex: {
                        let f = config["graphics.gamescope.filter"] || ""
                        let idx = ["", "nearest", "linear", "fsr", "nis", "pixel"].indexOf(f)
                        return idx >= 0 ? idx : 0
                    }
                    onSelected: (val) => updateField("graphics.gamescope.filter", val)
                }

                M3Slider {
                    label: qsTr("FSR Sharpness")
                    from: 0
                    to: 20
                    stepSize: 1
                    value: config["graphics.gamescope.fsr_sharpness"] || 0
                    width: parent.width
                    visible: (config["graphics.gamescope.filter"] || "") === "fsr"
                    onMoved: (val) => updateField("graphics.gamescope.fsr_sharpness", Math.round(val))
                }
            }
        }

        SettingsSection {
            label: qsTr("Audio")
            icon: "volume_up"
            width: parent.width

            SettingsRow {
                label: qsTr("Reduce Pulse Latency")
                width: parent.width
                M3Switch {
                    checked: config["system.pulse_latency"] === true
                    onToggled: (val) => updateField("system.pulse_latency", val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Power")
            icon: "power_settings_new"
            width: parent.width

            SettingsRow {
                label: qsTr("Prevent Sleep")
                description: qsTr("inhibit screensaver and sleep")
                width: parent.width
                M3Switch {
                    checked: config["system.prevent_sleep"] === true
                    onToggled: (val) => updateField("system.prevent_sleep", val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Environment")
            icon: "tune"
            width: parent.width

            KeyValueTable {
                width: parent.width
                json: config["launch.env"] || "{}"
                keyPlaceholder: "VAR_NAME"
                valuePlaceholder: "value"
                addLabel: qsTr("Add variable")
                onChanged: (j) => updateField("launch.env", j)
            }

            M3Button {
                text: {
                    let n = 0
                    try { n = JSON.parse(config["launch.env_sets"] || "[]").length } catch (e) {}
                    return n > 0 ? qsTr("Sets · %1 synced").arg(n) : qsTr("Sets")
                }
                variant: "tonal"
                icon: "view_list"
                onClicked: root.openEnvSets()
            }
        }

        SettingsSection {
            label: qsTr("Scripts")
            icon: "code"
            width: parent.width

            M3FileField {
                label: qsTr("Pre-Launch Script")
                placeholder: qsTr("runs before game starts")
                text: config["launch.pre_launch_script"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("launch.pre_launch_script", t)
            }

            M3FileField {
                label: qsTr("Post-Exit Script")
                placeholder: qsTr("runs after game exits")
                text: config["launch.post_exit_script"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("launch.post_exit_script", t)
            }
        }
    }
}
