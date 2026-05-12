import QtQuick
import QtQuick.Controls
import QtQuick.Window

Popup {
    id: root

    property var items: []
    property int itemWidth: 160
    // caller-supplied floor, long items still grow past it
    property int minWidth: 0

    // CloseOnPressOutside eats thee click before it reaches the trigger, callers check lastClosedAt to skip an immediate reopen
    property double lastClosedAt: 0

    signal itemClicked(string action)

    padding: 8
    margins: 0
    width: itemWidth + padding * 2

    onClosed: lastClosedAt = Date.now()

    onItemsChanged: Qt.callLater(calculateWidth)

    function calculateWidth() {
        let maxWidth = 100
        for (let i = 0; i < items.length; i++) {
            if (items[i].separator) continue
            let textWidth = items[i].text.length * 7.5 + 24
            if (textWidth > maxWidth) maxWidth = textWidth
        }
        let floor = minWidth > 0 ? minWidth : 160
        itemWidth = Math.min(280, Math.max(floor, maxWidth))
    }

    onMinWidthChanged: calculateWidth()

    // sized before open() to avoid the top-left flash from implicit size resolution
    function _computedHeight() {
        let h = padding * 2
        for (let i = 0; i < items.length; i++) {
            h += items[i].separator ? 17 : 32
        }
        return h
    }
    function _computedWidth() {
        calculateWidth()
        return itemWidth + padding * 2
    }

    function openAbove(anchorItem) {
        if (!anchorItem) { open(); return }
        let win = anchorItem.Window.window
        let p = anchorItem.mapToItem(null, anchorItem.width / 2, 0)
        let w = _computedWidth()
        let h = _computedHeight()
        let nx = p.x - w / 2
        let ny = p.y - h - 4
        if (win) {
            if (nx < 4) nx = 4
            if (nx + w > win.width - 4) nx = win.width - w - 4
            if (ny < 4) ny = 4
        }
        x = nx
        y = ny
        open()
    }

    // Popup x/y is in the parent coord frame but overflow checks need the window frame, mixing the two caused the deep-nested corner bug
    function openBelow(anchorItem) {
        if (!anchorItem) { open(); return }
        if (!parent) { open(); return }
        let win = anchorItem.Window.window
        let w = _computedWidth()
        let h = _computedHeight()

        let p = anchorItem.mapToItem(parent, 0, anchorItem.height + 4)
        let nx = p.x
        let ny = p.y

        let abs = anchorItem.mapToItem(null, 0, anchorItem.height + 4)
        if (win) {
            if (abs.y + h > win.height - 4) {
                openAbove(anchorItem)
                return
            }
            let overflowRight = (abs.x + w) - (win.width - 4)
            if (overflowRight > 0) nx -= overflowRight
        }

        x = nx
        y = ny
        open()
    }

    function openAtCursor(winX, winY) {
        let anchorItem = parent || null
        let win = anchorItem ? anchorItem.Window.window : null
        let w = _computedWidth()
        let h = _computedHeight()
        let nx = winX + 4
        let ny = winY + 4
        if (win) {
            if (nx + w > win.width - 4) nx = win.width - w - 4
            if (ny + h > win.height - 4) ny = win.height - h - 4
        }
        x = Math.max(4, nx)
        y = Math.max(4, ny)
        open()
    }

    // dark themes lighten the popup, light themes darken it becuase pure-white surfaces need to drop in lightness to stand out
    background: Rectangle {
        color: theme.active.window.hslLightness > 0.5
            ? Qt.darker(theme.popup, 1.06)
            : Qt.lighter(theme.popup, 1.3)
        radius: 12
        border.width: 1
        border.color: Qt.rgba(theme.active.windowText.r, theme.active.windowText.g, theme.active.windowText.b, 0.08)
    }

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 120; easing.type: Easing.OutCubic }
        NumberAnimation { property: "scale"; from: 0.95; to: 1; duration: 120; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 80 }
        NumberAnimation { property: "scale"; from: 1; to: 0.98; duration: 80 }
    }

    contentItem: Column {
        id: menuColumn
        spacing: 0
        width: root.itemWidth

        Repeater {
            id: itemRepeater
            model: root.items

            delegate: Loader {
                required property var modelData
                required property int index

                sourceComponent: modelData.separator ? separatorComponent : menuItemComponent

                Component {
                    id: menuItemComponent
                    Rectangle {
                        width: root.itemWidth
                        height: 32
                        radius: 8
                        // danger items tint red, normal items use ~2x the old 0.08 alpha so light-mode hovers are actually visible
                        color: hoverArea.containsMouse
                            ? (modelData.danger
                                ? Qt.rgba(theme.error.r, theme.error.g, theme.error.b, 0.18)
                                : modelData.accent
                                    ? Qt.rgba(theme.accent.r, theme.accent.g, theme.accent.b, 0.18)
                                    : Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.14))
                            : "transparent"

                        Behavior on color {
                            ColorAnimation { duration: 80 }
                        }

                        Text {
                            id: itemText
                            anchors.left: parent.left
                            anchors.leftMargin: 12
                            anchors.verticalCenter: parent.verticalCenter
                            text: modelData.text
                            color: modelData.danger
                                ? theme.error
                                : modelData.accent
                                    ? theme.accent
                                    : theme.text
                            font.pixelSize: 13
                        }

                        MouseArea {
                            id: hoverArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                root.itemClicked(modelData.action || modelData.text.toLowerCase().replace(/ /g, "_"))
                                root.close()
                            }
                        }
                    }
                }

                Component {
                    id: separatorComponent
                    Rectangle {
                        width: root.itemWidth
                        height: 17
                        color: "transparent"

                        Rectangle {
                            anchors.centerIn: parent
                            width: parent.width - 24
                            height: 1
                            color: theme.divider
                        }
                    }
                }
            }
        }
    }
}
