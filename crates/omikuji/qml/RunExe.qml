import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import omikuji 1.0
import "components/lib/RunnerGrouping.js" as RG

ApplicationWindow {
    id: root

    width: 571
    height: 589
    minimumWidth: 420
    minimumHeight: 480
    visible: true
    title: qsTr("Run with Omikuji")
    color: Theme.surface

    flags: Qt.Window

    property string exePath: ""
    property string runnerValue: ""
    property string prefixValue: ""
    property var games: []
    property string selectedId: ""
    property string searchText: ""

    readonly property string exeName: exePath ? exePath.substring(exePath.lastIndexOf('/') + 1) : ""
    readonly property var selectedGame: games.find(g => g.id === selectedId) || null
    readonly property var shownGames: {
        const q = searchText.trim().toLowerCase()
        return q ? games.filter(g => g.title.toLowerCase().includes(q)) : games
    }

    Component.onCompleted: {
        exePath = gameModel.run_exe_path()
        prefixValue = defaults.getConfig()["wine.prefix"] || ""
        try { games = JSON.parse(gameModel.prefix_games_json()) }
        catch (e) { games = [] }
    }

    function run() {
        const ok = root.selectedGame
            ? gameModel.launch_exe_in_game(root.exePath, root.selectedId)
            : gameModel.launch_exe(root.exePath, root.runnerValue, root.prefixValue)
        if (ok)
            gameModel.quit_now()
    }

    AppSettingsBridge {
        id: appSettings
        Component.onCompleted: Theme.bindSettings(appSettings)
    }

    GameModel { id: gameModel }

    DefaultsBridge { id: defaults }

    Shortcut {
        sequence: "Escape"
        onActivated: gameModel.quit_now()
    }

    MouseArea {
        anchors.fill: parent
        onClicked: forceActiveFocus()
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.space.xl
        spacing: Theme.space.lg

        Text {
            Layout.fillWidth: true
            text: root.exeName || qsTr("Run with Omikuji")
            color: Theme.text
            font.pixelSize: Theme.type.title.size
            font.weight: Theme.type.title.weight
            elide: Text.ElideRight
        }

        ArtCheckList {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.games.length > 0
            title: qsTr("Use a game's settings")
            single: true
            fillHeight: true
            items: root.shownGames
            headerTrailing: M3TextField {
                width: 200
                fieldHeight: 32
                placeholder: qsTr("Search games")
                text: root.searchText
                onTextEdited: (t) => root.searchText = t
            }
            checkedIds: root.selectedId ? [root.selectedId] : []
            onSelectionRequested: (ids) => root.selectedId = ids.length > 0 ? ids[0] : ""
        }

        M3Dropdown {
            Layout.fillWidth: true
            label: qsTr("Runner")
            enabled: !root.selectedGame
            options: RG.groupRunners(JSON.parse(gameModel.list_runners()))
            currentIndex: {
                const chosen = RG.indexOfValue(options, root.selectedGame ? root.selectedGame.runner : root.runnerValue)
                if (chosen >= 0) return chosen
                return RG.selectedIndex(options, defaults.getConfig()["wine.version"])
            }
            onSelected: (val) => root.runnerValue = val
            Component.onCompleted: root.runnerValue = currentValue
        }

        M3FileField {
            Layout.fillWidth: true
            label: qsTr("Prefix")
            placeholder: qsTr("new prefix (auto)")
            selectFolder: true
            enabled: !root.selectedGame
            gameModel: gameModel
            text: root.selectedGame ? root.selectedGame.prefix : root.prefixValue
            onTextEdited: (t) => root.prefixValue = t
            onAccepted: (p) => root.prefixValue = p
        }

        Item {
            Layout.fillHeight: root.games.length === 0
        }

        Row {
            Layout.alignment: Qt.AlignRight
            spacing: Theme.space.md

            M3Button {
                text: qsTr("Close")
                variant: "text"
                onClicked: gameModel.quit_now()
            }

            M3Button {
                text: qsTr("Run")
                variant: "filled"
                enabled: root.exePath !== "" && (root.selectedGame !== null || root.runnerValue !== "")
                onClicked: root.run()
            }
        }
    }
}
