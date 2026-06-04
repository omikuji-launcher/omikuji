import QtQuick
import "../widgets"

// dont re-declare required props here, QML rejects the redeclaration and model roles never reach the card
BaseCard {
    id: card

    title: name
    imageSource: coverart || banner
    leftIconName: runnerType === "steam" ? "steam"
                : runnerType === "flatpak" ? ""
                : runnerType === "native" ? "terminal"
                : "wine_bar"
    leftIconSize: 20
    clickable: true
    contextEnabled: true
}
