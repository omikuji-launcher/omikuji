pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import Qt5Compat.GraphicalEffects

Item {
    id: card

    property bool focused: false
    property string title: ""
    property url bannerSource: ""
    property url coverartSource: ""
    property color tint: Theme.accent
    property real uiScale: 1.0

    signal focusRequested()
    signal launchRequested()

    readonly property int unfocusedWidth: 220 * uiScale
    readonly property int focusedWidth: 660 * uiScale
    readonly property int cardHeight: 370 * uiScale
    readonly property int cardRadius: 14 * uiScale

    width: focused ? focusedWidth : unfocusedWidth
    height: cardHeight

    Behavior on width { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }

    Item {
        id: frame
        anchors.fill: parent

        layer.enabled: true
        layer.smooth: true
        layer.textureSize: Qt.size(width * 2, height * 2)

        Rectangle {
            anchors.fill: parent
            color: Qt.darker(card.tint, 2.8)
            radius: card.cardRadius
        }

        Image {
            id: coverartImg
            anchors.fill: parent
            source: card.coverartSource
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: true
            visible: status === Image.Ready
            opacity: card.focused ? 0 : 1
            Behavior on opacity { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }
        }

        Image {
            id: bannerImg
            anchors.fill: parent
            source: card.bannerSource
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            cache: true
            visible: status === Image.Ready
            opacity: card.focused ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }
        }

        Text {
            anchors.centerIn: parent
            text: card.title.length > 0 ? card.title.charAt(0) : "?"
            color: Theme.textFaint
            font.pixelSize: 64 * card.uiScale
            font.weight: Font.Bold
            visible: !coverartImg.visible && !bannerImg.visible
        }

        layer.effect: OpacityMask {
            maskSource: Rectangle {
                width: frame.width
                height: frame.height
                radius: card.cardRadius
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: card.focusRequested()
        onDoubleClicked: card.launchRequested()
    }
}
