import QtQuick
import QtQuick.Layouts
import "../controls"
import "../primitives"

DialogCard {
    sizeKey: "game_categories"
    id: root

    property var gameModel: null
    property var uiSettings: null
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
        if (!uiSettings) { root.tagCategories = []; return }
        let all = []
        try { all = JSON.parse(uiSettings.categoriesJson()) } catch (e) { all = [] }
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
        target: uiSettings
        function onCategoriesChanged() {
            if (root.shown) root._loadCategories()
        }
    }

    body: ColumnLayout {
        width: parent.width
        spacing: theme.space.sm

        Text {
            Layout.fillWidth: true
            text: root.gameName
            color: theme.textMuted
            font.pixelSize: theme.type.caption.size
            elide: Text.ElideRight
            visible: text.length > 0
        }

        Text {
            Layout.fillWidth: true
            text: qsTr("No tag categories yet. Create one to start tagging.")
            color: theme.textSubtle
            font.pixelSize: theme.type.caption.size
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
                        required property var modelData

                        width: parent.width
                        height: 40

                        readonly property bool selected: root.selectedTags.indexOf(modelData.value) !== -1

                        Rectangle {
                            anchors.fill: parent
                            radius: theme.radius.sm
                            color: rowHover.containsMouse
                                ? theme.alpha(theme.text, 0.06)
                                : "transparent"
                            Behavior on color { ColorAnimation { duration: 100 } }
                        }

                        Row {
                            anchors.left: parent.left
                            anchors.leftMargin: 10
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: theme.space.md

                            SvgIcon {
                                anchors.verticalCenter: parent.verticalCenter
                                name: selected ? "check_box" : "check_box_outline_blank"
                                size: 20
                                color: selected ? theme.accent : theme.alpha(theme.text, 0.55)
                            }

                            SvgIcon {
                                name: modelData.icon
                                size: 18
                                color: theme.icon
                                anchors.verticalCenter: parent.verticalCenter
                            }

                            Text {
                                text: modelData.name
                                color: theme.text
                                font.pixelSize: theme.type.body.size
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }

                        MouseArea {
                            id: rowHover
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root._toggleTag(modelData.value)
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
        spacing: theme.space.sm

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
