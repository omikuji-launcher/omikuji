import QtQuick
import QtQuick.Controls
import "../controls"
import "../popups"
import "../primitives"


Item {
    id: root

    property string currentTabLabel: ""
    property bool showTitle: false
    property real leftInset: 0
    property bool showAddButton: true
    property bool showSearch: true
    property bool showDisplayOptions: false
    property real zoomValue: 1.0
    property int spacingValue: 16
    property string sortValue: "default"
    property bool showSort: false
    property bool showHiddenValue: false
    property bool showHiddenOption: false
    property string cardStyleValue: "normal"
    property alias searchText: searchInput.text

    signal addClicked()
    signal installScriptClicked()
    signal zoomMoved(real value)
    signal spacingMoved(int value)
    signal sortSelected(string value)
    signal showHiddenToggled(bool value)
    signal cardStyleSelected(string value)
    signal consoleModeClicked()

    height: 54

    function defocusSearch() {
        searchInput.focus = false
    }

    // opaque fill becuase witout it lower-z dropdown popups bleed through the empty bar areas
    Rectangle {
        anchors.fill: parent
        color: theme.navBg
    }

    Text {
        id: titleText
        anchors.left: parent.left
        anchors.leftMargin: 24
        anchors.verticalCenter: parent.verticalCenter
        width: Math.min(implicitWidth, root.width * 0.5)
        text: root.currentTabLabel
        color: theme.text
        font.pixelSize: theme.type.display.size
        font.weight: Font.DemiBold
        elide: Text.ElideRight
        visible: root.showTitle
    }

    FieldSurface {
        id: searchBar
        anchors.verticalCenter: parent.verticalCenter
        x: (root.width - root.leftInset) / 2 - width / 2
        width: Math.min(360, parent.width * 0.4)
        height: 34
        radius: 17
        focused: searchInput.activeFocus
        visible: root.showSearch

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            spacing: 8

            SvgIcon {
                name: "search"
                size: 16
                color: theme.textSubtle
                anchors.verticalCenter: parent.verticalCenter
            }

            TextInput {
                id: searchInput
                width: searchBar.width - 44
                color: theme.text
                font.pixelSize: theme.type.body.size
                clip: true
                anchors.verticalCenter: parent.verticalCenter
                selectionColor: theme.accent
                selectedTextColor: theme.accentText

                Text {
                    anchors.fill: parent
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Search games...")
                    color: theme.textSubtle
                    font.pixelSize: theme.type.body.size
                    visible: !searchInput.text && !searchInput.activeFocus
                }
            }
        }
    }

    Row {
        anchors.right: parent.right
        anchors.rightMargin: 16
        anchors.verticalCenter: parent.verticalCenter
        spacing: 6

        IconButton {
            id: consoleBtn
            icon: "sports_esports"
            size: 32
            rounded: true
            anchors.verticalCenter: parent.verticalCenter
            onClicked: root.consoleModeClicked()

            Tooltip {
                text: qsTr("Console Mode")
                tipVisible: consoleBtn.hovered
                y: parent.height + 8
            }
        }

        IconButton {
            id: displayBtn
            icon: "tune"
            size: 32
            rounded: true
            anchors.verticalCenter: parent.verticalCenter
            visible: root.showDisplayOptions
            onClicked: displayPopup.visible ? displayPopup.close() : displayPopup.open()

            Tooltip {
                text: qsTr("Quick Settings")
                tipVisible: displayBtn.hovered && !displayPopup.visible
                y: parent.height + 8
            }
        }

        IconButton {
            id: addBtn
            icon: "add"
            size: 32
            rounded: true
            anchors.verticalCenter: parent.verticalCenter
            visible: root.showAddButton
            onClicked: if (Date.now() - addMenu.lastClosedAt > 150) addMenu.open()

            Tooltip {
                text: qsTr("Add")
                tipVisible: addBtn.hovered && !addMenu.visible
                y: parent.height + 8
            }
        }
    }

    ContextMenu {
        id: addMenu
        parent: addBtn
        x: addBtn.width - width
        y: addBtn.height + 8
        items: [
            { text: qsTr("Add game"), action: "add_game" },
            { text: qsTr("Install script"), action: "install_script" }
        ]
        onItemClicked: (action) => {
            close()
            if (action === "add_game") root.addClicked()
            else if (action === "install_script") root.installScriptClicked()
        }
    }

    DisplayOptionsPopup {
        id: displayPopup
        parent: displayBtn
        x: displayBtn.width - width
        y: displayBtn.height + 8

        zoomValue: root.zoomValue
        spacingValue: root.spacingValue
        sortValue: root.sortValue
        showSort: root.showSort
        showHiddenValue: root.showHiddenValue
        showHiddenOption: root.showHiddenOption
        cardStyleValue: root.cardStyleValue
        onZoomMoved: (v) => root.zoomMoved(v)
        onSpacingMoved: (v) => root.spacingMoved(v)
        onSortSelected: (v) => root.sortSelected(v)
        onShowHiddenToggled: (v) => root.showHiddenToggled(v)
        onCardStyleSelected: (v) => root.cardStyleSelected(v)
    }
}
