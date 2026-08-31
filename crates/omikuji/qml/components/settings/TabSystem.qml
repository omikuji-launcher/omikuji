pragma ComponentBehavior: Bound

import QtQuick

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
                    checked: root.config["system.gamemode"] === true
                    onToggled: (val) => root.updateField("system.gamemode", val)
                }
            }

            SettingsRow {
                label: qsTr("CPU Cores")
                description: qsTr("0 = no limit")
                width: parent.width
                contentRightMargin: 74
                M3SpinBox {
                    from: 0
                    to: root.gameModel ? Math.max(1, root.gameModel.cpuCoreCount()) : 1
                    stepSize: 1
                    value: root.config["system.cpu_limit"] || 0
                    zeroPlaceholder: "—"
                    onMoved: (val) => root.updateField("system.cpu_limit", val)
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
                    checked: root.config["graphics.mangohud"] === true
                    onToggled: (val) => root.updateField("graphics.mangohud", val)
                }
            }

            M3Dropdown {
                label: qsTr("GPU")
                width: parent.width
                options: root.gameModel ? JSON.parse(root.gameModel.list_gpus()).map(g => ({ label: g[0], value: g[1] })) : [{ label: qsTr("Default"), value: "" }]
                currentIndex: {
                    let v = root.config["graphics.gpu"] || ""
                    let gpus = root.gameModel ? JSON.parse(root.gameModel.list_gpus()) : []
                    let idx = gpus.findIndex(g => g[1] === v)
                    return idx >= 0 ? idx : 0
                }
                onSelected: (val) => root.updateField("graphics.gpu", val)
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
                    checked: root.config["system.pulse_latency"] === true
                    onToggled: (val) => root.updateField("system.pulse_latency", val)
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
                    checked: root.config["system.prevent_sleep"] === true
                    onToggled: (val) => root.updateField("system.prevent_sleep", val)
                }
            }

        }

        SettingsSection {
            label: qsTr("Discord")
            icon: "local_activity"
            width: parent.width

            SettingsRow {
                label: qsTr("Discord Rich Presence")
                description: qsTr("show this game on your Discord profile while it runs")
                width: parent.width
                M3Switch {
                    checked: root.config["system.discord_rpc"] === true
                    onToggled: (val) => root.updateField("system.discord_rpc", val)
                }
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
                    checked: root.config["graphics.gamescope.enabled"] === true
                    onToggled: (val) => root.updateField("graphics.gamescope.enabled", val)
                }
            }

            Column {
                width: parent.width
                spacing: 12
                visible: root.config["graphics.gamescope.enabled"] === true

                Row {
                    spacing: 12
                    width: parent.width

                    M3TextField {
                        label: qsTr("Output Width")
                        placeholder: qsTr("0 = native")
                        text: (root.config["graphics.gamescope.width"] || 0) > 0
                            ? String(root.config["graphics.gamescope.width"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: (t) => root.updateField("graphics.gamescope.width", t || "0")
                    }

                    M3TextField {
                        label: qsTr("Output Height")
                        placeholder: qsTr("0 = native")
                        text: (root.config["graphics.gamescope.height"] || 0) > 0
                            ? String(root.config["graphics.gamescope.height"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: (t) => root.updateField("graphics.gamescope.height", t || "0")
                    }
                }

                Row {
                    spacing: 12
                    width: parent.width

                    M3TextField {
                        label: qsTr("Game Width")
                        placeholder: qsTr("0 = output")
                        text: (root.config["graphics.gamescope.game_width"] || 0) > 0
                            ? String(root.config["graphics.gamescope.game_width"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: (t) => root.updateField("graphics.gamescope.game_width", t || "0")
                    }

                    M3TextField {
                        label: qsTr("Game Height")
                        placeholder: qsTr("0 = output")
                        text: (root.config["graphics.gamescope.game_height"] || 0) > 0
                            ? String(root.config["graphics.gamescope.game_height"]) : ""
                        width: (parent.width - 12) / 2
                        onTextEdited: (t) => root.updateField("graphics.gamescope.game_height", t || "0")
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
                        value: root.config["graphics.gamescope.fps"] || 0
                        zeroPlaceholder: "—"
                        onValueChanged: root.updateField("graphics.gamescope.fps", value)
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
                        value: root.config["graphics.gamescope.refresh_rate"] || 0
                        zeroPlaceholder: "—"
                        onValueChanged: root.updateField("graphics.gamescope.refresh_rate", value)
                    }
                }

                SettingsRow {
                    label: qsTr("Fullscreen")
                    width: parent.width
                    M3Switch {
                        checked: root.config["graphics.gamescope.fullscreen"] === true
                        onToggled: (val) => root.updateField("graphics.gamescope.fullscreen", val)
                    }
                }

                SettingsRow {
                    label: qsTr("Borderless")
                    width: parent.width
                    M3Switch {
                        checked: root.config["graphics.gamescope.borderless"] === true
                        onToggled: (val) => root.updateField("graphics.gamescope.borderless", val)
                    }
                }

                SettingsRow {
                    label: qsTr("Integer Scaling")
                    width: parent.width
                    M3Switch {
                        checked: root.config["graphics.gamescope.integer_scaling"] === true
                        onToggled: (val) => root.updateField("graphics.gamescope.integer_scaling", val)
                    }
                }

                SettingsRow {
                    label: qsTr("HDR")
                    width: parent.width
                    M3Switch {
                        checked: root.config["graphics.gamescope.hdr"] === true
                        onToggled: (val) => root.updateField("graphics.gamescope.hdr", val)
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
                        let f = root.config["graphics.gamescope.filter"] || ""
                        let idx = ["", "nearest", "linear", "fsr", "nis", "pixel"].indexOf(f)
                        return idx >= 0 ? idx : 0
                    }
                    onSelected: (val) => root.updateField("graphics.gamescope.filter", val)
                }

                M3Slider {
                    label: qsTr("FSR Sharpness")
                    from: 0
                    to: 20
                    stepSize: 1
                    value: root.config["graphics.gamescope.fsr_sharpness"] || 0
                    width: parent.width
                    visible: (root.config["graphics.gamescope.filter"] || "") === "fsr"
                    onMoved: (val) => root.updateField("graphics.gamescope.fsr_sharpness", Math.round(val))
                }
            }
        }

        SettingsSection {
            label: qsTr("Environment")
            icon: "tune"
            width: parent.width

            KeyValueTable {
                width: parent.width
                json: root.config["launch.env"] || "{}"
                keyPlaceholder: "VAR_NAME"
                valuePlaceholder: "value"
                addLabel: qsTr("Add variable")
                gameModel: root.gameModel
                onChanged: (j) => root.updateField("launch.env", j)
            }

            M3Button {
                text: {
                    let n = 0
                    try { n = JSON.parse(root.config["launch.env_sets"] || "[]").length } catch (e) {}
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
                text: root.config["launch.pre_launch_script"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => root.updateField("launch.pre_launch_script", t)
            }

            M3FileField {
                label: qsTr("Post-Exit Script")
                placeholder: qsTr("runs after game exits")
                text: root.config["launch.post_exit_script"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => root.updateField("launch.post_exit_script", t)
            }

            M3FileField {
                id: alongsideField

                readonly property bool inPrefix: !["flatpak", "native"].includes(root.config["runner.type"] || "")

                label: qsTr("Run Alongside")
                placeholder: inPrefix ? qsTr("an .exe to run in the prefix while the game runs") : qsTr("a command to run while the game runs")
                text: root.config["launch.alongside"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => root.updateField("launch.alongside", t)
            }

            M3TextField {
                label: qsTr("Arguments")
                placeholder: "-b -a -l -L -s"
                text: root.config["launch.alongside_args"] || ""
                width: parent.width
                visible: alongsideField.text !== ""
                gameModel: root.gameModel
                onTextEdited: (t) => root.updateField("launch.alongside_args", t)
            }

            SettingsRow {
                label: qsTr("Start It First")
                description: qsTr("run it before the game instead of after")
                width: parent.width
                visible: alongsideField.text !== ""
                M3Switch {
                    checked: (root.config["launch.alongside_when"] || "after") === "before"
                    onToggled: (val) => root.updateField("launch.alongside_when", val ? "before" : "after")
                }
            }

            SettingsRow {
                label: qsTr("Delay")
                description: qsTr("seconds to wait between the two, for whichever starts first")
                width: parent.width
                visible: alongsideField.text !== ""
                contentRightMargin: 74
                M3SpinBox {
                    from: 0
                    to: 600
                    stepSize: 5
                    value: root.config["launch.alongside_delay"] || 0
                    zeroPlaceholder: "—"
                    onMoved: (val) => root.updateField("launch.alongside_delay", val)
                }
            }
        }
    }
}
