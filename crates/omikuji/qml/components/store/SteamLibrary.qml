import QtQuick
import QtQuick.Controls
import "../cards"


Item {
    id: root

    property var gameModel: null
    property real cardZoom: 1.0
    property string cardStyle: "normal"
    property int cardSpacing: 16
    property bool cardElevation: false
    property string searchText: ""
    property string cardFlow: "center"

    signal backClicked()
    signal gameImported()

    onVisibleChanged: {
        if (visible) {
            loadSteamGames()
            if (gameModel) {
                gameModel.steam_sync_playtime()
            }
        }
    }

    // z:10 so it sits above the CardGrid's empty Flow during load
    Text {
        anchors.centerIn: parent
        text: loading ? qsTr("Loading Steam games...") : qsTr("No Steam games found")
        color: theme.textFaint
        font.pixelSize: theme.type.label.size
        visible: loading || steamGames.length === 0
        z: 10
    }

    CardGrid {
        id: cardGrid
        anchors.fill: parent
        visible: !loading && steamGames.length > 0
        model: steamGames
        cardZoom: root.cardZoom
        cardSpacing: root.cardSpacing
        cardFlow: root.cardFlow
        onBackgroundClicked: root.selectedGameIndex = -1

        delegate: BaseCard {
            id: steamCard
            required property var modelData
            required property int index

            width: 180 * root.cardZoom
            height: styledHeight
            cardStyle: root.cardStyle
            elevation: root.cardElevation

            title: modelData.name
            imageSource: "https://cdn.akamai.steamstatic.com/steam/apps/" + modelData.appid + "/library_600x900.jpg"
            imageFallback: {
                if (!gameModel) return ""
                let local = gameModel.steam_local_library_image(String(modelData.appid))
                return local ? "file://" + local : ""
            }
            leftIconName: "steam"
            leftIconSize: 20
            selected: modelData.imported
            selectedBgTint: theme.alpha(theme.accent, 0.05)
            clickable: false
            cardVisible: root.searchText === ""
                || (modelData.name || "").toLowerCase().includes(root.searchText.toLowerCase())

            actionComponent: Component {
                StoreCardAction {
                    icon: steamCard.modelData.imported ? "bookmark_check" : "add"
                    primary: !steamCard.modelData.imported
                    onClicked: {
                        if (!gameModel) return
                        let success = gameModel.steam_import_game(
                            steamCard.modelData.appid, steamCard.modelData.name)
                        if (success) {
                            steamCard.modelData.imported = true
                            root.gameImported()
                            gameModel.steam_sync_playtime()
                            loadSteamGames()
                        }
                    }
                }
            }
        }
    }

    property bool loading: false
    property var steamGames: []

    Component.onCompleted: loadSteamGames()

    function loadSteamGames() {
        if (!gameModel) return

        loading = true
        let gamesJson = gameModel.steam_get_installed_games()

        try {
            let games = JSON.parse(gamesJson)

            let importedIds = []
            for (let i = 0; i < gameModel.count; i++) {
                let game = gameModel.get_game(i)
                if (game && game.gameId) {
                    importedIds.push(game.gameId)
                }
            }

            steamGames = games
                .map(function(g) {
                    g.imported = importedIds.includes(g.appid)
                    return g
                })
                // sort for stable order, readdir is hash-ordered on ext4 and shuffles between launches
                .sort(function(a, b) {
                    return (a.name || "").toLowerCase().localeCompare((b.name || "").toLowerCase())
                })

        } catch (e) {
            steamGames = []
        }
        loading = false
    }
}
