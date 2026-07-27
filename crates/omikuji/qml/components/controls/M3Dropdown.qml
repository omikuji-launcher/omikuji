import QtQuick
import QtQuick.Controls
import "../popups"
import "../primitives"

Item {
    id: root

    property var options: []
    property int currentIndex: 0
    property var currentValue: {
        if (options.length === 0) return ""
        var opt = options[currentIndex]
        return (opt && !opt.header) ? opt.value : ""
    }
    property string label: ""
    property real fieldHeight: 44

    signal selected(var value)

    readonly property bool popupOpen: popup.visible

    property int _savedIndex: 0

    function openPopup() {
        _savedIndex = currentIndex
        popup.open()
    }
    function closePopupCancel() {
        currentIndex = _savedIndex
        popup.close()
    }
    function closePopupCommit() {
        selected(currentValue)
        popup.close()
    }
    function highlightPrev() {
        if (options.length === 0) return
        var i = currentIndex
        for (var c = 0; c < options.length; c++) {
            i = (i - 1 + options.length) % options.length
            if (!options[i].header) { currentIndex = i; return }
        }
    }
    function highlightNext() {
        if (options.length === 0) return
        var i = currentIndex
        for (var c = 0; c < options.length; c++) {
            i = (i + 1) % options.length
            if (!options[i].header) { currentIndex = i; return }
        }
    }

    implicitWidth: 200
    readonly property real boxCenterY: button.y + button.height / 2

    implicitHeight: label ? labelText.height + 4 + button.height : button.height

    Text {
        id: labelText
        text: root.label
        color: popup.visible ? theme.accent : theme.textMuted
        font.pixelSize: theme.type.label.size
        font.weight: Font.Medium
        visible: root.label !== ""

        Behavior on color { ColorAnimation { duration: 100 } }
    }

    FieldSurface {
        id: button
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: root.fieldHeight
        focused: popup.visible

        Text {
            anchors.left: parent.left
            anchors.leftMargin: 12
            anchors.right: chevronIcon.left
            anchors.rightMargin: 6
            anchors.verticalCenter: parent.verticalCenter
            elide: Text.ElideRight
            text: {
                if (root.options.length === 0) return ""
                var opt = root.options[root.currentIndex]
                return (opt && !opt.header) ? opt.label : ""
            }
            color: theme.text
            font.pixelSize: theme.type.body.size
        }

        SvgIcon {
            id: chevronIcon
            anchors.right: parent.right
            anchors.rightMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            name: "chevron_left"
            size: 20
            color: theme.textMuted
            rotation: popup.visible ? -90 : 0

            Behavior on rotation {
                NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
            }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: popup.visible ? popup.close() : popup.open()
        }
    }

    // auto-close when off-screen, the popup has no other way to know its anchor disappeared
    onVisibleChanged: {
        if (!visible && popup.visible) popup.close()
    }

    readonly property var popupHost: {
        var p = root.parent
        while (p) {
            if (p.isDropdownHost === true) return p
            p = p.parent
        }
        if (root.ApplicationWindow && root.ApplicationWindow.contentItem)
            return root.ApplicationWindow.contentItem
        return root.Window ? root.Window.contentItem : root
    }

    MouseArea {
        id: outsideCatcher
        parent: root.popupHost
        anchors.fill: parent
        visible: popup.visible
        z: popup.z - 1
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onPressed: popup.close()
        onWheel: (wheel) => {
            popup.close()
            wheel.accepted = false
        }
    }

    PopupSurface {
        id: popup
        parent: root.popupHost
        visible: false
        x: 0
        y: 0
        width: 0
        // clamp against the window not the popup parent, so a small dialog card doestn shrink the dropdown to nothing
        height: {
            if (!visible) return 0
            var wanted = col.height + 16
            var win = root.Window
            if (!win || !parent) return Math.round(wanted)
            var topInWin = parent.mapToItem(win.contentItem, x, y).y
            var maxAvail = win.height - topInWin - 12
            return Math.round(Math.min(wanted, Math.max(80, maxAvail)))
        }
        z: 50
        radius: theme.radius.sm

        function open() {
            if (!popup.parent) return
            syncPosition()
            visible = true
        }
        function close() { visible = false }
        function syncPosition() {
            if (!popup.parent) return
            var pos = button.mapToItem(popup.parent, 0, button.height + 4)
            popup.x = Math.round(pos.x)
            popup.y = Math.round(pos.y)
            popup.width = Math.round(button.width)
        }

        // flickable scroll is a visual transform with no property-change signal, so a cheap poll keeps the popup glued. i suppose. Lets hope! 
        Timer {
            running: popup.visible
            interval: 16
            repeat: true
            onTriggered: popup.syncPosition()
        }

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.NoButton
            onWheel: (wheel) => wheel.accepted = true
        }

        SvgIcon {
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 4
            anchors.horizontalCenter: parent.horizontalCenter
            name: "chevron_left"
            size: 18
            rotation: -90
            color: theme.textMuted
            z: 1
            opacity: {
                if (!popup.visible) return 0
                var remaining = popupFlick.contentHeight - (popupFlick.contentY + popupFlick.height)
                if (remaining <= 2) return 0
                return Math.min(1.0, remaining / 12)
            }
            Behavior on opacity { NumberAnimation { duration: 120 } }
        }

        Flickable {
            id: popupFlick
            anchors.fill: parent
            anchors.margins: 8
            contentWidth: width
            contentHeight: col.height
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            interactive: contentHeight > height

            Column {
                id: col
                width: popupFlick.width

                Repeater {
                    model: root.options

                    Rectangle {
                        id: optionRow
                        required property int index
                        required property var modelData
                        readonly property bool isHeader: modelData && modelData.header === true
                        width: col.width
                        height: isHeader ? (index === 0 ? 22 : 28) : 40
                        radius: theme.radius.xs
                        color: !isHeader && optionMouse.containsMouse
                            ? (index === root.currentIndex
                                ? theme.alpha(theme.accent, 0.18)
                                : theme.alpha(theme.text, 0.14))
                            : "transparent"

                        // group caption, non-interactive
                        Text {
                            visible: optionRow.isHeader
                            anchors.left: parent.left
                            anchors.leftMargin: 12
                            anchors.bottom: parent.bottom
                            anchors.bottomMargin: 4
                            text: modelData.label
                            color: theme.textMuted
                            font.pixelSize: theme.type.body.size
                            font.weight: Font.Medium
                        }

                        Item {
                            id: labelClip
                            visible: !optionRow.isHeader
                            anchors.left: parent.left
                            anchors.leftMargin: 8
                            anchors.right: parent.right
                            anchors.rightMargin: 8
                            anchors.verticalCenter: parent.verticalCenter
                            height: optionText.implicitHeight
                            clip: true

                            readonly property real overflow: Math.max(0, optionText.implicitWidth - width)
                            property real pan: 0
                            property bool manualPan: false
                            readonly property color bg: !optionRow.isHeader && optionMouse.containsMouse
                                ? (index === root.currentIndex
                                    ? theme.mix(popup.color, theme.accent, 0.18)
                                    : theme.mix(popup.color, theme.text, 0.14))
                                : popup.color

                            function panBy(delta) {
                                manualPan = true
                                pan = Math.max(0, Math.min(overflow, pan + delta))
                            }
                            function reset() {
                                manualPan = false
                                pan = 0
                            }

                            Text {
                                id: optionText
                                x: -Math.round(labelClip.pan)
                                text: modelData.label
                                color: index === root.currentIndex ? theme.accent : theme.text
                                font.pixelSize: theme.type.body.size
                                font.weight: index === root.currentIndex ? Font.Medium : Font.Normal
                            }

                            SequentialAnimation {
                                running: optionMouse.containsMouse && labelClip.overflow > 0 && !labelClip.manualPan
                                loops: Animation.Infinite
                                PauseAnimation { duration: 350 }
                                NumberAnimation {
                                    target: labelClip
                                    property: "pan"
                                    to: labelClip.overflow
                                    duration: Math.max(300, labelClip.overflow * 16)
                                }
                                PauseAnimation { duration: 650 }
                                NumberAnimation {
                                    target: labelClip
                                    property: "pan"
                                    to: 0
                                    duration: Math.max(300, labelClip.overflow * 16)
                                }
                                PauseAnimation { duration: 450 }
                            }

                            Rectangle {
                                anchors.left: parent.left
                                anchors.top: parent.top
                                anchors.bottom: parent.bottom
                                width: 16
                                opacity: labelClip.pan > 1 ? 1 : 0
                                gradient: Gradient {
                                    orientation: Gradient.Horizontal
                                    GradientStop { position: 0.0; color: labelClip.bg }
                                    GradientStop { position: 1.0; color: theme.alpha(labelClip.bg, 0) }
                                }
                                Behavior on opacity { NumberAnimation { duration: 120 } }
                            }

                            Rectangle {
                                anchors.right: parent.right
                                anchors.top: parent.top
                                anchors.bottom: parent.bottom
                                width: 16
                                opacity: labelClip.overflow > 0 && labelClip.pan < labelClip.overflow - 1 ? 1 : 0
                                gradient: Gradient {
                                    orientation: Gradient.Horizontal
                                    GradientStop { position: 0.0; color: theme.alpha(labelClip.bg, 0) }
                                    GradientStop { position: 1.0; color: labelClip.bg }
                                }
                                Behavior on opacity { NumberAnimation { duration: 120 } }
                            }
                        }

                        MouseArea {
                            id: optionMouse
                            anchors.fill: parent
                            enabled: !optionRow.isHeader
                            hoverEnabled: !optionRow.isHeader
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (optionRow.isHeader) return
                                root.currentIndex = index
                                root.selected(root.options[index].value)
                                popup.close()
                            }
                            onContainsMouseChanged: if (!containsMouse) labelClip.reset()
                            onWheel: (wheel) => {
                                if (optionRow.isHeader || labelClip.overflow <= 0) {
                                    wheel.accepted = false
                                    return
                                }
                                var d = wheel.angleDelta.x
                                if (d === 0 && ((wheel.modifiers & Qt.ShiftModifier) || !popupFlick.interactive))
                                    d = wheel.angleDelta.y
                                if (d === 0) {
                                    wheel.accepted = false
                                    return
                                }
                                labelClip.panBy(-d * 0.5)
                            }
                        }
                    }
                }
            }
        }

    }
}
