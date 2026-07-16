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
    property string cardStyle: "normal"

    readonly property bool imageFit: cardStyle === "fit"
    readonly property bool frameless: cardStyle === "frameless"

    readonly property real imageAspect: bannerImg.implicitWidth > 0 && bannerImg.implicitHeight > 0
        ? bannerImg.implicitWidth / bannerImg.implicitHeight
        : 0

    readonly property real styledHeight: frameless && imageAspect > 0
        ? Math.round(width / imageAspect) + 40
        : (imageFit ? 290 : 240) * (width / 180)

    property string leftIconName: ""
    property int leftIconSize: 20
    property color leftIconColor: theme.icon

    property bool selected: false
    property real selectedBorderWidth: 2
    property color selectedBorderColor: theme.accent
    property color selectedBgTint: "transparent"

    property bool cardVisible: true
    property bool dimmed: false

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

    opacity: !cardVisible ? 0 : dimmed ? 0.55 : 1
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
        scale: root.reordering ? 1.0
            : cardHover.containsPress ? 0.96
            : (cardHover.containsMouse ? 1.02 : 1.0)

        Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

        Item {
            id: bannerClip
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: root.frameless ? 0 : 8
            height: parent.height - (root.frameless ? 40 : 44)

            Rectangle {
                anchors.fill: parent
                color: root.placeholderTint
                radius: theme.radius.md
                visible: !bannerImg.visible
            }

            Item {
                id: imageFrame
                anchors.fill: parent

                layer.enabled: true
                layer.smooth: true
                layer.textureSize: Qt.size(width * 2, height * 2)

                Image {
                    id: bannerImg
                    anchors.fill: parent
                    source: root.imageSource
                    fillMode: root.imageFit ? Image.PreserveAspectFit : Image.PreserveAspectCrop
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

                layer.effect: OpacityMask {
                    maskSource: Item {
                        width: imageFrame.width
                        height: imageFrame.height

                        Rectangle {
                            x: (parent.width - width) / 2
                            y: root.frameless ? 0 : (parent.height - height) / 2
                            width: root.imageFit && !root.frameless && bannerImg.visible
                                ? bannerImg.paintedWidth : parent.width
                            height: root.frameless
                                ? parent.height + radius
                                : root.imageFit && bannerImg.visible ? bannerImg.paintedHeight : parent.height
                            radius: root.frameless ? theme.radius.lg : theme.radius.md
                        }
                    }
                }
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
            border.width: root.selected ? root.selectedBorderWidth : 1
            border.color: root.selected ? root.selectedBorderColor
                : cardHover.containsMouse ? theme.cardBorderHover : theme.cardBorder

            Behavior on border.color { ColorAnimation { duration: 150 } }
            Behavior on border.width { NumberAnimation { duration: 100 } }
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
