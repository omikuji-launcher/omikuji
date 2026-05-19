import QtQuick
import QtQuick.Layouts

import "."
import "../widgets"

Item {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var gameModel: null

    implicitHeight: content.height

    Column {
        id: content
        width: parent.width
        spacing: 20

        SettingsSection {
            label: "Performance"
            icon: "speed"
            width: parent.width

            SettingsRow {
                label: "GameMode"
                description: "Feral GameMode (gamemoderun)"
                width: parent.width
                M3Switch {
                    checked: config["system.gamemode"] === true
                    onToggled: (val) => updateField("system.gamemode", val)
                }
            }

            SettingsRow {
                label: "CPU Cores"
                description: "0 = no limit"
                width: parent.width
                contentRightMargin: 52
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
            label: "Display"
            icon: "fullscreen"
            width: parent.width

            SettingsRow {
                label: "MangoHUD"
                description: "FPS overlay"
                width: parent.width
                M3Switch {
                    checked: config["graphics.mangohud"] === true
                    onToggled: (val) => updateField("graphics.mangohud", val)
                }
            }

            M3Dropdown {
                label: "GPU"
                width: parent.width
                options: gameModel ? JSON.parse(gameModel.list_gpus()).map(g => ({ label: g[0], value: g[1] })) : [{ label: "Default", value: "" }]
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
                label: "Enable Gamescope"
                description: "run game inside gamescope compositor"
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
                        label: "Output Width"
                        placeholder: "0 = native"
                        text: (config["graphics.gamescope.width"] || 0) > 0
                            ? String(config["graphics.gamescope.width"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: updateField("graphics.gamescope.width", text || "0")
                    }

                    M3TextField {
                        label: "Output Height"
                        placeholder: "0 = native"
                        text: (config["graphics.gamescope.height"] || 0) > 0
                            ? String(config["graphics.gamescope.height"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: updateField("graphics.gamescope.height", text || "0")
                    }
                }

                Row {
                    spacing: 12
                    width: parent.width

                    M3TextField {
                        label: "Game Width"
                        placeholder: "0 = output"
                        text: (config["graphics.gamescope.game_width"] || 0) > 0
                            ? String(config["graphics.gamescope.game_width"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: updateField("graphics.gamescope.game_width", text || "0")
                    }

                    M3TextField {
                        label: "Game Height"
                        placeholder: "0 = output"
                        text: (config["graphics.gamescope.game_height"] || 0) > 0
                            ? String(config["graphics.gamescope.game_height"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: updateField("graphics.gamescope.game_height", text || "0")
                    }
                }

                SettingsRow {
                    label: "FPS Limit"
                    width: parent.width
                    M3SpinBox {
                        from: 0
                        to: 999
                        stepSize: 1
                        value: config["graphics.gamescope.fps"] || 0
                        onValueChanged: updateField("graphics.gamescope.fps", value)
                    }
                }

                SettingsRow {
                    label: "Fullscreen"
                    width: parent.width
                    M3Switch {
                        checked: config["graphics.gamescope.fullscreen"] === true
                        onToggled: (val) => updateField("graphics.gamescope.fullscreen", val)
                    }
                }

                SettingsRow {
                    label: "Borderless"
                    width: parent.width
                    M3Switch {
                        checked: config["graphics.gamescope.borderless"] === true
                        onToggled: (val) => updateField("graphics.gamescope.borderless", val)
                    }
                }

                SettingsRow {
                    label: "Integer Scaling"
                    width: parent.width
                    M3Switch {
                        checked: config["graphics.gamescope.integer_scaling"] === true
                        onToggled: (val) => updateField("graphics.gamescope.integer_scaling", val)
                    }
                }

                SettingsRow {
                    label: "HDR"
                    width: parent.width
                    M3Switch {
                        checked: config["graphics.gamescope.hdr"] === true
                        onToggled: (val) => updateField("graphics.gamescope.hdr", val)
                    }
                }

                M3Dropdown {
                    label: "Filter"
                    width: parent.width
                    options: [
                        { label: "None", value: "" },
                        { label: "Nearest", value: "nearest" },
                        { label: "Linear", value: "linear" },
                        { label: "FSR", value: "fsr" },
                        { label: "NIS", value: "nis" },
                        { label: "Pixel", value: "pixel" }
                    ]
                    currentIndex: {
                        let f = config["graphics.gamescope.filter"] || ""
                        let idx = ["", "nearest", "linear", "fsr", "nis", "pixel"].indexOf(f)
                        return idx >= 0 ? idx : 0
                    }
                    onSelected: (val) => updateField("graphics.gamescope.filter", val)
                }

                M3Slider {
                    label: "FSR Sharpness"
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
            label: "Audio"
            icon: "volume_up"
            width: parent.width

            SettingsRow {
                label: "Reduce Pulse Latency"
                width: parent.width
                M3Switch {
                    checked: config["system.pulse_latency"] === true
                    onToggled: (val) => updateField("system.pulse_latency", val)
                }
            }
        }

        SettingsSection {
            label: "Power"
            icon: "power_settings_new"
            width: parent.width

            SettingsRow {
                label: "Prevent Sleep"
                description: "inhibit screensaver and sleep"
                width: parent.width
                M3Switch {
                    checked: config["system.prevent_sleep"] === true
                    onToggled: (val) => updateField("system.prevent_sleep", val)
                }
            }
        }

        SettingsSection {
            label: "Environment"
            icon: "tune"
            width: parent.width

            KeyValueTable {
                width: parent.width
                json: config["launch.env"] || "{}"
                keyPlaceholder: "VAR_NAME"
                valuePlaceholder: "value"
                addLabel: "Add variable"
                onChanged: (j) => updateField("launch.env", j)
            }
        }

        SettingsSection {
            label: "Scripts"
            icon: "code"
            width: parent.width

            M3FileField {
                label: "Pre-Launch Script"
                placeholder: "runs before game starts"
                text: config["launch.pre_launch_script"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("launch.pre_launch_script", t)
            }

            M3FileField {
                label: "Post-Exit Script"
                placeholder: "runs after game exits"
                text: config["launch.post_exit_script"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("launch.post_exit_script", t)
            }
        }
    }
}
