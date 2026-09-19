pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../lib/PlayState.js" as PlayState
import "../lib/RunnerGrouping.js" as RG

// dont re-declare required props here, QML rejects the redeclaration and model roles never reach the card
BaseCard {
    id: card

    property var actions: null
    property bool showPlayButton: false

    property int cardPlayState: PlayState.Play

    function refreshPlayState() {
        card.cardPlayState = card.actions ? card.actions.playStateAt(card.index) : PlayState.Play
    }

    title: name
    imageSource: coverart || banner
    leftIconName: RG.runnerIcon(runnerType)
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
