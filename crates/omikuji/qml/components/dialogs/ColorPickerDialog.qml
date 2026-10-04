pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import omikuji 1.0

DialogCard {
    id: root

    property color initial
    property color current
    property real hue: 0
    property real saturation: 0
    property real value: 0
    property var _onPicked: null

    readonly property real _keyStep: 0.05

    title: qsTr("Pick a color")
    maxWidth: 380

    onCloseRequested: close()

    function pick(color, onPicked) {
        initial = color
        _setColor(initial)
        _onPicked = onPicked
        open()
    }

    function _clamp01(x) {
        return Math.max(0, Math.min(1, x))
    }

    function _setColor(c) {
        current = c
        if (c.hsvHue >= 0) hue = c.hsvHue
        saturation = c.hsvSaturation
        value = c.hsvValue
    }

    function _setHsv(h, s, v) {
        hue = _clamp01(h)
        saturation = _clamp01(s)
        value = _clamp01(v)
        current = Qt.hsva(hue, saturation, value, 1)
    }

    function _apply() {
        _onPicked(current.toString())
        close()
    }

    component Knob: Rectangle {
        width: 18
        height: 18
        radius: width / 2
        border.width: 2
        border.color: "white"

        Rectangle {
            anchors.fill: parent
            anchors.margins: -1
            z: -1
            radius: width / 2
            color: "transparent"
            border.width: 1
            border.color: Qt.rgba(0, 0, 0, 0.4)
        }
    }

    body: Column {
        width: parent.width
        spacing: Theme.space.lg

        Item {
            id: svArea
            width: parent.width
            height: 200

            readonly property bool navigable: true
            readonly property real navRingRadius: Theme.radius.md

            Keys.onPressed: (event) => {
                const ds = event.key === Qt.Key_Right ? 1 : (event.key === Qt.Key_Left ? -1 : 0)
                const dv = event.key === Qt.Key_Up ? 1 : (event.key === Qt.Key_Down ? -1 : 0)
                const s = root._clamp01(root.saturation + ds * root._keyStep)
                const v = root._clamp01(root.value + dv * root._keyStep)
                event.accepted = s !== root.saturation || v !== root.value
                if (event.accepted) root._setHsv(root.hue, s, v)
            }

            Rectangle {
                anchors.fill: parent
                color: Qt.hsva(root.hue, 1, 1, 1)
                layer.enabled: true
                layer.smooth: true
                layer.effect: RoundedRectMask {
                    radius: Theme.radius.md
                }

                Rectangle {
                    anchors.fill: parent
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0; color: "white" }
                        GradientStop { position: 1; color: "transparent" }
                    }
                }

                Rectangle {
                    anchors.fill: parent
                    gradient: Gradient {
                        GradientStop { position: 0; color: "transparent" }
                        GradientStop { position: 1; color: "black" }
                    }
                }
            }

            Knob {
                x: root.saturation * svArea.width - width / 2
                y: (1 - root.value) * svArea.height - height / 2
                color: root.current
            }

            MouseArea {
                anchors.fill: parent
                preventStealing: true
                cursorShape: Qt.CrossCursor

                function pickAt(mx, my) {
                    root._setHsv(root.hue, mx / width, 1 - my / height)
                }

                onPressed: (mouse) => pickAt(mouse.x, mouse.y)
                onPositionChanged: (mouse) => { if (pressed) pickAt(mouse.x, mouse.y) }
            }
        }

        Item {
            id: hueStrip
            width: parent.width
            height: 20

            readonly property bool navigable: true
            readonly property real navRingRadius: hueTrack.radius
            function navRectItem() { return hueTrack }

            Keys.onPressed: (event) => {
                const dir = event.key === Qt.Key_Right ? 1 : (event.key === Qt.Key_Left ? -1 : 0)
                const h = root._clamp01(root.hue + dir * root._keyStep)
                event.accepted = h !== root.hue
                if (event.accepted) root._setHsv(h, root.saturation, root.value)
            }

            Rectangle {
                id: hueTrack
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width
                height: 12
                radius: height / 2
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    // are we genuinely serious
                    GradientStop { position: 0 / 6; color: Qt.hsva(0 / 6, 1, 1, 1) }
                    GradientStop { position: 1 / 6; color: Qt.hsva(1 / 6, 1, 1, 1) }
                    GradientStop { position: 2 / 6; color: Qt.hsva(2 / 6, 1, 1, 1) }
                    GradientStop { position: 3 / 6; color: Qt.hsva(3 / 6, 1, 1, 1) }
                    GradientStop { position: 4 / 6; color: Qt.hsva(4 / 6, 1, 1, 1) }
                    GradientStop { position: 5 / 6; color: Qt.hsva(5 / 6, 1, 1, 1) }
                    GradientStop { position: 6 / 6; color: Qt.hsva(0, 1, 1, 1) }
                }
            }

            Knob {
                anchors.verticalCenter: parent.verticalCenter
                x: root.hue * hueStrip.width - width / 2
                color: Qt.hsva(root.hue, 1, 1, 1)
            }

            MouseArea {
                anchors.fill: parent
                preventStealing: true
                cursorShape: Qt.PointingHandCursor

                onPressed: (mouse) => root._setHsv(mouse.x / width, root.saturation, root.value)
                onPositionChanged: (mouse) => { if (pressed) root._setHsv(mouse.x / width, root.saturation, root.value) }
            }
        }

        RowLayout {
            width: parent.width
            spacing: Theme.space.sm

            ColorSwatch {
                fillColor: root.initial
            }

            ColorSwatch {
                fillColor: root.current
            }

            M3TextField {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.space.sm
                text: root.current.toString()
                onTextEdited: (t) => {
                    if (/^#?[0-9a-fA-F]{6}$/.test(t)) root._setColor(Qt.color(t.startsWith("#") ? t : "#" + t))
                }
                onAccepted: root._apply()
            }
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            variant: "text"
            text: qsTr("Cancel")
            onClicked: root.close()
        }

        M3Button {
            variant: "filled"
            text: qsTr("Apply")
            onClicked: root._apply()
        }
    }
}
