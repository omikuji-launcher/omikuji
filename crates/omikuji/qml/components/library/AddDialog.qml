pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

DialogCard {
    id: root

    signal addGameChosen()
    signal installScriptChosen()

    readonly property var entries: [
        {
            icon: "sports_esports",
            title: qsTr("Add game"),
            subtitle: qsTr("Add a library entry for a game you have on disk"),
            spotlightKey: "add.game",
            chosen: () => root.addGameChosen()
        },
        {
            icon: "code",
            title: qsTr("Install script"),
            subtitle: qsTr("Use scripts to download and set up entries for you"),
            spotlightKey: "",
            chosen: () => root.installScriptChosen()
        }
    ]

    title: qsTr("Add to library")
    maxWidth: 460

    onCloseRequested: close()

    body: Column {
        id: entryList
        width: parent.width

        Repeater {
            model: root.entries

            TileRow {
                id: entryRow
                required property var modelData

                width: entryList.width
                height: 58
                tileSize: 40
                iconSize: 20
                icon: modelData.icon
                title: modelData.title
                subtitle: modelData.subtitle

                onActivated: {
                    root.close()
                    modelData.chosen()
                }

                SvgIcon {
                    name: "chevron_left"
                    rotation: 180
                    size: 16
                    color: Theme.textMuted
                }

                SpotlightTarget {
                    key: entryRow.modelData.spotlightKey
                    target: entryRow
                    padding: 0
                    activate: () => entryRow.activated()
                }
            }
        }
    }
}
