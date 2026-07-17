import QtQuick
import "../primitives"

Item {
    id: root

    property int from: 0
    property int to: 100
    property int stepSize: 1
    property int value: 0
    property string zeroPlaceholder: ""

    signal moved(int value)

    implicitWidth: boxRow.implicitWidth
    implicitHeight: 36

    function _clamp(v) { return Math.max(root.from, Math.min(root.to, v)) }
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
            radius: theme.radius.xs
            color: stepArea.containsPress
                ? theme.alpha(theme.text, 0.14)
                : (stepArea.containsMouse ? theme.alpha(theme.text, 0.08) : "transparent")

            Behavior on color { ColorAnimation { duration: 100 } }
        }

        SvgIcon {
            anchors.centerIn: parent
            name: btn.icon
            size: 14
            color: stepArea.containsMouse ? theme.text : theme.textMuted
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
    }

    Row {
        id: boxRow
        anchors.fill: parent
        spacing: 0

        StepButton { direction: -1; icon: "remove" }

        Item {
            width: 44
            height: parent.height

            TextInput {
                id: valueInput
                anchors.fill: parent
                text: (root.zeroPlaceholder !== "" && root.value === 0) ? root.zeroPlaceholder : root.value
                color: theme.text
                font.pixelSize: theme.type.body.size
                horizontalAlignment: TextInput.AlignHCenter
                verticalAlignment: TextInput.AlignVCenter
                selectByMouse: true
                inputMethodHints: Qt.ImhDigitsOnly
                validator: IntValidator { bottom: root.from; top: root.to }
                onEditingFinished: {
                    let parsed = parseInt(text, 10)
                    if (isNaN(parsed)) parsed = root.from
                    let clamped = root._clamp(parsed)
                    if (clamped !== root.value) {
                        root.value = clamped
                        root.moved(clamped)
                    }
                    text = (root.zeroPlaceholder !== "" && root.value === 0) ? root.zeroPlaceholder : root.value
                }
                Keys.onUpPressed: root._bump(1)
                Keys.onDownPressed: root._bump(-1)
            }
        }

        StepButton { direction: 1; icon: "add" }
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
