pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property string icon: ""
    property string monogram: ""
    property string badge: ""
    property url source: ""
    property bool cache: true
    property int iconSize: 18
    property real radius: Theme.radius.sm
    property color tint: Theme.secondary

    readonly property bool hasImage: img.status === Image.Ready

    implicitWidth: 36
    implicitHeight: 36

    Squircle {
        anchors.fill: parent
        visible: !root.hasImage
        radius: root.radius
        fillColor: Theme.alpha(root.tint, 0.15)
    }

    SvgIcon {
        anchors.centerIn: parent
        visible: !root.hasImage && root.icon !== ""
        name: root.icon
        size: root.iconSize
        color: root.tint
    }

    Text {
        anchors.centerIn: parent
        visible: !root.hasImage && root.icon === ""
        text: root.badge !== "" ? root.badge : root.monogram.charAt(0).toUpperCase()
        color: root.tint
        font.pixelSize: Theme.type.title.size
        font.weight: Font.DemiBold
    }

    Image {
        id: img
        anchors.fill: parent
        visible: root.hasImage
        source: root.source
        cache: root.cache
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        sourceSize.width: root.width * 2
        sourceSize.height: root.height * 2
        layer.enabled: visible
        layer.smooth: true
        layer.effect: RoundedRectMask {
            radius: root.radius
        }
    }
}
