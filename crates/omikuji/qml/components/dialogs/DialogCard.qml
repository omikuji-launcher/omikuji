pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
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
    property string errorText: ""
    property bool errorAtTop: true
    property real maxWidth: 440
    property Component leftPanel: null
    property Component rightPanel: null
    property bool panelsShown: false
    property real leftPanelWidth: 300
    property real rightPanelWidth: 460

    readonly property bool _leftActive: panelsShown && leftPanel !== null
    readonly property bool _rightActive: panelsShown && rightPanel !== null
    readonly property real _panelGap: Theme.space.lg
    readonly property real _sideSpace: (width - cardWrap.width) / 2 - _panelGap - Theme.space.lg
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

    function _clampBody() {
        const max = Math.max(0, bodyFlick.contentHeight - bodyFlick.height)
        if (bodyFlick.contentY > max) {
            revealAnim.stop()
            bodyFlick.contentY = max
        }
    }

    function revealInBody(item, margin) {
        if (!item || bodyFlick.contentHeight <= bodyFlick.height) return
        const pad = margin === undefined ? Theme.space.xxl : margin
        const top = item.mapToItem(bodyLoader, 0, 0).y
        const bottom = top + item.height
        let to = bodyFlick.contentY
        if (bottom > bodyFlick.contentY + bodyFlick.height) to = bottom + pad - bodyFlick.height
        else if (top < bodyFlick.contentY) to = top - pad
        to = Math.max(0, Math.min(to, bodyFlick.contentHeight - bodyFlick.height))
        if (Math.abs(to - bodyFlick.contentY) < 1) return
        revealAnim.stop()
        revealAnim.to = to
        revealAnim.start()
    }

    NumberAnimation {
        id: revealAnim
        target: bodyFlick
        property: "contentY"
        duration: Theme.dur.med
        easing.type: Theme.ease.standard
    }

    onShownChanged: {
        resizer.markUnsettled()
        if (shown && sizeKey !== "") resizer.loadSize()
    }

    Shortcut {
        sequence: "Escape"
        enabled: root.shown && root.escEnabled
        onActivated: root.closeRequested()
    }

    component SidePanel: Item {
        property alias panelContent: panelLoader.sourceComponent
        property real panelWidth: 300

        width: panelWidth
        height: Math.min(panelLoader.implicitHeight + Theme.space.lg * 2, cardWrap.height)
        opacity: root.panelsShown && panelLoader.sourceComponent !== null && width > 0 ? 1 : 0
        visible: opacity > 0.01

        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast; easing.type: Theme.ease.standard } }

        RectangularGlow {
            anchors.fill: panelSurf
            glowRadius: 26
            spread: 0.06
            color: Qt.rgba(0, 0, 0, 0.45)
            cornerRadius: Theme.radius.xl + 26
        }

        Squircle {
            id: panelSurf
            anchors.fill: parent
            radius: Theme.radius.xl
            fillColor: Theme.surface
        }

        Flickable {
            id: panelFlick
            anchors.fill: parent
            anchors.margins: Theme.space.lg
            contentWidth: width
            contentHeight: panelLoader.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ThinScrollBar {
                parent: panelFlick.parent
                anchors.top: panelFlick.top
                anchors.bottom: panelFlick.bottom
                anchors.right: parent.right
                anchors.rightMargin: Theme.space.xs
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
        anchors.centerIn: parent
        width: resizer.widthFor(root.maxWidth)
        height: resizer.heightFor(root.fillHeight ? root.preferredHeight : naturalHeight)

        Behavior on width { enabled: root.shown && resizer.settled; NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.standard } }
        Behavior on height { enabled: root.shown && resizer.settled; NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.standard } }

        opacity: root.shown ? 1 : 0
        scale: root.shown ? 1 : 0.96
        visible: opacity > 0.01

        readonly property bool footerActive: actionsLoader.active || footerLeftLoader.active
        readonly property real footerHeight: Math.max(
            actionsLoader.active ? actionsLoader.implicitHeight : 0,
            footerLeftLoader.active ? footerLeftLoader.implicitHeight : 0)
        readonly property real naturalHeight: header.height + bodyLoader.implicitHeight
            + Theme.space.lg * 2 + (footerActive ? footerHeight + Theme.space.xl : 0)

        Behavior on opacity { NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.standard } }
        Behavior on scale { NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.emphasized; easing.overshoot: Theme.ease.overshoot } }

        RectangularGlow {
            anchors.fill: card
            glowRadius: 26
            spread: 0.06
            color: Qt.rgba(0, 0, 0, 0.45)
            cornerRadius: Theme.radius.xl + 26
            opacity: 1 - resizer.hugT
        }

        Squircle {
            id: card
            anchors.fill: parent
            radius: Theme.radius.xl * (1 - resizer.hugT)
            fillColor: Theme.surface
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
            anchors.topMargin: Theme.space.lg
            anchors.leftMargin: Theme.space.xl
            anchors.rightMargin: Theme.space.xl
            height: titleText.text !== "" ? titleText.implicitHeight + Theme.space.md : 0

            Text {
                id: titleText
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                text: root.title
                color: Theme.text
                font.pixelSize: Theme.type.title.size
                font.weight: Theme.type.title.weight
                wrapMode: Text.Wrap
                visible: text !== ""
            }
        }

        NoteChip {
            id: errorChip
            anchors.top: header.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: Theme.space.xl
            anchors.rightMargin: Theme.space.xl
            visible: root.errorAtTop && root.errorText !== ""
            height: visible ? implicitHeight : 0
            text: root.errorText
            icon: "error"
            tone: Theme.error
        }

        Flickable {
            id: bodyFlick
            anchors.top: errorChip.bottom
            anchors.topMargin: errorChip.visible ? Theme.space.md : 0
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: actionsLoader.active ? actionsLoader.top
                : (footerLeftLoader.active ? footerLeftLoader.top : parent.bottom)
            anchors.leftMargin: Theme.space.xl
            anchors.rightMargin: Theme.space.xl
            anchors.bottomMargin: cardWrap.footerActive ? (root.fillHeight ? Theme.space.md : Theme.space.xl) : Theme.space.lg
            contentWidth: width
            contentHeight: root.fillHeight ? height : bodyLoader.implicitHeight
            clip: true
            interactive: root.scrollable && !root.fillHeight && contentHeight > height
            boundsBehavior: Flickable.StopAtBounds
            onContentHeightChanged: root._clampBody()
            onHeightChanged: root._clampBody()
            ScrollBar.vertical: ThinScrollBar {
                parent: cardWrap
                anchors.top: bodyFlick.top
                anchors.bottom: bodyFlick.bottom
                anchors.right: parent.right
                anchors.rightMargin: Theme.space.xs
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
            anchors.bottomMargin: Theme.space.lg
            anchors.leftMargin: Theme.space.xl
            active: root.footerLeft !== null && cardWrap.visible
            sourceComponent: root.footerLeft
        }

        Loader {
            id: actionsLoader
            anchors.bottom: parent.bottom
            anchors.right: parent.right
            anchors.bottomMargin: Theme.space.lg
            anchors.rightMargin: Theme.space.xl
            active: root.actions !== null && cardWrap.visible
            sourceComponent: root.actions
        }

        ResizeGrips {
            id: resizer
            visible: root.resizable
            sizeKey: root.sizeKey
            minWidth: root.minWidth
            minHeight: root.minHeight
            demandH: root.fillHeight ? 0 : cardWrap.naturalHeight
        }
    }
}
