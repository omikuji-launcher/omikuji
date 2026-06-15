import QtQuick
import QtQuick.Controls
import "."

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
    implicitHeight: label ? labelText.height + 4 + button.height : button.height

    Text {
        id: labelText
        text: root.label
        color: popup.visible ? theme.accent : theme.textMuted
        font.pixelSize: 13
        font.weight: Font.Medium
        visible: root.label !== ""

        Behavior on color { ColorAnimation { duration: 100 } }
    }

    Rectangle {
        id: button
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: 44
        radius: 8
        color: "transparent"
        border.width: popup.visible ? 2 : 1
        border.color: popup.visible
            ? theme.accent
            : Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.15)

        Behavior on border.width { NumberAnimation { duration: 100 } }
        Behavior on border.color { ColorAnimation { duration: 100 } }

        Text {
            anchors.left: parent.left
            anchors.leftMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            text: {
                if (root.options.length === 0) return ""
                var opt = root.options[root.currentIndex]
                return (opt && !opt.header) ? opt.label : ""
            }
            color: theme.text
            font.pixelSize: 14
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
    }

    Rectangle {
        id: popup
        parent: root.popupHost
        visible: false
        x: 0
        y: 0
        width: button.width
        // clamp against the window not the popup parent, so a small dialog card doestn shrink the dropdown to nothing
        height: {
            if (!visible) return 0
            var wanted = col.height + 8
            var win = root.Window
            if (!win || !parent) return wanted
            var topInWin = parent.mapToItem(win.contentItem, x, y).y
            var maxAvail = win.height - topInWin - 12
            return Math.min(wanted, Math.max(80, maxAvail))
        }
        z: 50
        radius: 8
        color: theme.bg
        border.width: 1
        border.color: theme.surfaceBorder

        function open() {
            if (!popup.parent) return
            syncPosition()
            popup.width = button.width
            visible = true
        }
        function close() { visible = false }
        function syncPosition() {
            if (!popup.parent) return
            var pos = button.mapToItem(popup.parent, 0, button.height + 4)
            popup.x = pos.x
            popup.y = pos.y
            popup.width = button.width
        }

        // flickable scroll is a visual transform with no property-change signal, so a cheap poll keeps the popup glued. i suppose. Lets hope! 
        Timer {
            running: popup.visible
            interval: 16
            repeat: true
            onTriggered: popup.syncPosition()
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
            anchors.margins: 4
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
                        radius: 6
                        color: !isHeader && optionMouse.containsMouse ? theme.surfaceHover : "transparent"

                        // group caption, non-interactive
                        Text {
                            visible: optionRow.isHeader
                            anchors.left: parent.left
                            anchors.leftMargin: 12
                            anchors.bottom: parent.bottom
                            anchors.bottomMargin: 4
                            text: modelData.label
                            color: theme.textMuted
                            font.pixelSize: 14
                            font.weight: Font.Medium
                        }

                        Text {
                            visible: !optionRow.isHeader
                            anchors.left: parent.left
                            anchors.leftMargin: 8
                            anchors.verticalCenter: parent.verticalCenter
                            text: modelData.label
                            color: index === root.currentIndex ? theme.accent : theme.text
                            font.pixelSize: 14
                            font.weight: index === root.currentIndex ? Font.Medium : Font.Normal
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
                        }
                    }
                }
            }
        }

    }
}
