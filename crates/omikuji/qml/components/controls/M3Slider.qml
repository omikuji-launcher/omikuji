import QtQuick

Item {
    id: root

    property real from: 0
    property real to: 100
    property real stepSize: 1
    property real value: 0
    property string label: ""
    property bool showValue: true
    property string valueText: String(Math.round(value))

    signal moved(real value)

    property real trackThickness: 12
    property real handleRestWidth: 4
    property real handlePressedWidth: 2
    property real handleOverhang: 5
    property real gap: 6
    property real outerRadius: 8
    property real innerRadius: 2
    property real stopDotSize: 4

    implicitWidth: 200
    implicitHeight: header.visible
        ? header.height + 8 + slider.height
        : slider.height

    Item {
        id: header
        visible: root.label !== "" || root.showValue
        anchors.left: parent.left
        anchors.right: parent.right
        height: childrenRect.height

        Text {
            visible: root.label !== ""
            anchors.left: parent.left
            text: root.label
            color: theme.textMuted
            font.pixelSize: theme.type.label.size
            font.weight: Font.Medium
        }
        Text {
            visible: root.showValue
            anchors.right: parent.right
            text: root.valueText
            color: theme.text
            font.pixelSize: theme.type.label.size
        }
    }

    Item {
        id: slider
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: root.trackThickness + root.handleOverhang * 2

        readonly property real normalizedValue: root.to > root.from
            ? (root.value - root.from) / (root.to - root.from)
            : 0
        readonly property real currentHandleWidth: dragArea.pressed ? root.handlePressedWidth : root.handleRestWidth
        readonly property real handleX: slider.normalizedValue * (slider.width - currentHandleWidth)
        readonly property color trackBg: theme.alpha(theme.text, 0.16)

        Rectangle {
            anchors.verticalCenter: parent.verticalCenter
            x: 0
            width: Math.max(0, slider.handleX - root.gap)
            height: root.trackThickness
            color: theme.accent
            topLeftRadius: root.outerRadius
            bottomLeftRadius: root.outerRadius
            topRightRadius: root.innerRadius
            bottomRightRadius: root.innerRadius
        }

        Rectangle {
            id: rightTrack
            anchors.verticalCenter: parent.verticalCenter
            x: Math.min(slider.width, slider.handleX + slider.currentHandleWidth + root.gap)
            width: Math.max(0, slider.width - x)
            height: root.trackThickness
            color: slider.trackBg
            topLeftRadius: root.innerRadius
            bottomLeftRadius: root.innerRadius
            topRightRadius: root.outerRadius
            bottomRightRadius: root.outerRadius

            Rectangle {
                visible: parent.width > 12
                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: 6
                width: root.stopDotSize
                height: root.stopDotSize
                radius: width / 2
                color: theme.alpha(theme.text, 0.45)
            }
        }

        Rectangle {
            x: slider.handleX
            anchors.verticalCenter: parent.verticalCenter
            width: slider.currentHandleWidth
            height: parent.height
            radius: width / 2
            color: theme.accent

            Behavior on width {
                NumberAnimation { duration: 100; easing.type: Easing.OutCubic }
            }
        }

        MouseArea {
            id: dragArea
            anchors.fill: parent
            anchors.margins: -8
            cursorShape: Qt.PointingHandCursor
            hoverEnabled: true

            onPressed: (mouse) => updateValue(mouse.x)
            onPositionChanged: (mouse) => { if (pressed) updateValue(mouse.x) }

            function updateValue(mx) {
                let usable = slider.width
                let normalized = Math.max(0, Math.min(1, (mx - 8) / usable))
                let raw = root.from + normalized * (root.to - root.from)
                if (root.stepSize > 0) {
                    raw = Math.round(raw / root.stepSize) * root.stepSize
                }
                root.value = Math.max(root.from, Math.min(root.to, raw))
                root.moved(root.value)
            }
        }
    }
}
