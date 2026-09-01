pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../primitives"
import "../controls"


// built on ListView so add/remove gives smooth stack shifts, not instant pops
Item {
    id: root

    property int dismissMs: 4500
    property int maxVisible: 5
    readonly property int toastWidth: 340
    readonly property int toastSpacing: 10

    property int nextId: 0

    function show(level, title, message) {
        return insert(level, title, message, false, -1)
    }

    function showProgress(level, title, message) {
        return insert(level, title, message, true, 0)
    }

    function update(id, message, progress) {
        let i = indexOfToast(id)
        if (i < 0) return
        toastModel.setProperty(i, "message", String(message || ""))
        if (progress !== undefined) toastModel.setProperty(i, "progress", progress)
    }

    function dismiss(id) {
        let i = indexOfToast(id)
        if (i >= 0) toastModel.remove(i)
    }

    function insert(level, title, message, sticky, progress) {
        while (toastModel.count >= root.maxVisible) {
            toastModel.remove(toastModel.count - 1)
        }
        let id = root.nextId++
        toastModel.insert(0, {
            toastId: id,
            level: String(level || "info"),
            title: String(title || ""),
            message: String(message || ""),
            sticky: sticky === true,
            progress: progress
        })
        return id
    }

    function indexOfToast(id) {
        for (let i = 0; i < toastModel.count; i++) {
            if (toastModel.get(i).toastId === id) return i
        }
        return -1
    }

    ListModel { id: toastModel }

    ListView {
        id: stack
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 18
        width: root.toastWidth
        height: Math.max(40, parent.height - 36)
        spacing: root.toastSpacing
        interactive: false
        clip: false
        model: toastModel

        add: Transition {
            ParallelAnimation {
                NumberAnimation {
                    properties: "opacity"
                    from: 0; to: 1
                    duration: 220
                    easing.type: Easing.OutCubic
                }
                NumberAnimation {
                    properties: "x"
                    from: stack.width + 24
                    to: 0
                    duration: 260
                    easing.type: Easing.OutBack
                    easing.overshoot: 1.1
                }
            }
        }

        remove: Transition {
            ParallelAnimation {
                NumberAnimation {
                    properties: "opacity"
                    to: 0
                    duration: 180
                    easing.type: Easing.InCubic
                }
                NumberAnimation {
                    properties: "x"
                    to: stack.width + 24
                    duration: 200
                    easing.type: Easing.InCubic
                }
            }
        }

        displaced: Transition {
            NumberAnimation {
                properties: "y"
                duration: 280
                easing.type: Easing.OutCubic
            }
        }

        delegate: PopupSurface {
            id: toast
            required property int index
            required property int toastId
            required property string level
            required property string title
            required property string message
            required property bool sticky
            required property real progress

            readonly property color levelColor: {
                switch (level) {
                    case "success": return Theme.success
                    case "warning": return Theme.warning
                    case "error":   return Theme.error
                    default:        return Theme.accent
                }
            }
            readonly property string levelIcon: {
                switch (level) {
                    case "success": return "check_circle_fill"
                    case "warning": return "warning_fill"
                    case "error":   return "error_fill"
                    default:        return "info_fill"
                }
            }

            width: root.toastWidth
            height: toastCol.implicitHeight + 24
            radius: Theme.radius.lg

            SvgIcon {
                anchors.left: parent.left
                anchors.leftMargin: 14
                anchors.top: parent.top
                anchors.topMargin: 12
                name: toast.levelIcon
                size: 20
                color: toast.levelColor
            }

            Column {
                id: toastCol
                anchors.left: parent.left
                anchors.leftMargin: 44
                anchors.right: closeBtn.left
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                spacing: 3

                Text {
                    width: parent.width
                    text: toast.title
                    color: Theme.text
                    font.pixelSize: Theme.type.label.size
                    font.weight: Font.DemiBold
                    elide: Text.ElideRight
                    visible: text.length > 0
                }

                Text {
                    width: parent.width
                    text: toast.message
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.WordWrap
                    visible: text.length > 0
                }

                WavyProgressBar {
                    width: parent.width
                    visible: toast.progress >= 0
                    value: toast.progress
                    wavy: false
                    trackWidth: 3
                    handleWidth: 3
                    handleHeight: 12
                    handleMargins: 2
                    fillColor: toast.levelColor
                    handleColor: fillColor
                    trackColor: Theme.alpha(Theme.text, 0.18)
                }
            }

            IconButton {
                id: closeBtn
                icon: "close"
                size: 28
                rounded: true
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: 8
                onClicked: toastModel.remove(toast.index)
            }

            // paused while hovered so users can read longer messages
            Timer {
                id: dismissTimer
                interval: root.dismissMs
                running: !hoverArea.containsMouse && !toast.sticky
                repeat: false
                onTriggered: toastModel.remove(toast.index)
            }

            MouseArea {
                id: hoverArea
                anchors.fill: parent
                anchors.rightMargin: closeBtn.width + 8
                hoverEnabled: true
                acceptedButtons: Qt.NoButton
                propagateComposedEvents: true
            }
        }
    }
}
