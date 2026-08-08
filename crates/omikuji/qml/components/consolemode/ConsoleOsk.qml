pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: osk

    property real uiScale: 1.0

    signal keyPressed(string ch)
    signal backspaceRequested()
    signal spaceRequested()
    signal submitRequested()

    readonly property var _layout: [
        ["1","2","3","4","5","6","7","8","9","0"],
        ["q","w","e","r","t","y","u","i","o","p"],
        ["a","s","d","f","g","h","j","k","l","-"],
        ["z","x","c","v","b","n","m",",",".","_"]
    ]

    property int cursorRow: 1
    property int cursorCol: 0

    readonly property int letterCols: 10
    readonly property int letterRows: _layout.length
    readonly property int actionRow: _layout.length

    readonly property int keySize: 44 * uiScale
    readonly property int keySpacing: 5 * uiScale
    readonly property int gridWidth: letterCols * keySize + (letterCols - 1) * keySpacing
    readonly property int gridHeight: (letterRows + 1) * keySize + letterRows * keySpacing

    implicitWidth: gridWidth + 28 * uiScale * 2
    implicitHeight: gridHeight + 28 * uiScale * 2

    function moveLeft() {
        if (cursorRow === actionRow) {
            cursorCol = (cursorCol - 1 + 3) % 3
        } else {
            cursorCol = (cursorCol - 1 + letterCols) % letterCols
        }
    }

    function moveRight() {
        if (cursorRow === actionRow) {
            cursorCol = (cursorCol + 1) % 3
        } else {
            cursorCol = (cursorCol + 1) % letterCols
        }
    }

    function moveUp() {
        if (cursorRow === actionRow) {
            if (cursorCol === 0) cursorCol = 1
            else if (cursorCol === 1) cursorCol = 5
            else cursorCol = 8
            cursorRow = letterRows - 1
            return
        }
        cursorRow = (cursorRow - 1 + (actionRow + 1)) % (actionRow + 1)
    }

    function moveDown() {
        if (cursorRow === letterRows - 1) {
            if (cursorCol <= 2) cursorCol = 0
            else if (cursorCol <= 7) cursorCol = 1
            else cursorCol = 2
            cursorRow = actionRow
            return
        }
        cursorRow = (cursorRow + 1) % (actionRow + 1)
    }

    function tapFocused() {
        if (cursorRow === actionRow) {
            if (cursorCol === 0) osk.backspaceRequested()
            else if (cursorCol === 1) osk.spaceRequested()
            else osk.submitRequested()
            return
        }
        let ch = _layout[cursorRow][cursorCol]
        if (ch) osk.keyPressed(ch)
    }

    Rectangle {
        anchors.fill: parent
        color: Qt.darker(Theme.surface, 1.45)
        radius: 14 * osk.uiScale
        border.width: 1
        border.color: Theme.alpha(Theme.text, 0.08)
    }

    Column {
        anchors.centerIn: parent
        spacing: osk.keySpacing

        Repeater {
            model: osk.letterRows
            delegate: Row {
                id: letterRow
                required property int index
                spacing: osk.keySpacing

                Repeater {
                    model: osk.letterCols
                    delegate: Item {
                        id: keyCell
                        required property int index
                        readonly property int row: letterRow.index
                        readonly property int col: index
                        readonly property string ch: osk._layout[row][col]
                        readonly property bool isFocused: row === osk.cursorRow && col === osk.cursorCol

                        width: osk.keySize
                        height: osk.keySize

                        Rectangle {
                            anchors.fill: parent
                            radius: 7 * osk.uiScale
                            color: keyCell.isFocused
                                ? Theme.accent
                                : (keyMouse.containsMouse ? Qt.darker(Theme.surface, 1.15) : Qt.darker(Theme.surface, 1.25))
                            scale: keyMouse.containsPress ? 0.94 : 1.0
                            Behavior on color { ColorAnimation { duration: 120 } }
                            Behavior on scale { NumberAnimation { duration: 100 } }
                        }

                        Text {
                            anchors.centerIn: parent
                            text: keyCell.ch
                            color: keyCell.isFocused ? Theme.accentOn : Theme.text
                            font.pixelSize: 17 * osk.uiScale
                            font.weight: Font.Medium
                        }

                        MouseArea {
                            id: keyMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                osk.cursorRow = keyCell.row
                                osk.cursorCol = keyCell.col
                                osk.keyPressed(keyCell.ch)
                            }
                        }
                    }
                }
            }
        }

        Row {
            spacing: osk.keySpacing

            Item {
                id: bsBtn
                width: osk.keySize * 2.5 + osk.keySpacing * 2
                height: osk.keySize
                readonly property bool isFocused: osk.cursorRow === osk.actionRow && osk.cursorCol === 0

                Rectangle {
                    anchors.fill: parent
                    radius: 7 * osk.uiScale
                    color: bsBtn.isFocused
                        ? Theme.accent
                        : (bsMouse.containsMouse ? Qt.darker(Theme.surface, 1.15) : Qt.darker(Theme.surface, 1.25))
                    scale: bsMouse.containsPress ? 0.96 : 1.0
                    Behavior on color { ColorAnimation { duration: 120 } }
                    Behavior on scale { NumberAnimation { duration: 100 } }
                }

                Text {
                    anchors.centerIn: parent
                    text: qsTr("Backspace")
                    color: bsBtn.isFocused ? Theme.accentOn : Theme.text
                    font.pixelSize: 14 * osk.uiScale
                    font.weight: Font.Medium
                }

                MouseArea {
                    id: bsMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        osk.cursorRow = osk.actionRow
                        osk.cursorCol = 0
                        osk.backspaceRequested()
                    }
                }
            }

            Item {
                id: spaceBtn
                width: osk.keySize * 5 + osk.keySpacing * 4
                height: osk.keySize
                readonly property bool isFocused: osk.cursorRow === osk.actionRow && osk.cursorCol === 1

                Rectangle {
                    anchors.fill: parent
                    radius: 7 * osk.uiScale
                    color: spaceBtn.isFocused
                        ? Theme.accent
                        : (spaceMouse.containsMouse ? Qt.darker(Theme.surface, 1.15) : Qt.darker(Theme.surface, 1.25))
                    scale: spaceMouse.containsPress ? 0.96 : 1.0
                    Behavior on color { ColorAnimation { duration: 120 } }
                    Behavior on scale { NumberAnimation { duration: 100 } }
                }

                Text {
                    anchors.centerIn: parent
                    text: qsTr("Space")
                    color: spaceBtn.isFocused ? Theme.accentOn : Theme.text
                    font.pixelSize: 14 * osk.uiScale
                    font.weight: Font.Medium
                }

                MouseArea {
                    id: spaceMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        osk.cursorRow = osk.actionRow
                        osk.cursorCol = 1
                        osk.spaceRequested()
                    }
                }
            }

            Item {
                id: enterBtn
                width: osk.keySize * 2.5 + osk.keySpacing * 2
                height: osk.keySize
                readonly property bool isFocused: osk.cursorRow === osk.actionRow && osk.cursorCol === 2

                Rectangle {
                    anchors.fill: parent
                    radius: 7 * osk.uiScale
                    color: enterBtn.isFocused
                        ? Theme.accent
                        : (enterMouse.containsMouse ? Qt.darker(Theme.surface, 1.15) : Qt.darker(Theme.surface, 1.25))
                    scale: enterMouse.containsPress ? 0.96 : 1.0
                    Behavior on color { ColorAnimation { duration: 120 } }
                    Behavior on scale { NumberAnimation { duration: 100 } }
                }

                Text {
                    anchors.centerIn: parent
                    text: qsTr("Enter")
                    color: enterBtn.isFocused ? Theme.accentOn : Theme.text
                    font.pixelSize: 14 * osk.uiScale
                    font.weight: Font.Medium
                }

                MouseArea {
                    id: enterMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        osk.cursorRow = osk.actionRow
                        osk.cursorCol = 2
                        osk.submitRequested()
                    }
                }
            }
        }
    }
}
