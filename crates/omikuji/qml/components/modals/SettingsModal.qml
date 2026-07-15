import QtQuick
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

    signal closeRequested()

    onShownChanged: if (shown && sizeKey !== "") resizer.loadSize()

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
        Behavior on opacity { NumberAnimation { duration: theme.dur.med } }

        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.AllButtons
            onClicked: root.closeRequested()
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

        Behavior on opacity { NumberAnimation { duration: theme.dur.med; easing.type: theme.ease.standard } }
        Behavior on scale { NumberAnimation { duration: theme.dur.med; easing.type: theme.ease.emphasized; easing.overshoot: theme.ease.overshoot } }

        RectangularGlow {
            anchors.fill: card
            glowRadius: 30
            spread: 0.08
            color: Qt.rgba(0, 0, 0, 0.5)
            cornerRadius: theme.radius.xxl + 30
        }

        Squircle {
            id: card
            anchors.fill: parent
            radius: theme.radius.xxl
            fillColor: theme.bg
        }

        MouseArea {
            anchors.fill: card
            acceptedButtons: Qt.AllButtons
            onClicked: {}
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
                anchors.leftMargin: theme.space.xl
                anchors.right: actions.left
                anchors.rightMargin: theme.space.md
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space.md

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.pageItem ? root.pageItem.modalTitle : ""
                    color: theme.text
                    font.pixelSize: theme.type.display.size
                    font.weight: theme.type.display.weight
                    elide: Text.ElideRight
                }

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: theme.space.sm
                    visible: subtitleText.text !== ""

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 4; height: 4; radius: 2
                        color: theme.dot
                    }
                    Text {
                        id: subtitleText
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.pageItem && root.pageItem.modalSubtitle ? root.pageItem.modalSubtitle : ""
                        color: theme.textSubtle
                        font.pixelSize: theme.type.caption.size
                    }
                }
            }

            Row {
                id: actions
                anchors.right: parent.right
                anchors.rightMargin: theme.space.lg
                anchors.verticalCenter: parent.verticalCenter
                spacing: theme.space.sm

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
            anchors.leftMargin: theme.space.lg
            anchors.bottomMargin: theme.space.lg
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
            anchors.leftMargin: theme.space.sm

            Rectangle {
                anchors.fill: parent
                color: theme.surface
                topLeftRadius: theme.radius.lg
                bottomRightRadius: theme.radius.xxl
            }

            Flickable {
                id: contentFlick
                anchors.fill: parent
                anchors.leftMargin: theme.space.lg
                anchors.topMargin: theme.space.md
                anchors.bottomMargin: theme.space.md
                contentWidth: width
                contentHeight: pageLoader.item ? pageLoader.item.implicitHeight + theme.space.lg : height
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ThinScrollBar {}

                Loader {
                    id: pageLoader
                    readonly property int rightGap: 64
                    width: contentFlick.width - rightGap
                    active: cardWrap.visible
                    sourceComponent: root.pageComponent
                }
            }
        }

        ResizeGrips {
            id: resizer
            visible: root.resizable
            sizeKey: root.sizeKey
            minWidth: 720
            minHeight: 480
            frameMargin: theme.space.lg * 2
        }
    }
}
