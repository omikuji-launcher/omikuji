import QtQuick
import QtQuick.Controls
import Qt5Compat.GraphicalEffects
import "../controls"
import "../primitives"

Item {
    id: root

    anchors.fill: parent
    z: 2000
    enabled: shown

    property bool shown: false
    property string title: ""
    property real maxWidth: 440
    property Component leftPanel: null
    property Component rightPanel: null
    property bool panelsShown: false
    property real leftPanelWidth: 300
    property real rightPanelWidth: 460

    readonly property bool _leftActive: panelsShown && leftPanel !== null
    readonly property bool _rightActive: panelsShown && rightPanel !== null
    readonly property real _panelGap: theme.space.lg
    readonly property real _sideSpace: (width - cardWrap.width) / 2 - _panelGap - theme.space.lg
    readonly property real _leftW: _leftActive && _sideSpace >= 140 ? Math.min(leftPanelWidth, _sideSpace) : 0
    readonly property real _rightW: _rightActive && _sideSpace >= 140 ? Math.min(rightPanelWidth, _sideSpace) : 0
    property bool scrollable: true
    property bool fillHeight: false
    property real preferredHeight: 560
    property Component body: null
    property Component actions: null
    property Component footerLeft: null

    readonly property alias bodyItem: bodyLoader.item

    signal closeRequested()

    property bool escEnabled: true

    property bool resizable: true
    property string sizeKey: ""
    property real minWidth: 320
    property real minHeight: 220

    function open() { shown = true }
    function close() { shown = false }

    onShownChanged: if (shown && sizeKey !== "") resizer.loadSize()

    Shortcut {
        sequence: "Escape"
        enabled: root.shown && root.escEnabled
        onActivated: root.closeRequested()
    }

    component SidePanel: Item {
        property alias panelContent: panelLoader.sourceComponent
        property real panelWidth: 300

        width: panelWidth
        height: Math.min(panelLoader.implicitHeight + theme.space.lg * 2, cardWrap.height)
        opacity: root.panelsShown && panelLoader.sourceComponent !== null && width > 0 ? 1 : 0
        visible: opacity > 0.01

        Behavior on opacity { NumberAnimation { duration: theme.dur.fast; easing.type: theme.ease.standard } }

        RectangularGlow {
            anchors.fill: panelSurf
            glowRadius: 26
            spread: 0.06
            color: Qt.rgba(0, 0, 0, 0.45)
            cornerRadius: theme.radius.xl + 26
        }

        Squircle {
            id: panelSurf
            anchors.fill: parent
            radius: theme.radius.xl
            fillColor: theme.surface
        }

        Flickable {
            id: panelFlick
            anchors.fill: parent
            anchors.margins: theme.space.lg
            contentWidth: width
            contentHeight: panelLoader.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ThinScrollBar {
                parent: panelFlick.parent
                anchors.top: panelFlick.top
                anchors.bottom: panelFlick.bottom
                anchors.right: parent.right
                anchors.rightMargin: theme.space.xs
            }

            Loader {
                id: panelLoader
                width: panelFlick.width
            }
        }
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
        anchors.centerIn: parent
        width: resizer.widthFor(root.maxWidth)
        height: resizer.heightFor(root.fillHeight ? root.preferredHeight : naturalHeight)
        opacity: root.shown ? 1 : 0
        scale: root.shown ? 1 : 0.96
        visible: opacity > 0.01

        readonly property bool footerActive: actionsLoader.active || footerLeftLoader.active
        readonly property real footerHeight: Math.max(
            actionsLoader.active ? actionsLoader.implicitHeight : 0,
            footerLeftLoader.active ? footerLeftLoader.implicitHeight : 0)
        readonly property real naturalHeight: header.height + bodyLoader.implicitHeight
            + theme.space.lg * 2 + (footerActive ? footerHeight + theme.space.xl : 0)

        Behavior on opacity { NumberAnimation { duration: theme.dur.med; easing.type: theme.ease.standard } }
        Behavior on scale { NumberAnimation { duration: theme.dur.med; easing.type: theme.ease.emphasized; easing.overshoot: theme.ease.overshoot } }

        RectangularGlow {
            anchors.fill: card
            glowRadius: 26
            spread: 0.06
            color: Qt.rgba(0, 0, 0, 0.45)
            cornerRadius: theme.radius.xl + 26
            opacity: 1 - resizer.hugT
        }

        Squircle {
            id: card
            anchors.fill: parent
            radius: theme.radius.xl * (1 - resizer.hugT)
            fillColor: theme.surface
        }

        MouseArea {
            anchors.fill: card
            acceptedButtons: Qt.AllButtons
            onClicked: forceActiveFocus()
            onWheel: (wheel) => wheel.accepted = true
        }

        SidePanel {
            panelContent: root.leftPanel
            panelWidth: root._leftW
            anchors.right: parent.left
            anchors.rightMargin: root._panelGap
            anchors.top: parent.top
        }

        SidePanel {
            panelContent: root.rightPanel
            panelWidth: root._rightW
            anchors.left: parent.right
            anchors.leftMargin: root._panelGap
            anchors.top: parent.top
        }

        Item {
            id: header
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.topMargin: theme.space.lg
            anchors.leftMargin: theme.space.xl
            anchors.rightMargin: theme.space.xl
            height: titleText.text !== "" ? titleText.implicitHeight + theme.space.md : 0

            Text {
                id: titleText
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                text: root.title
                color: theme.text
                font.pixelSize: theme.type.title.size
                font.weight: theme.type.title.weight
                wrapMode: Text.Wrap
                visible: text !== ""
            }
        }

        Flickable {
            id: bodyFlick
            anchors.top: header.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: actionsLoader.active ? actionsLoader.top
                : (footerLeftLoader.active ? footerLeftLoader.top : parent.bottom)
            anchors.leftMargin: theme.space.xl
            anchors.rightMargin: theme.space.xl
            anchors.bottomMargin: cardWrap.footerActive ? (root.fillHeight ? theme.space.md : theme.space.xl) : theme.space.lg
            contentWidth: width
            contentHeight: root.fillHeight ? height : bodyLoader.implicitHeight
            clip: true
            interactive: root.scrollable && !root.fillHeight && contentHeight > height
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ThinScrollBar {
                parent: cardWrap
                anchors.top: bodyFlick.top
                anchors.bottom: bodyFlick.bottom
                anchors.right: parent.right
                anchors.rightMargin: theme.space.xs
            }

            MouseArea {
                width: bodyFlick.contentWidth
                height: Math.max(bodyFlick.contentHeight, bodyFlick.height)
                z: -1
                onClicked: forceActiveFocus()
            }

            Loader {
                id: bodyLoader
                width: bodyFlick.width
                height: root.fillHeight ? bodyFlick.height : implicitHeight
                active: cardWrap.visible
                sourceComponent: root.body
            }
        }

        Loader {
            id: footerLeftLoader
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.bottomMargin: theme.space.lg
            anchors.leftMargin: theme.space.xl
            active: root.footerLeft !== null && cardWrap.visible
            sourceComponent: root.footerLeft
        }

        Loader {
            id: actionsLoader
            anchors.bottom: parent.bottom
            anchors.right: parent.right
            anchors.bottomMargin: theme.space.lg
            anchors.rightMargin: theme.space.xl
            active: root.actions !== null && cardWrap.visible
            sourceComponent: root.actions
        }

        ResizeGrips {
            id: resizer
            visible: root.resizable
            sizeKey: root.sizeKey
            minWidth: root.minWidth
            minHeight: root.minHeight
        }
    }
}
