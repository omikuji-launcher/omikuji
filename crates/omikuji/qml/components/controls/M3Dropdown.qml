pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
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
    property string labelSuffix: ""
    property color labelSuffixColor: Theme.textMuted
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

    Row {
        id: labelText
        spacing: 5
        visible: root.label !== ""

        Text {
            text: root.label
            color: popup.visible ? Theme.accent : Theme.textMuted
            font.pixelSize: Theme.type.label.size
            font.weight: Font.Medium

            Behavior on color { ColorAnimation { duration: 100 } }
        }

        Text {
            visible: root.labelSuffix !== ""
            text: "· " + root.labelSuffix
            color: root.labelSuffixColor
            font.pixelSize: Theme.type.label.size
        }
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
            color: {
                var opt = root.options[root.currentIndex]
                return (opt && opt.tint) ? opt.tint : Theme.text
            }
            font.pixelSize: Theme.type.body.size
        }

        SvgIcon {
            id: chevronIcon
            anchors.right: parent.right
            anchors.rightMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            name: "chevron_left"
            size: 20
            color: Theme.textMuted
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
        radius: Theme.radius.sm

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

        ScrollEdgeFade {
            anchors.fill: parent
            z: 1
            flickable: popupFlick
            fade: false
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
                        readonly property color tint: (modelData && modelData.tint)
                            ? modelData.tint
                            : (index === root.currentIndex ? Theme.accent : Theme.text)
                        width: col.width
                        height: isHeader ? (index === 0 ? 22 : 28) : 40
                        radius: Theme.radius.xs
                        color: !isHeader && optionMouse.containsMouse
                            ? Theme.alpha(tint, index === root.currentIndex ? 0.18 : 0.14)
                            : "transparent"

                        // group caption, non-interactive
                        Text {
                            visible: optionRow.isHeader
                            anchors.left: parent.left
                            anchors.leftMargin: 12
                            anchors.bottom: parent.bottom
                            anchors.bottomMargin: 4
                            text: optionRow.modelData.label
                            color: Theme.textMuted
                            font.pixelSize: Theme.type.body.size
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
                                ? Theme.mix(popup.color, optionRow.tint,
                                            optionRow.index === root.currentIndex ? 0.18 : 0.14)
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
                                text: optionRow.modelData.label
                                color: optionRow.tint
                                font.pixelSize: Theme.type.body.size
                                font.weight: optionRow.index === root.currentIndex ? Font.Medium : Font.Normal
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
                                    GradientStop { position: 1.0; color: Theme.alpha(labelClip.bg, 0) }
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
                                    GradientStop { position: 0.0; color: Theme.alpha(labelClip.bg, 0) }
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
                                root.currentIndex = optionRow.index
                                root.selected(root.options[optionRow.index].value)
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
