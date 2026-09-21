import QtQuick
import omikuji 1.0

Item {
    id: root

    property color accentColor: Theme.accent
    property color markColor: Theme.error
    property real lid: 0.25
    property real dread: 1
    property string mark: "bang"
    property bool animated: true

    implicitWidth: 62
    implicitHeight: 68

    property real _blink: 0

    readonly property bool playing: root.animated && root.visible

    onPlayingChanged: if (!root.playing)
        root._blink = 0

    SequentialAnimation on _blink {
        running: root.playing
        loops: Animation.Infinite
        PauseAnimation { duration: 2700 }
        NumberAnimation { to: 1; duration: 90 }
        NumberAnimation { to: 0; duration: 110 }
    }

    PosedSpirit {
        id: spirit

        anchors.fill: parent
        anchors.topMargin: Math.round(root.height * 0.03)
        anchors.rightMargin: root.mark === "bang" ? Math.round(root.width * 0.18) : 0
        accentColor: root.accentColor
        animated: root.animated
        lookUp: -1
        dread: root.dread
        squeeze: Math.max(root.lid, root._blink)

        transform: Scale {
            origin.x: spirit.width / 2
            origin.y: spirit.height * 0.9
            xScale: 1.05
            yScale: 0.94
        }
    }

    Text {
        id: bang

        x: root.width * 0.54
        y: -root.height * 0.04
        visible: root.mark === "bang"
        text: "!"
        color: root.markColor
        font.pixelSize: Math.round(root.height * 0.52)
        font.weight: Font.Black

        transform: [
            Scale {
                origin.x: bang.width / 2
                origin.y: bang.height
                xScale: 1.35
            },
            Rotation {
                origin.x: bang.width / 2
                origin.y: bang.height
                angle: 8
            }
        ]
    }

    AngerMark {
        x: root.width * 0.60
        y: root.height * 0.19
        width: Math.round(root.height * 0.26)
        height: width
        visible: root.mark === "anger"
        markColor: root.markColor
    }
}
