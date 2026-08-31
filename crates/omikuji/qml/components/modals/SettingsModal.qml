import QtQuick
import omikuji 1.0
import QtQuick.Controls
import Qt5Compat.GraphicalEffects
import "../navigation"
import "../controls"
import "../primitives"

Item {
    id: root

    anchors.fill: parent
    z: 1500
    enabled: shown

    property bool shown: false
    property Component pageComponent: null
    property bool resizable: true
    property string sizeKey: ""

    readonly property alias pageItem: pageLoader.item
    readonly property real viewportHeight: contentFlick.height

    signal closeRequested()

    onShownChanged: {
        resizer.markUnsettled()
        if (shown && sizeKey !== "") resizer.loadSize()
    }

    Shortcut {
        sequence: "Escape"
        enabled: root.shown
        onActivated: root.closeRequested()
    }

    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, 0.55)
        opacity: root.shown ? 1 : 0
        visible: opacity > 0.01
        Behavior on opacity { NumberAnimation { duration: Theme.dur.med } }

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.AllButtons
            onClicked: {
                forceActiveFocus()
                root.closeRequested()
            }
            onWheel: (wheel) => wheel.accepted = true
        }
    }

    Item {
        id: cardWrap
        property bool isDropdownHost: true
        x: Math.round((parent.width - width) / 2)
        y: Math.round((parent.height - height) / 2)
        width: Math.round(resizer.widthFor(parent.width * 0.95))
        height: Math.round(resizer.heightFor(parent.height * 0.95))
        opacity: root.shown ? 1 : 0
        scale: root.shown ? 1 : 0.97
        visible: opacity > 0.01

        Behavior on opacity { NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.standard } }
        Behavior on scale { NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.emphasized; easing.overshoot: Theme.ease.overshoot } }
        Behavior on width { enabled: root.shown && resizer.settled; NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.standard } }
        Behavior on height { enabled: root.shown && resizer.settled; NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.standard } }

        RectangularGlow {
            anchors.fill: card
            glowRadius: 30
            spread: 0.08
            color: Qt.rgba(0, 0, 0, 0.5)
            cornerRadius: Theme.radius.xxl + 30
            opacity: 1 - resizer.hugT
        }

        Squircle {
            id: card
            anchors.fill: parent
            radius: Theme.radius.xxl * (1 - resizer.hugT)
            fillColor: Theme.bg
        }

        MouseArea {
            anchors.fill: card
            acceptedButtons: Qt.AllButtons
            onClicked: forceActiveFocus()
            onWheel: (wheel) => wheel.accepted = true
        }

        Item {
            id: header
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 66

            Row {
                anchors.left: parent.left
                anchors.leftMargin: Theme.space.xl
                anchors.right: actions.left
                anchors.rightMargin: Theme.space.md
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.space.md

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.pageItem ? root.pageItem.modalTitle : ""
                    color: Theme.text
                    font.pixelSize: Theme.type.display.size
                    font.weight: Theme.type.display.weight
                    elide: Text.ElideRight
                }

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: Theme.space.sm
                    visible: subtitleText.text !== ""

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 4; height: 4; radius: 2
                        color: Theme.dot
                    }
                    Text {
                        id: subtitleText
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.pageItem && root.pageItem.modalSubtitle ? root.pageItem.modalSubtitle : ""
                        color: Theme.textSubtle
                        font.pixelSize: Theme.type.caption.size
                    }
                }
            }

            Row {
                id: actions
                anchors.right: parent.right
                anchors.rightMargin: Theme.space.lg
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.space.sm

                M3Button {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.pageItem ? root.pageItem.secondaryLabel : ""
                    variant: "tonal"
                    visible: text !== ""
                    enabled: root.pageItem ? root.pageItem.secondaryEnabled : false
                    onClicked: if (root.pageItem) root.pageItem.secondaryAction()
                }
                M3Button {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.pageItem ? root.pageItem.primaryLabel : ""
                    visible: text !== ""
                    enabled: root.pageItem ? root.pageItem.primaryEnabled : false
                    onClicked: if (root.pageItem) root.pageItem.primaryAction()
                }
                IconButton {
                    anchors.verticalCenter: parent.verticalCenter
                    icon: "close"
                    size: 38
                    rounded: true
                    onClicked: root.closeRequested()
                }
            }
        }

        SubNavRail {
            id: rail
            anchors.top: header.bottom
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.leftMargin: Theme.space.lg
            anchors.bottomMargin: Theme.space.lg
            width: 184
            items: root.pageItem ? root.pageItem.tabs : []
            currentIndex: root.pageItem ? root.pageItem.currentTabIndex : 0
            onItemClicked: (i) => {
                if (root.pageItem) root.pageItem.currentTabIndex = i
                contentFlick.contentY = 0
            }
        }

        Item {
            anchors.top: header.bottom
            anchors.left: rail.right
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: Theme.space.sm

            Rectangle {
                anchors.fill: parent
                color: Theme.surface
                topLeftRadius: Theme.radius.lg
                bottomRightRadius: Theme.radius.xxl * (1 - resizer.hugT)
            }

            Flickable {
                id: contentFlick
                anchors.fill: parent
                anchors.leftMargin: Theme.space.lg
                anchors.topMargin: Theme.space.md
                anchors.bottomMargin: Theme.space.md
                contentWidth: width
                contentHeight: pageLoader.item ? pageLoader.item.implicitHeight + Theme.space.lg : height
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ThinScrollBar {}

                MouseArea {
                    width: contentFlick.contentWidth
                    height: Math.max(contentFlick.contentHeight, contentFlick.height)
                    z: -1
                    onClicked: forceActiveFocus()
                }

                Loader {
                    id: pageLoader
                    readonly property int rightGap: 64
                    width: contentFlick.width - rightGap
                    active: cardWrap.visible
                    sourceComponent: root.pageComponent
                }
            }

            ScrollEdgeFade {
                anchors.fill: contentFlick
                flickable: contentFlick
                chevronOffset: -75
            }
        }

        ResizeGrips {
            id: resizer
            visible: root.resizable
            sizeKey: root.sizeKey
            minWidth: 720
            minHeight: 480
            frameMargin: Theme.space.lg * 2
        }
    }
}
