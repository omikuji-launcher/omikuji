import QtQuick
import QtQuick.Effects
import Qt5Compat.GraphicalEffects
import "../popups"
import "../primitives"

Item {
    id: root

    property string title: ""
    property url imageSource: ""
    property url imageFallback: ""
    property color placeholderTint: "#1a1a2e"
    property string letter: ""
    property int letterFontSize: 48
    property color letterColor: theme.textFaint
    property real imageOpacity: 1.0

    property string leftIconName: ""
    property int leftIconSize: 20
    property color leftIconColor: theme.icon

    property bool selected: false
    property real selectedBorderWidth: 2
    property color selectedBorderColor: theme.accent
    property color selectedBgTint: "transparent"

    property bool cardVisible: true

    property bool elevation: false

    property bool clickable: true
    property bool contextEnabled: false
    property bool reorderable: false
    property bool reordering: false

    property Component actionComponent: null
    property Component overlayComponent: null

    signal clicked()
    signal doubleClicked()
    signal rightClicked(real winX, real winY)
    signal reorderStarted(real grabX, real grabY)
    signal reorderMoved(real winX, real winY)
    signal reorderEnded()

    readonly property bool nameHovered: {
        if (!cardHover.containsMouse) return false
        let p = nameLabel.mapFromItem(cardHover, cardHover.mouseX, cardHover.mouseY)
        return p.x >= 0 && p.x <= nameLabel.width
            && p.y >= 0 && p.y <= nameLabel.height
    }

    implicitWidth: 180
    implicitHeight: 240

    opacity: cardVisible ? 1 : 0
    visible: opacity > 0.01
    Behavior on opacity { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }

    // declared before frame so it renders under in paint order, only the halo that bleeds into paddingRect ends up visible
    MultiEffect {
        anchors.fill: frame
        source: frame
        visible: root.elevation && opacity > 0.01
        opacity: root.elevation ? 1 : 0
        paddingRect: Qt.rect(24, 16, 24, 28)
        shadowEnabled: true
        shadowHorizontalOffset: 0
        shadowVerticalOffset: 8
        shadowBlur: 1.0
        shadowColor: Qt.rgba(0, 0, 0, 0.45)
        scale: frame.scale
        transformOrigin: Item.Center
        Behavior on opacity { NumberAnimation { duration: 150 } }
    }

    Rectangle {
        id: frame
        anchors.fill: parent
        radius: theme.radius.lg
        color: root.selected && root.selectedBgTint.a > 0
            ? root.selectedBgTint
            : theme.cardBg
        border.width: root.selected ? root.selectedBorderWidth : 0
        border.color: root.selectedBorderColor
        scale: root.reordering ? 1.0
            : cardHover.containsPress ? 0.96
            : (cardHover.containsMouse ? 1.02 : 1.0)

        Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }
        Behavior on border.width { NumberAnimation { duration: 100 } }

        Item {
            id: bannerClip
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: 8
            height: parent.height - 44
            layer.enabled: true
            layer.smooth: true
            layer.textureSize: Qt.size(width * 2, height * 2)

            Rectangle {
                anchors.fill: parent
                color: root.placeholderTint
                radius: theme.radius.md
            }

            Image {
                id: bannerImg
                anchors.fill: parent
                source: root.imageSource
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
                sourceSize.width: 360
                sourceSize.height: 480
                onStatusChanged: {
                    if (status === Image.Error && root.imageFallback != "" && source != root.imageFallback) {
                        source = root.imageFallback
                    }
                }
                // without cache false QQuickPixmapCache holds decoded pixmaps long after the Image dies, so idle-unloading a store tab doesnt free RAM. some days update: IT STILL DOESNT FREE THE RAM. Fuck you Qt.
                cache: false
                visible: status === Image.Ready
                opacity: root.imageOpacity
            }

            Text {
                anchors.centerIn: parent
                text: root.letter !== ""
                    ? root.letter
                    : (root.title.length > 0 ? root.title.charAt(0) : "")
                color: root.letterColor
                font.pixelSize: root.letterFontSize
                font.weight: Font.Bold
                visible: !bannerImg.visible
            }

            layer.effect: OpacityMask {
                maskSource: Rectangle {
                    width: bannerClip.width
                    height: bannerClip.height
                    radius: theme.radius.md
                }
            }
        }

        Item {
            id: nameRow
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 10
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            height: 20

            SvgIcon {
                id: leftIcon
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                name: root.leftIconName
                size: root.leftIconSize
                color: root.leftIconColor
                visible: root.leftIconName !== ""
            }

            Loader {
                id: actionLoader
                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                sourceComponent: root.actionComponent
                active: root.actionComponent !== null
            }

            readonly property real leftReserve: leftIcon.visible ? leftIcon.width + 4 : 0
            readonly property real rightReserve: actionLoader.active ? actionLoader.width + 4 : 0
            readonly property real reserve: Math.max(leftReserve, rightReserve)

            Text {
                id: nameLabel
                anchors.verticalCenter: parent.verticalCenter
                anchors.horizontalCenter: parent.horizontalCenter
                width: parent.width - nameRow.reserve * 2
                horizontalAlignment: Text.AlignHCenter
                text: root.title
                color: theme.text
                font.pixelSize: 13
                font.weight: Font.Medium
                elide: Text.ElideRight
                maximumLineCount: 1

                Tooltip {
                    text: root.title
                    tipVisible: nameLabel.truncated && root.nameHovered
                }
            }
        }

        Loader {
            id: overlayLoader
            anchors.fill: parent
            sourceComponent: root.overlayComponent
            active: root.overlayComponent !== null
        }

        Rectangle {
            anchors.fill: parent
            radius: theme.radius.lg
            color: "transparent"
            border.width: 1
            border.color: cardHover.containsMouse ? theme.cardBorderHover : theme.cardBorder

            Behavior on border.color { ColorAnimation { duration: 150 } }
        }
    }

    MouseArea {
        id: cardHover
        anchors.fill: parent
        hoverEnabled: root.cardVisible
        enabled: root.cardVisible
        cursorShape: root.reordering ? Qt.ClosedHandCursor
            : root.cardVisible && root.clickable
            ? Qt.PointingHandCursor
            : Qt.ArrowCursor
        acceptedButtons: root.clickable
            ? (root.contextEnabled ? Qt.LeftButton | Qt.RightButton : Qt.LeftButton)
            : Qt.NoButton
        preventStealing: root.reordering

        onClicked: (mouse) => {
            if (!root.cardVisible) return
            if (mouse.wasHeld && root.reorderable) return
            mouse.accepted = true
            if (mouse.button === Qt.RightButton) {
                let winPos = root.mapToItem(null, mouse.x, mouse.y)
                root.rightClicked(winPos.x, winPos.y)
            } else {
                root.clicked()
            }
        }

        onDoubleClicked: (mouse) => {
            if (!root.cardVisible) return
            if (mouse.button !== Qt.LeftButton) return
            mouse.accepted = true
            root.doubleClicked()
        }

        onPressAndHold: (mouse) => {
            if (!root.reorderable || mouse.button !== Qt.LeftButton) return
            root.reordering = true
            root.reorderStarted(mouse.x, mouse.y)
        }

        onPositionChanged: (mouse) => {
            if (!root.reordering) return
            let p = root.mapToItem(null, mouse.x, mouse.y)
            root.reorderMoved(p.x, p.y)
        }

        onReleased: root._finishReorder()
        onCanceled: root._finishReorder()
    }

    function _finishReorder() {
        if (!reordering) return
        reordering = false
        reorderEnded()
    }
}
