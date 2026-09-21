pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import omikuji 1.0
import "../lib/Omikuji.js" as Omikuji


DialogCard {
    id: root

    property var gameModel: null
    property int drawnIndex: -1
    property string phase: "shaking"

    readonly property var game: (root.gameModel && root.drawnIndex >= 0)
        ? root.gameModel.get_game(root.drawnIndex) : null
    readonly property var reading: root.game ? Omikuji.readingFor(root.game) : null
    readonly property bool cursed: root.reading !== null && root.reading.tier === "kyo"
    readonly property color tierColor: root.cursed ? Theme.error : Theme.accent

    signal playRequested(int index)
    signal tieRequested(string gameId)
    signal redrawRequested()
    signal ritualStart()

    function reveal(index) {
        root.drawnIndex = index
        root.phase = index >= 0 ? "shaking" : "revealed"
        if (index >= 0) root.ritualStart()
    }

    function tierLabel(tier) {
        switch (tier) {
            case "daikichi": return qsTr("Great blessing")
            case "kichi": return qsTr("Blessing")
            case "chukichi": return qsTr("Middle blessing")
            case "shokichi": return qsTr("Small blessing")
            case "suekichi": return qsTr("Future blessing")
            case "kyo": return qsTr("Curse")
        }
        return ""
    }

    function readingLine(reason) {
        switch (reason) {
            case "untrodden": return qsTr("Never once started it. What, you forgot this exists? Tsk.")
            case "turnedBack": return qsTr("Bounced off in under half an hour. Do better.")
            case "noRecord": return qsTr("Hours played. No idea when though.")
            case "aYearGone": return qsTr("Untouched for over a year. You know it's like cheating, right...? 💢")
            case "wellWorn": return qsTr("A hundred hours deep. Touch grass, really.")
            case "seasonsPassed": return qsTr("It's been months. You've forgotten the controls, dummy.")
            case "awhileBack": return qsTr("Two weeks off. Don't drop it.")
            case "justHere": return qsTr("You literally just played this. Greedy.")
        }
        return ""
    }

    sizeKey: "omikujiDraw"
    maxWidth: 380
    preferredHeight: 430
    scrollable: false
    resizable: false
    title: qsTr("Draw a fortune")

    onCloseRequested: root.close()

    body: Item {
        width: parent.width
        implicitHeight: 300

        // lives in here because body is a Component: ids inside it are invisible from the outside
        Connections {
            target: root
            function onRitualStart() { scene.start() }
        }

        Component.onCompleted: if (root.phase === "shaking") scene.start()

        Item {
            id: stage

            property real vanish: root.phase === "shaking" ? 1 : 0

            anchors.centerIn: parent
            anchors.verticalCenterOffset: -14
            width: 200
            height: 170
            opacity: stage.vanish

            Behavior on vanish { NumberAnimation { duration: 220 } }

            MikujiScene {
                id: scene
                anchors.fill: parent
                onRevealed: root.phase = "revealed"
            }
        }

        Item {
            id: slip

            property real unroll: root.phase === "revealed" ? 1 : 0
            property real stamp: root.phase === "revealed" ? 1 : 0

            anchors.fill: parent
            visible: root.phase === "revealed"

            Behavior on unroll {
                enabled: root.game !== null
                NumberAnimation { duration: 280; easing.type: Easing.OutCubic }
            }

            Behavior on stamp {
                enabled: root.game !== null
                SequentialAnimation {
                    PauseAnimation { duration: 280 }
                    NumberAnimation { duration: 300; easing.type: Easing.OutBack }
                }
            }

            transform: Scale {
                origin.x: slip.width / 2
                origin.y: 0
                yScale: slip.unroll
            }

            ColumnLayout {
                anchors.centerIn: parent
                width: parent.width - Theme.space.xl * 2
                spacing: Theme.space.sm

                Text {
                    Layout.alignment: Qt.AlignHCenter
                    visible: root.reading !== null
                    text: root.reading ? Omikuji.TIERS[root.reading.tier].kanji : ""
                    color: root.tierColor
                    font.pixelSize: 72
                    font.weight: Font.DemiBold
                    opacity: slip.stamp
                    scale: 1.3 - 0.3 * slip.stamp
                }

                MoodySpirit {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: 150
                    Layout.preferredHeight: 165
                    visible: root.reading === null
                    mark: "anger"
                    dread: 0
                }

                Text {
                    Layout.alignment: Qt.AlignHCenter
                    text: root.reading ? root.tierLabel(root.reading.tier) : qsTr("The box is empty")
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.micro.size
                    font.capitalization: Font.AllUppercase
                    font.letterSpacing: 0.6
                    font.weight: Font.DemiBold
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.topMargin: Theme.space.sm
                    Layout.bottomMargin: Theme.space.sm
                    Layout.preferredHeight: 1
                    color: Theme.separator
                }

                Text {
                    Layout.fillWidth: true
                    text: root.game ? root.game.name
                        : qsTr("Nothing in the library matches the current filter.")
                    color: Theme.text
                    font.pixelSize: Theme.type.headline.size
                    font.weight: Font.DemiBold
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.WordWrap
                }


                Text {
                    Layout.fillWidth: true
                    visible: root.reading !== null
                    text: root.reading ? root.readingLine(root.reading.reason) : ""
                    color: Theme.textSubtle
                    font.pixelSize: Theme.type.body.size
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.WordWrap
                }
            }
        }
    }

    footerLeft: M3Button {
        text: qsTr("Draw again")
        variant: "tonal"
        enabled: root.phase === "revealed"
        onClicked: root.redrawRequested()
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Tie it")
            variant: "tonal"
            visible: root.cursed && root.phase === "revealed"
            onClicked: {
                root.tieRequested(root.game.gameId)
                root.redrawRequested()
            }
        }

        M3Button {
            text: qsTr("Play")
            variant: "filled"
            enabled: root.game !== null && root.phase === "revealed"
            onClicked: {
                root.playRequested(root.drawnIndex)
                root.close()
            }
        }
    }
}
