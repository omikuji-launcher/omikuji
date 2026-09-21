import QtQuick
import omikuji 1.0

Item {
    id: root

    property color accentColor: Theme.accent
    property bool animated: true

    implicitWidth: 176
    implicitHeight: 145

    property real _look: 0
    property real _blink: 0

    readonly property real _feet: root.height * 0.85
    readonly property bool playing: root.animated && root.visible

    onPlayingChanged: if (!root.playing) {
        root._look = 0
        root._blink = 0
    }

    SequentialAnimation on _look {
        running: root.playing
        loops: Animation.Infinite
        PauseAnimation { duration: 900 }
        NumberAnimation { to: 1; duration: 700; easing.type: Easing.OutBack }
        PauseAnimation { duration: 2400 }
        NumberAnimation { to: 0; duration: 900; easing.type: Easing.InOutQuad }
        PauseAnimation { duration: 1500 }
    }

    // its own loop so the blink drifts against the glance instead of marching with it
    SequentialAnimation on _blink {
        running: root.playing
        loops: Animation.Infinite
        PauseAnimation { duration: 3100 }
        NumberAnimation { to: 1; duration: 90 }
        NumberAnimation { to: 0; duration: 110 }
    }

    PosedSpirit {
        id: spirit

        anchors.fill: parent
        accentColor: root.accentColor
        animated: root.animated
        lookUp: root._look
        squeeze: root._blink

        transform: Rotation {
            origin.x: spirit.width / 2
            origin.y: root._feet
            angle: root._look * 5
        }
    }

    Text {
        id: mark

        x: root.width * 0.72
        y: -root.height * 0.015 - root._look * 6
        text: "?"
        color: root.accentColor
        font.pixelSize: Math.round(root.height * 0.38)
        font.weight: Font.DemiBold
        opacity: root._look
        scale: 0.6 + 0.4 * root._look
        transformOrigin: Item.Bottom
    }
}
