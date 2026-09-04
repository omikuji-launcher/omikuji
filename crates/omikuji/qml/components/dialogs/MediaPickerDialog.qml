pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import Qt5Compat.GraphicalEffects
import omikuji 1.0
import "../controls"
import "../primitives"

DialogCard {
    id: root

    property var gameModel: null
    property string gameId: ""
    property string kind: ""
    property var games: []
    property var assets: []
    property bool loading: false
    property bool picking: false
    property int sgdbGameId: 0

    readonly property real aspect: ({ coverart: 2 / 3, banner: 1920 / 620, icon: 1 })[kind] || 1
    readonly property real minCellWidth: ({ coverart: 180, banner: 320, icon: 120 })[kind] || 180

    readonly property var gameOptions: root.games.map(g => ({
        label: g.verified ? qsTr("%1 (verified)").arg(g.name) : g.name,
        value: g.id
    }))

    title: qsTr("Choose artwork")
    sizeKey: "media_picker"
    maxWidth: 880
    preferredHeight: 640
    scrollable: false
    fillHeight: true

    function show(gameId_, kind_) {
        root.gameId = gameId_
        root.kind = kind_
        root.games = []
        root.assets = []
        root.sgdbGameId = 0
        root.errorText = ""
        root.picking = false
        pickTimeout.stop()
        root.loading = true
        root.open()
        root.search(0)
    }

    Timer {
        id: pickTimeout
        interval: 20000
        onTriggered: {
            root.picking = false
            root.errorText = qsTr("Couldn't download that image.")
        }
    }

    function search(sgdbGameId_) {
        if (!root.gameModel) return
        root.gameModel.searchMediaCandidates(root.gameId, root.kind, sgdbGameId_)
    }

    onCloseRequested: close()

    Connections {
        target: root.gameModel
        enabled: root.gameModel !== null && root.shown

        function onMediaChanged(gameId) {
            if (!root.picking || gameId !== root.gameId) return
            pickTimeout.stop()
            root.picking = false
            root.close()
        }

        function onMediaCandidatesReady(gameId, kind, payload) {
            if (gameId !== root.gameId || kind !== root.kind) return
            root.loading = false

            let data = null
            try { data = JSON.parse(payload) } catch (e) { data = null }
            if (!data) {
                root.errorText = qsTr("Couldn't read the SteamGridDB response.")
                return
            }
            if (data.error) {
                root.errorText = data.error
                return
            }

            root.errorText = ""
            if (data.games && data.games.length > 0) root.games = data.games
            root.sgdbGameId = data.sgdbGameId || 0
            root.assets = data.assets || []
        }
    }

    body: Item {
        height: parent.height

        M3Dropdown {
            id: matchPicker
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            visible: root.gameOptions.length > 1
            label: qsTr("SteamGridDB match")
            options: root.gameOptions
            currentIndex: Math.max(0, root.gameOptions.findIndex(o => o.value === root.sgdbGameId))
            onSelected: (v) => {
                if (v === root.sgdbGameId) return
                root.sgdbGameId = v
                root.assets = []
                root.loading = true
                root.search(v)
            }
        }

        Flickable {
            id: artFlick
            anchors.top: matchPicker.visible ? matchPicker.bottom : parent.top
            anchors.topMargin: matchPicker.visible ? Theme.space.xl : 0
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.rightMargin: Theme.space.xs
            anchors.bottom: parent.bottom
            contentWidth: width
            contentHeight: grid.height
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            visible: !root.loading && !root.picking && root.assets.length > 0
            ScrollBar.vertical: ThinScrollBar {}

            Grid {
                id: grid
                columns: Math.max(1, Math.floor((artFlick.width + spacing) / (root.minCellWidth + spacing)))
                spacing: Theme.space.sm

                readonly property real cellWidth: (artFlick.width - spacing * (columns - 1)) / columns

                Repeater {
                    model: root.assets

                    Item {
                        id: cell
                        required property var modelData

                        width: grid.cellWidth
                        height: Math.round(grid.cellWidth / root.aspect)

                        Rectangle {
                            id: cellBg
                            anchors.fill: parent
                            radius: Theme.radius.sm
                            color: Theme.alpha(Theme.text, 0.06)
                            antialiasing: true
                        }

                        Image {
                            id: art
                            anchors.fill: parent
                            source: cell.modelData.thumb || cell.modelData.url
                            fillMode: Image.PreserveAspectCrop
                            asynchronous: true
                            cache: false
                            sourceSize.width: Math.round(grid.cellWidth * 2)
                            layer.enabled: true
                            layer.smooth: true
                            layer.effect: OpacityMask {
                                maskSource: Rectangle {
                                    width: art.width
                                    height: art.height
                                    radius: cellBg.radius
                                    antialiasing: true
                                }
                            }
                        }

                        Rectangle {
                            anchors.fill: parent
                            radius: cellBg.radius
                            color: "transparent"
                            antialiasing: true
                            border.width: tapArea.containsMouse ? 3 : 0
                            border.color: Theme.accent
                        }

                        Rectangle {
                            anchors.bottom: parent.bottom
                            anchors.horizontalCenter: parent.horizontalCenter
                            anchors.bottomMargin: Theme.space.xs
                            width: sizeLabel.implicitWidth + Theme.space.sm
                            height: sizeLabel.implicitHeight + 4
                            radius: Theme.radius.xs
                            color: "#99000000"
                            visible: tapArea.containsMouse && cell.modelData.width > 0

                            Text {
                                id: sizeLabel
                                anchors.centerIn: parent
                                text: cell.modelData.width + " x " + cell.modelData.height
                                color: "#ffffff"
                                font.pixelSize: Theme.type.micro.size
                            }
                        }

                        MouseArea {
                            id: tapArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (!root.gameModel) return
                                root.errorText = ""
                                root.picking = true
                                pickTimeout.restart()
                                root.gameModel.pickMediaCandidate(
                                    root.gameId, root.kind, cell.modelData.url)
                            }
                        }
                    }
                }
            }
        }

        LoadingDots {
            anchors.centerIn: parent
            visible: root.loading || root.picking
            running: visible
            text: root.picking ? qsTr("Downloading") : qsTr("Searching SteamGridDB")
        }

        EmptyState {
            anchors.centerIn: parent
            visible: !root.loading && !root.picking
                && root.assets.length === 0 && root.errorText === ""
            icon: "image"
            text: qsTr("No artwork found")
            hint: root.gameOptions.length > 1
                ? qsTr("Try a different SteamGridDB match.")
                : qsTr("SteamGridDB has nothing for this game.")
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Close")
            variant: "text"
            onClicked: root.close()
        }
    }
}
