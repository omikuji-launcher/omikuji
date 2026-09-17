pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Squircle {
    id: root

    property var options: []
    property int currentIndex: 0

    signal selected(int index)

    readonly property int inset: 4
    readonly property real segWidth: options.length > 0
        ? (width - inset * 2) / options.length
        : 0

    implicitHeight: 44
    radius: Theme.radius.md
    fillColor: Theme.alpha(Theme.text, 0.05)

    readonly property bool navigable: options.length > 0
    readonly property real navRingRadius: radius
    Keys.onPressed: (event) => {
        const step = event.key === Qt.Key_Right ? 1 : (event.key === Qt.Key_Left ? -1 : 0)
        const next = currentIndex + step
        event.accepted = step !== 0 && next >= 0 && next < options.length
        if (event.accepted) selected(next)
    }

    Squircle {
        x: root.inset + root.currentIndex * root.segWidth
        y: root.inset
        width: root.segWidth
        height: root.height - root.inset * 2
        radius: Theme.radius.sm
        fillColor: Theme.alpha(Theme.accent, 0.18)
        visible: root.options.length > 0

        Behavior on x {
            NumberAnimation {
                duration: Theme.dur.med
                easing.type: Theme.ease.standard
            }
        }
    }

    Row {
        id: track
        anchors.fill: parent
        anchors.margins: root.inset

        Repeater {
            model: root.options

            Item {
                id: seg

                required property var modelData
                required property int index

                readonly property bool current: seg.index === root.currentIndex

                width: root.segWidth
                height: track.height

                Squircle {
                    anchors.fill: parent
                    radius: Theme.radius.sm
                    visible: !seg.current && segHover.containsMouse
                    fillColor: Theme.alpha(Theme.text, 0.06)
                }

                Text {
                    anchors.centerIn: parent
                    width: parent.width - Theme.space.sm * 2
                    horizontalAlignment: Text.AlignHCenter
                    elide: Text.ElideRight
                    text: seg.modelData.label !== undefined ? seg.modelData.label : seg.modelData
                    color: seg.current ? Theme.accent : Theme.textMuted
                    font.pixelSize: Theme.type.label.size
                    font.weight: seg.current ? Font.DemiBold : Font.Normal

                    Behavior on color {
                        ColorAnimation { duration: Theme.dur.fast }
                    }
                }

                MouseArea {
                    id: segHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.selected(seg.index)
                }
            }
        }
    }
}
