import QtQuick
import QtQuick.Controls
import QtQuick.Window

import omikuji 1.0
import "components/lib/RunnerGrouping.js" as RG
import "components/controls"

ApplicationWindow {
    id: root

    width: 460
    height: 300
    minimumWidth: 380
    minimumHeight: 260
    visible: true
    title: qsTr("Run with Omikuji")
    color: Theme.surface

    flags: Qt.Window

    property string exePath: ""
    property string runnerValue: ""
    property string prefixValue: ""

    readonly property string exeName: exePath ? exePath.substring(exePath.lastIndexOf('/') + 1) : ""

    Component.onCompleted: {
        exePath = gameModel.run_exe_path()
        prefixValue = defaults.getConfig()["wine.prefix"] || ""
    }

    AppSettingsBridge {
        id: appSettings
        Component.onCompleted: {
            Theme.mutedIcons = Qt.binding(() => appSettings.mutedIcons)
            Theme.filledIcons = Qt.binding(() => appSettings.filledIcons)
            Theme.followSystemColors = Qt.binding(() => appSettings.followSystemColors)
            Theme.followSystemFont = Qt.binding(() => appSettings.followSystemFont)
            Theme.fontFamily = Qt.binding(() => appSettings.fontFamily)
            Theme.fillFields = Qt.binding(() => appSettings.fillFields)
            Theme.overrides = JSON.parse(overridesJson())
            Theme.fontSizes = JSON.parse(fontSizesJson())
            Theme.radiusOverrides = JSON.parse(radiusOverridesJson())
        }
    }

    Connections {
        target: appSettings
        function onThemeChanged() {
            Theme.overrides = JSON.parse(appSettings.overridesJson())
        }
        function onFontSizesChanged() {
            Theme.fontSizes = JSON.parse(appSettings.fontSizesJson())
        }
        function onRadiusOverridesChanged() {
            Theme.radiusOverrides = JSON.parse(appSettings.radiusOverridesJson())
        }
    }

    GameModel { id: gameModel }

    DefaultsBridge { id: defaults }

    Shortcut {
        sequence: "Escape"
        onActivated: gameModel.quit_now()
    }

    Item {
        anchors.fill: parent
        anchors.margins: Theme.space.xl

        Text {
            id: titleText
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            text: root.exeName || qsTr("Run with Omikuji")
            color: Theme.text
            font.pixelSize: Theme.type.title.size
            font.weight: Theme.type.title.weight
            elide: Text.ElideRight
        }

        Column {
            anchors.top: titleText.bottom
            anchors.topMargin: Theme.space.lg
            anchors.left: parent.left
            anchors.right: parent.right
            spacing: Theme.space.lg

            M3Dropdown {
                label: qsTr("Runner")
                width: parent.width
                options: RG.groupRunners(JSON.parse(gameModel.list_runners()))
                currentIndex: {
                    let def = defaults.getConfig()["wine.version"] || ""
                    let i = RG.preferredIndex(options, def, ["GE-Proton", "Proton-GE", "wine-ge"])
                    if (i >= 0) return i
                    let f = RG.firstNonHeader(options)
                    return f >= 0 ? f : 0
                }
                onSelected: (val) => root.runnerValue = val
                Component.onCompleted: root.runnerValue = currentValue
            }

            M3FileField {
                label: qsTr("Prefix")
                placeholder: qsTr("empty = auto-create")
                selectFolder: true
                width: parent.width
                gameModel: gameModel
                text: root.prefixValue
                onTextEdited: (t) => root.prefixValue = t
                onAccepted: (p) => root.prefixValue = p
            }
        }

        Row {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            spacing: Theme.space.md

            M3Button {
                text: qsTr("Close")
                variant: "text"
                onClicked: gameModel.quit_now()
            }

            M3Button {
                text: qsTr("Run")
                variant: "filled"
                enabled: root.exePath !== "" && root.runnerValue !== ""
                onClicked: {
                    if (gameModel.launch_exe(root.exePath, root.runnerValue, root.prefixValue))
                        gameModel.quit_now()
                }
            }
        }
    }
}
