pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts
import "../controls"
import "../primitives"

DialogCard {
    sizeKey: "game_categories"
    id: root

    property var gameModel: null
    property var appSettings: null
    property int gameIndex: -1
    property string gameName: ""

    property var tagCategories: []
    property var selectedTags: []

    signal requestNewCategory()

    maxWidth: 440
    title: qsTr("Categories")

    function show(index) {
        if (!gameModel || index < 0) return
        let g = gameModel.get_game(index)
        if (!g) return
        root.gameIndex = index
        root.gameName = g.name || ""
        try { root.selectedTags = JSON.parse(g.categories || "[]") }
        catch (e) { root.selectedTags = [] }
        _loadCategories()
        open()
    }

    function hide() { close() }

    function _loadCategories() {
        if (!appSettings) { root.tagCategories = []; return }
        let all = []
        try { all = JSON.parse(appSettings.categoriesJson()) } catch (e) { all = [] }
        let tags = []
        for (let i = 0; i < all.length; i++) {
            if (all[i].kind === "tag") tags.push(all[i])
        }
        root.tagCategories = tags
    }

    function _toggleTag(value) {
        let current = root.selectedTags.slice()
        let idx = current.indexOf(value)
        if (idx === -1) current.push(value)
        else current.splice(idx, 1)
        root.selectedTags = current
    }

    function _save() {
        if (!gameModel || gameIndex < 0) return
        let json = JSON.stringify(root.selectedTags)
        gameModel.update_game_field(gameIndex, "meta.categories", json)
        let g = gameModel.get_game(gameIndex)
        if (g) gameModel.save_game(g.gameId)
        close()
    }

    onCloseRequested: root.close()

    Connections {
        target: root.appSettings
        function onCategoriesChanged() {
            if (root.shown) root._loadCategories()
        }
    }

    body: ColumnLayout {
        width: parent.width
        spacing: Theme.space.sm

        Text {
            Layout.fillWidth: true
            text: root.gameName
            color: Theme.textMuted
            font.pixelSize: Theme.type.caption.size
            elide: Text.ElideRight
            visible: text.length > 0
        }

        Text {
            Layout.fillWidth: true
            text: qsTr("No tag categories yet. Create one to start tagging.")
            color: Theme.textSubtle
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.Wrap
            visible: root.tagCategories.length === 0
        }

        Flickable {
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(tagList.height, 320)
            contentHeight: tagList.height
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            interactive: contentHeight > height
            visible: root.tagCategories.length > 0

            Column {
                id: tagList
                width: parent.width
                spacing: 4

                Repeater {
                    model: root.tagCategories

                    Item {
                        id: tagRow
                        required property var modelData

                        width: parent.width
                        height: 40

                        readonly property bool selected: root.selectedTags.indexOf(modelData.value) !== -1

                        Rectangle {
                            anchors.fill: parent
                            radius: Theme.radius.sm
                            color: rowHover.containsMouse
                                ? Theme.alpha(Theme.text, 0.06)
                                : "transparent"
                            Behavior on color { ColorAnimation { duration: 100 } }
                        }

                        Row {
                            anchors.left: parent.left
                            anchors.leftMargin: 10
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: Theme.space.md

                            M3Checkbox {
                                anchors.verticalCenter: parent.verticalCenter
                                checked: tagRow.selected
                            }

                            SvgIcon {
                                name: tagRow.modelData.icon
                                size: 18
                                color: Theme.icon
                                anchors.verticalCenter: parent.verticalCenter
                            }

                            Text {
                                text: tagRow.modelData.name
                                color: Theme.text
                                font.pixelSize: Theme.type.body.size
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }

                        MouseArea {
                            id: rowHover
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root._toggleTag(tagRow.modelData.value)
                        }
                    }
                }
            }
        }

    }

    footerLeft: M3Button {
        text: qsTr("New category")
        variant: "tonal"
        icon: "add"
        onClicked: root.requestNewCategory()
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "text"
            onClicked: root.close()
        }

        M3Button {
            text: qsTr("Save")
            variant: "filled"
            onClicked: root._save()
        }
    }
}
