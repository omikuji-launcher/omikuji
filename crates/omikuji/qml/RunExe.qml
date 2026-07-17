import QtQuick
import QtQuick.Controls
import QtQuick.Window

import omikuji 1.0
import "components"
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
    color: theme.surface

    flags: Qt.Window

    property string exePath: ""
    property string runnerValue: ""
    property string prefixValue: ""

    readonly property string exeName: exePath ? exePath.substring(exePath.lastIndexOf('/') + 1) : ""

    Component.onCompleted: {
        exePath = gameModel.run_exe_path()
        prefixValue = defaults.getConfig()["wine.prefix"] || ""
    }

    UiSettingsBridge {
        id: uiSettings
        Component.onCompleted: {
            theme.overrides = JSON.parse(overridesJson())
            theme.fontSizes = JSON.parse(fontSizesJson())
        }
    }

    Theme {
        id: theme
        mutedIcons: uiSettings.mutedIcons
        filledIcons: uiSettings.filledIcons
        followSystemColors: uiSettings.followSystemColors
        followSystemFont: uiSettings.followSystemFont
        fontFamily: uiSettings.fontFamily
        fillFields: uiSettings.fillFields
        radiusScale: uiSettings.radiusScale
    }

    Connections {
        target: uiSettings
        function onThemeChanged() {
            theme.overrides = JSON.parse(uiSettings.overridesJson())
        }
        function onFontSizesChanged() {
            theme.fontSizes = JSON.parse(uiSettings.fontSizesJson())
        }
    }

    GameModel { id: gameModel }

    DefaultsBridge { id: defaults }

    Timer {
        interval: 200
        repeat: true
        running: true
        onTriggered: gameModel.drain_file_dialog_results()
    }

    Shortcut {
        sequence: "Escape"
        onActivated: gameModel.quit_now()
    }

    Item {
        anchors.fill: parent
        anchors.margins: theme.space.xl

        Text {
            id: titleText
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            text: root.exeName || qsTr("Run with Omikuji")
            color: theme.text
            font.pixelSize: theme.type.title.size
            font.weight: theme.type.title.weight
            elide: Text.ElideRight
        }

        Column {
            anchors.top: titleText.bottom
            anchors.topMargin: theme.space.lg
            anchors.left: parent.left
            anchors.right: parent.right
            spacing: theme.space.lg

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
            spacing: theme.space.md

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
