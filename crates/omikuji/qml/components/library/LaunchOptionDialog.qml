pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

DialogCard {
    id: root

    property var gameModel: null
    property int gameIndex: -1
    property var onPicked: null
    property var options: []
    property string pickedId: ""
    property bool remember: false

    sizeKey: "launch_options"
    maxWidth: 370
    title: qsTr("Launch options")

    function start(idx, then) {
        gameIndex = idx
        onPicked = then
        options = JSON.parse(gameModel.launch_options(idx))
        pickedId = options.length > 0 ? options[0].id : ""
        remember = false
        open()
    }

    function confirm() {
        if (remember && gameModel.update_game_field(gameIndex, "launch.store_option", pickedId))
            gameModel.save_game(gameModel.get_game(gameIndex)["gameId"])
        close()
        onPicked(pickedId)
    }

    onCloseRequested: close()

    body: Column {
        width: parent.width
        spacing: Theme.space.md

        ArtCheckList {
            width: parent.width
            single: true
            artVisible: false
            visibleRows: 5
            items: root.options
            checkedIds: [root.pickedId]
            onSelectionRequested: (ids) => { if (ids.length > 0) root.pickedId = ids[0] }
        }

        SwitchField {
            width: parent.width
            label: qsTr("Remember for this game")
            checked: root.remember
            onToggled: (val) => root.remember = val
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "text"
            onClicked: root.close()
        }
        M3Button {
            text: qsTr("Play")
            variant: "filled"
            onClicked: root.confirm()
        }
    }
}
