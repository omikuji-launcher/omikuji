pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../primitives"

Item {
    id: root

    property real from: 0
    property real to: 100
    property real stepSize: 1
    property real value: 0
    property int decimals: 0
    property string zeroPlaceholder: ""
    property string suffix: ""

    signal moved(real value)

    implicitWidth: boxRow.implicitWidth
    implicitHeight: 36

    readonly property bool _showingPlaceholder: zeroPlaceholder !== "" && value === 0
    readonly property string _displayText: _showingPlaceholder ? zeroPlaceholder : root.value.toFixed(root.decimals)

    function _clamp(v) { return Number(Math.max(root.from, Math.min(root.to, v)).toFixed(root.decimals)) }
    function _bump(delta) {
        let next = _clamp(root.value + delta * root.stepSize)
        if (next === root.value) return
        root.value = next
        root.moved(next)
    }

    component StepButton: Item {
        id: btn

        property int direction: 1
        property string icon: "add"
        readonly property bool canStep: direction > 0 ? root.value < root.to : root.value > root.from

        width: 24
        height: parent.height
        opacity: canStep ? 1 : 0.4

        Rectangle {
            anchors.centerIn: parent
            width: 20
            height: 20
            radius: Theme.radius.xs
            color: stepArea.containsPress
                ? Theme.alpha(Theme.text, 0.14)
                : (stepArea.containsMouse ? Theme.alpha(Theme.text, 0.08) : "transparent")

            Behavior on color { ColorAnimation { duration: 100 } }
        }

        SvgIcon {
            anchors.centerIn: parent
            name: btn.icon
            size: 14
            color: stepArea.containsMouse ? Theme.text : Theme.textMuted
        }

        MouseArea {
            id: stepArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: btn.canStep ? Qt.PointingHandCursor : Qt.ArrowCursor
            onClicked: root._bump(btn.direction)
            onPressAndHold: holdTimer.startWith(btn.direction)
            onReleased: holdTimer.stop()
            onExited: holdTimer.stop()
        }
    }

    FieldSurface {
        anchors.fill: parent
        focused: valueInput.activeFocus
        squareRight: suffixChip.visible
    }

    Row {
        id: boxRow
        anchors.fill: parent
        spacing: 0

        StepButton { direction: -1; icon: "remove" }

        Item {
            width: Math.max(44, valueInput.implicitWidth + Theme.space.sm * 2)
            height: parent.height

            TextInput {
                id: valueInput
                anchors.fill: parent
                text: root._displayText
                color: Theme.text
                font.pixelSize: Theme.type.body.size
                horizontalAlignment: TextInput.AlignHCenter
                verticalAlignment: TextInput.AlignVCenter
                selectByMouse: true
                inputMethodHints: root.decimals > 0 ? Qt.ImhFormattedNumbersOnly : Qt.ImhDigitsOnly
                validator: RegularExpressionValidator {
                    regularExpression: root.decimals > 0 ? /[0-9]*[.,]?[0-9]*/ : /[0-9]*/
                }
                onEditingFinished: {
                    let parsed = parseFloat(text.replace(",", "."))
                    if (isNaN(parsed)) parsed = root.from
                    let clamped = root._clamp(parsed)
                    if (clamped !== root.value) {
                        root.value = clamped
                        root.moved(clamped)
                    }
                    text = Qt.binding(() => root._displayText)
                }
                Keys.onUpPressed: root._bump(1)
                Keys.onDownPressed: root._bump(-1)
            }
        }

        StepButton { direction: 1; icon: "add" }
    }

    Item {
        id: suffixChip

        anchors.left: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        visible: root.suffix !== "" && !root._showingPlaceholder
        width: visible ? suffixText.implicitWidth + Theme.space.sm * 2 : 0

        Rectangle {
            anchors.fill: parent
            anchors.topMargin: Theme.fillFields ? 0 : 1
            anchors.bottomMargin: Theme.fillFields ? 0 : 1
            topRightRadius: Theme.radius.sm
            bottomRightRadius: Theme.radius.sm
            color: Theme.alpha(Theme.text, 0.07)
        }

        Text {
            id: suffixText
            anchors.centerIn: parent
            text: root.suffix
            color: Theme.textMuted
            font.pixelSize: Theme.type.label.size
        }
    }

    Timer {
        id: holdTimer
        property int direction: 0
        interval: 80
        repeat: true
        onTriggered: root._bump(direction)
        function startWith(d) { direction = d; start() }
    }
}
