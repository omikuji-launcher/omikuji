pragma ComponentBehavior: Bound

import QtQuick
import "../cards"
import "../lib/PlayState.js" as PlayState

// dont re-declare required props here, QML rejects the redeclaration and model roles never reach the card
BaseCard {
    id: card

    property var actions: null
    property var gameModel: null
    property bool showPlayButton: false

    property int cardPlayState: PlayState.Play

    function refreshPlayState() {
        if (!card.gameModel || card.index < 0) {
            card.cardPlayState = PlayState.Play
            return
        }
        card.cardPlayState = PlayState.stateFor(
            card.gameModel.is_launching(card.index),
            card.gameModel.is_running(card.index),
            false)
    }

    title: name
    imageSource: coverart || banner
    leftIconName: runnerType === "steam" ? "steam"
                : runnerType === "flatpak" ? ""
                : runnerType === "native" ? "terminal"
                : "wine_bar"
    leftIconSize: 20
    clickable: true
    contextEnabled: true

    overlayComponent: card.showPlayButton && card.actions ? playOverlay : null

    Timer {
        interval: 250
        repeat: true
        triggeredOnStart: true
        running: card.showPlayButton && card.hovered
        onTriggered: card.refreshPlayState()
    }

    Component {
        id: playOverlay

        Item {
            GameActionButton {
                x: (parent.width - width) / 2
                y: card.bannerArea.y + (card.bannerArea.height - height) / 2
                opacity: card.hovered ? 1 : 0
                visible: opacity > 0.01
                iconOnly: true
                actions: card.actions
                index: card.index
                gameId: card.gameId
                playState: card.cardPlayState
                activity: null
                runnerUpdating: false

                Behavior on opacity {
                    NumberAnimation { duration: 120; easing.type: Easing.OutCubic }
                }
            }
        }
    }
}
