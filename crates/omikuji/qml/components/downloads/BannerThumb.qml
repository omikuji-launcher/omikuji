pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import Qt5Compat.GraphicalEffects
import "../primitives"

Item {
    id: root

    property string source: ""
    property real cornerRadius: Theme.radius.md
    property string fallbackFrom: ""
    property real fallbackTextSize: 30

    readonly property bool ready: img.status === Image.Ready

    Item {
        id: content
        anchors.fill: parent
        visible: root.ready
        layer.enabled: true
        layer.effect: OpacityMask {
            maskSource: Rectangle {
                width: content.width
                height: content.height
                radius: root.cornerRadius
            }
        }

        Image {
            id: img
            anchors.fill: parent
            source: root.visible ? root.source : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: false
        }
    }

    Squircle {
        anchors.fill: parent
        radius: root.cornerRadius
        smoothing: 0.75
        fillColor: Theme.alpha(Theme.text, 0.05)
        visible: !root.ready
    }

    Text {
        anchors.centerIn: parent
        text: root.fallbackFrom ? root.fallbackFrom.charAt(0) : "?"
        color: Theme.textFaint
        font.pixelSize: root.fallbackTextSize
        font.weight: Font.Bold
        visible: !root.ready
    }
}
