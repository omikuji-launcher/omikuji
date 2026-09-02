pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Effects
import omikuji 1.0

Item {
    id: root

    property string icon: ""
    property url art
    property string text: ""
    property string hint: ""
    property int textSize: Theme.type.title.size
    property int hintSize: Theme.type.body.size
    property int artSize: 48
    property color tint: Theme.textFaint

    readonly property bool _hasArt: String(root.art) !== ""

    Column {
        anchors.centerIn: parent
        spacing: 10

        SvgIcon {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: !root._hasArt && root.icon !== ""
            name: root.icon
            size: root.artSize
            color: root.tint
        }

        Image {
            id: artImg
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root._hasArt
            source: root.art
            width: root.artSize
            height: sourceSize.width > 0
                ? Math.round(width * sourceSize.height / sourceSize.width)
                : 0
            fillMode: Image.PreserveAspectFit
            smooth: false
            layer.enabled: visible
            layer.effect: MultiEffect {
                contrast: -1
                brightness: 0.5
                colorization: 1
                colorizationColor: Qt.rgba(root.tint.r, root.tint.g, root.tint.b, 1)
                opacity: root.tint.a
            }
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.text !== ""
            text: root.text
            color: Theme.textMuted
            font.pixelSize: root.textSize
            font.weight: Font.Medium
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.hint !== ""
            text: root.hint
            width: Math.min(root.width - 48, 420)
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            color: Theme.textFaint
            font.pixelSize: root.hintSize
            font.weight: Theme.type.body.weight
        }
    }
}
