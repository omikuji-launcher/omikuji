import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window
import "../controls"
import "../primitives"
import "../popups"


// this is such a mess sned help

Window {
    id: logWindow

    property string gameId: ""
    property string gameName: ""
    property var gameModel: null
    property var theme: null
    property var uiSettings: null
    property bool autoScroll: true
    property bool justSaved: false
    property bool searchExpanded: false
    property string rawLog: ""

    signal windowClosed()

    width: 860
    height: 520
    minimumWidth: 420
    minimumHeight: 280
    title: qsTr("omikuji · %1 logs").arg(gameName || gameId)
    color: theme ? theme.bg : "#0a0a0a"

    function refresh() {
        if (!gameModel) return
        logWindow.rawLog = gameModel.game_log(gameId)
        textArea.text = rawLog
        if (autoScroll && (!searchExpanded || searchInput.text.length === 0)) {
            textArea.cursorPosition = textArea.length
        }
    }

    Connections {
        target: gameModel
        function onGameLogAppended(id) {
            if (id !== logWindow.gameId) return
            let wasAtBottom = scroll.contentItem ? scroll.contentItem.atYEnd : true
            let fresh = gameModel.game_log(gameId)
            if (fresh.startsWith(logWindow.rawLog)) {
                if (fresh.length > logWindow.rawLog.length)
                    textArea.insert(textArea.length, fresh.substring(logWindow.rawLog.length))
            } else {
                textArea.text = fresh
            }
            logWindow.rawLog = fresh
            if (wasAtBottom || logWindow.autoScroll) {
                timerScroll.start()
            }
        }
    }

    Timer {
        id: timerScroll
        interval: 50
        onTriggered: textArea.cursorPosition = textArea.length
    }

    Component.onCompleted: {
        highlighter.attach(textArea.textDocument)
        visible = true
        refresh()
        raise()
        requestActivate()
    }

    ThemedLogHighlighter {
        id: highlighter
        settings: logWindow.uiSettings
    }

    onClosing: windowClosed()

    function openSearch() {
        searchExpanded = true
        Qt.callLater(() => {
            searchInput.forceActiveFocus()
            floatingBar.updateMatches()
        })
    }

    function closeSearch() {
        searchExpanded = false
        textArea.select(0, 0)
        textArea.forceActiveFocus()
    }

    Shortcut {
        sequences: [StandardKey.Cancel]
        context: Qt.WindowShortcut
        onActivated: logWindow.searchExpanded ? logWindow.closeSearch() : logWindow.close()
    }

    Shortcut {
        sequence: "Ctrl+F"
        context: Qt.WindowShortcut
        onActivated: logWindow.searchExpanded ? logWindow.closeSearch() : logWindow.openSearch()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 44
            color: logWindow.theme.bgAlt

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 10
                spacing: 10

                Text {
                    Layout.fillWidth: true
                    text: logWindow.gameName
                    color: logWindow.theme.text
                    font.pixelSize: 13
                    font.weight: Font.DemiBold
                    elide: Text.ElideRight
                }

                Item {
                    Layout.preferredWidth: followRow.implicitWidth + 12
                    Layout.preferredHeight: 28

                    Rectangle {
                        anchors.fill: parent
                        radius: theme.radius.xs
                        color: followArea.containsMouse
                            ? Qt.rgba(logWindow.theme.text.r, logWindow.theme.text.g, logWindow.theme.text.b, 0.08)
                            : "transparent"
                        Behavior on color { ColorAnimation { duration: 100 } }
                    }

                    Row {
                        id: followRow
                        anchors.centerIn: parent
                        spacing: 8

                        SvgIcon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: logWindow.autoScroll ? "check_box" : "check_box_outline_blank"
                            size: 18
                            color: logWindow.autoScroll ? logWindow.theme.accent : logWindow.theme.textMuted
                        }

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: qsTr("Follow")
                            color: logWindow.theme.text
                            font.pixelSize: 12
                        }
                    }

                    MouseArea {
                        id: followArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: logWindow.autoScroll = !logWindow.autoScroll
                    }
                }

                M3Button {
                    Layout.preferredWidth: implicitWidth
                    Layout.preferredHeight: implicitHeight
                    small: true
                    variant: "text"
                    text: qsTr("Clear")
                    onClicked: {
                        if (gameModel) {
                            gameModel.clear_game_log(logWindow.gameId)
                            logWindow.refresh()
                        }
                    }
                }

                M3Button {
                    Layout.preferredWidth: implicitWidth
                    Layout.preferredHeight: implicitHeight
                    small: true
                    variant: "text"
                    text: qsTr("Copy all")
                    onClicked: {
                        textArea.selectAll()
                        textArea.copy()
                        textArea.deselect()
                    }
                }

                M3Button {
                    Layout.preferredWidth: implicitWidth
                    Layout.preferredHeight: implicitHeight
                    small: true
                    variant: "tonal"
                    success: logWindow.justSaved
                    text: logWindow.justSaved ? qsTr("Saved ✓") : qsTr("Save")
                    onClicked: {
                        if (!gameModel) return
                        let path = gameModel.save_game_log(logWindow.gameId)
                        if (path && path.length > 0) {
                            logWindow.justSaved = true
                            savedRevertTimer.restart()
                        }
                    }
                }

                Timer {
                    id: savedRevertTimer
                    interval: 2000
                    repeat: false
                    onTriggered: logWindow.justSaved = false
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: logWindow.theme.surfaceBorder
        }

        ScrollView {
            id: scroll
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            TextArea {
                id: textArea
                readOnly: true
                wrapMode: TextArea.Wrap
                selectByMouse: true
                color: logWindow.theme.text
                font.family: "monospace"
                font.pixelSize: 14
                leftPadding: 14
                rightPadding: 14
                topPadding: 10
                bottomPadding: 10
                background: Rectangle { color: logWindow.theme.bg }
                text: ""
            }
        }
    }

    PopupSurface {
        id: searchFab
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 20
        width: 40
        height: 40
        radius: 20
        visible: opacity > 0
        opacity: logWindow.searchExpanded ? 0 : 1
        scale: logWindow.searchExpanded ? 0.85 : 1

        Behavior on opacity { NumberAnimation { duration: logWindow.theme.dur.xfast } }
        Behavior on scale { NumberAnimation { duration: logWindow.theme.dur.fast; easing.type: logWindow.theme.ease.standard } }

        SvgIcon {
            anchors.centerIn: parent
            name: "search"
            size: 18
            color: logWindow.theme.text
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: logWindow.openSearch()
        }
    }

    PopupSurface {
        id: floatingBar
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 20
        width: searchRow.implicitWidth + 32
        height: 40
        radius: logWindow.theme.radius.lg
        transformOrigin: Item.Right
        visible: opacity > 0
        opacity: logWindow.searchExpanded ? 1 : 0
        scale: logWindow.searchExpanded ? 1 : 0.9

        Behavior on opacity { NumberAnimation { duration: logWindow.theme.dur.fast } }
        Behavior on scale { NumberAnimation { duration: logWindow.theme.dur.fast; easing.type: logWindow.theme.ease.standard } }

        property var matchPositions: []
        property int matchCount: 0
        property int currentMatchIndex: -1

        function updateMatches() {
            matchPositions = []
            matchCount = 0
            currentMatchIndex = -1
            if (searchInput.text.length === 0) return

            let content = logWindow.rawLog.toLowerCase()
            let query = searchInput.text.toLowerCase()
            let pos = content.indexOf(query)
            while (pos !== -1) {
                matchPositions.push(pos)
                pos = content.indexOf(query, pos + 1)
            }
            matchCount = matchPositions.length
            if (matchCount > 0) {
                jumpToMatch(0)
            } else {
                textArea.select(0, 0)
            }
        }

        function jumpToMatch(index) {
            if (matchCount === 0) return
            if (index < 0) index = matchCount - 1
            if (index >= matchCount) index = 0
            currentMatchIndex = index
            let pos = matchPositions[index]
            textArea.cursorPosition = pos
            textArea.select(pos, pos + searchInput.text.length)
        }

        Row {
            id: searchRow
            anchors.centerIn: parent
            spacing: 12

            SvgIcon {
                anchors.verticalCenter: parent.verticalCenter
                name: "search"
                size: 16
                color: logWindow.theme.textMuted
            }

            TextInput {
                id: searchInput
                anchors.verticalCenter: parent.verticalCenter
                width: 160
                color: logWindow.theme.text
                font.pixelSize: 14
                clip: true
                selectionColor: logWindow.theme.accent
                selectedTextColor: logWindow.theme.accentText
                selectByMouse: true

                Text {
                    anchors.fill: parent
                    verticalAlignment: Text.AlignVCenter
                    text: qsTr("Search...")
                    color: logWindow.theme.textSubtle
                    font.pixelSize: 14
                    visible: !searchInput.text && !searchInput.activeFocus
                }

                onTextChanged: floatingBar.updateMatches()
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: 50
                horizontalAlignment: Text.AlignHCenter
                text: floatingBar.matchCount > 0 ? (floatingBar.currentMatchIndex + 1) + "/" + floatingBar.matchCount : "0/0"
                color: logWindow.theme.textSubtle
                font.pixelSize: 13
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 4
                IconButton {
                    icon: "chevron_up"
                    size: 24
                    blocked: floatingBar.matchCount === 0
                    onClicked: floatingBar.jumpToMatch(floatingBar.currentMatchIndex - 1)
                }
                IconButton {
                    icon: "chevron_up"
                    size: 24
                    rotation: 180
                    blocked: floatingBar.matchCount === 0
                    onClicked: floatingBar.jumpToMatch(floatingBar.currentMatchIndex + 1)
                }
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 1
                height: 20
                color: logWindow.theme.surfaceBorder
            }

            IconButton {
                anchors.verticalCenter: parent.verticalCenter
                icon: "close"
                size: 24
                onClicked: logWindow.closeSearch()
            }
        }
    }
}
