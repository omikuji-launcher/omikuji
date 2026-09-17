pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Controls
import QtQuick.Window
import "../lib/Nav.js" as Nav

Popup {
    id: root

    property var items: []
    property int itemWidth: 160
    // caller-supplied floor, long items still grow past it
    property int minWidth: 0

    // CloseOnPressOutside eats thee click before it reaches the trigger, callers check lastClosedAt to skip an immediate reopen
    property double lastClosedAt: 0

    readonly property int _defaultClosePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
    property int _submenuIndex: -1
    property int _submenuSide: 1
    property Item _submenuAnchor: null
    property var _submenu: null
    property bool _subCloseExpected: false

    // tracks shift for items that declare a shiftText/shiftAction variant
    property bool _shiftDown: false

    signal itemClicked(string action)
    signal navigateBack()

    property int _keyIndex: -1

    function _selectable(i) {
        return !items[i].separator && !items[i].disabled
    }

    function _moveKey(step) {
        let i = _keyIndex !== -1 ? _keyIndex : (step > 0 ? -1 : items.length)
        for (let c = 0; c < items.length; c++) {
            i = ((i + step) % items.length + items.length) % items.length
            if (_selectable(i)) {
                _keyIndex = i
                return
            }
        }
    }

    function _activate(index, shiftHeld) {
        if (index < 0 || index >= items.length || !_selectable(index)) return
        const it = items[index]
        if (it.submenu) {
            _openSubmenuNow(index, itemRepeater.itemAt(index))
            return
        }
        itemClicked(it.shiftAction && shiftHeld
            ? it.shiftAction
            : (it.action || it.text.toLowerCase().replace(/ /g, "_")))
        close()
    }

    padding: 8
    margins: 0
    width: itemWidth + padding * 2
    focus: true

    onClosed: {
        lastClosedAt = Date.now()
        closePolicy = _defaultClosePolicy
        _closeSubmenu()
    }

    onAboutToShow: {
        _shiftDown = false
        _keyIndex = -1
    }

    Component.onDestruction: {
        if (_submenu) _submenu.destroy()
    }

    onItemsChanged: Qt.callLater(calculateWidth)

    function calculateWidth() {
        let maxWidth = 100
        for (let i = 0; i < items.length; i++) {
            if (items[i].separator) continue
            let textWidth = items[i].text.length * 7.5 + 24
            if (items[i].submenu) textWidth += 24
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

    // openers work in window coords (visual size = logical * zoom), popup x/y wants the parent frame, mixing the two caused the deep-nested corner bug
    function _moveTo(winX, winY) {
        let p = parent ? parent.mapFromItem(null, winX, winY) : Qt.point(winX, winY)
        x = p.x
        y = p.y
    }

    function openAbove(anchorItem) {
        if (!anchorItem) { open(); return }
        let win = anchorItem.Window.window
        let z = Theme.uiScale
        let w = _computedWidth() * z
        let h = _computedHeight() * z
        let a = anchorItem.mapToItem(null, anchorItem.width / 2, 0)
        let nx = a.x - w / 2
        let ny = a.y - h - 4
        if (win) {
            if (nx < 4) nx = 4
            if (nx + w > win.width - 4) nx = win.width - w - 4
            if (ny < 4) ny = 4
        }
        _moveTo(nx, ny)
        open()
    }

    function openAtCursor(winX, winY) {
        let anchorItem = parent || null
        let win = anchorItem ? anchorItem.Window.window : null
        let z = Theme.uiScale
        let w = _computedWidth() * z
        let h = _computedHeight() * z
        let nx = winX + 4
        let ny = winY + 4
        if (win) {
            if (nx + w > win.width - 4) nx = win.width - w - 4
            if (ny + h > win.height - 4) ny = win.height - h - 4
        }
        _moveTo(Math.max(4, nx), Math.max(4, ny))
        open()
    }

    function openBeside(anchorItem, side) {
        if (!anchorItem || !parent) { open(); return }
        let win = anchorItem.Window.window
        let z = Theme.uiScale
        let h = _computedHeight() * z
        let dx = side === -1 ? -_computedWidth() - padding + 4 : anchorItem.width + padding - 4
        let a = anchorItem.mapToItem(null, dx, -padding)
        let ny = a.y
        if (win) {
            let overflow = ny + h - (win.height - 4)
            if (overflow > 0) ny -= overflow
        }
        _moveTo(a.x, Math.max(4, ny))
        open()
    }

    Timer {
        id: submenuOpenTimer
        interval: 150
        onTriggered: root._openSubmenu()
    }

    function _scheduleSubmenu(index, row) {
        _submenuAnchor = row
        if (_submenuIndex === index) return
        _submenuSide = 1
        _submenuIndex = index
        submenuOpenTimer.restart()
    }

    function _openSubmenuNow(index, row) {
        submenuOpenTimer.stop()
        _submenuIndex = index
        _submenuAnchor = row
        _openSubmenu()
    }

    function _closeSubmenu() {
        submenuOpenTimer.stop()
        _submenuIndex = -1
        if (_submenu && _submenu.visible) {
            _subCloseExpected = true
            _submenu.close()
        }
    }

    function _openSubmenu() {
        if (_submenuIndex < 0 || !_submenuAnchor || !items[_submenuIndex] || !items[_submenuIndex].submenu) return
        let sub = _ensureSubmenu()
        if (!sub) return
        sub.items = items[_submenuIndex].submenu
        let w = sub._computedWidth()
        let win = _submenuAnchor.Window.window
        let absRight = _submenuAnchor.mapToItem(null, _submenuAnchor.width + padding - 4, 0).x
        _submenuSide = (win && absRight + w * Theme.uiScale > win.width - 4) ? -1 : 1
        sub.openBeside(_submenuAnchor, _submenuSide)
    }

    function _ensureSubmenu() {
        if (_submenu) return _submenu
        let comp = Qt.createComponent("ContextMenu.qml")
        if (comp.status !== Component.Ready) {
            console.warn("ContextMenu: submenu component failed:", comp.errorString())
            return null
        }
        _submenu = comp.createObject(parent)
        _submenu.itemClicked.connect(function(action) {
            root.itemClicked(action)
            root.close()
        })
        _submenu.navigateBack.connect(root._closeSubmenu)
        return _submenu
    }

    Connections {
        target: root._submenu
        function onOpened() { root.closePolicy = Popup.NoAutoClose }
        function onClosed() {
            root.closePolicy = root._defaultClosePolicy
            root._submenuIndex = -1
            if (!root._subCloseExpected) root.close()
            root._subCloseExpected = false
        }
    }

    background: PopupSurface {}

    PopupZoom { target: root }

    PopupFocusReturn { popup: root }

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
        focus: true

        Keys.onPressed: (event) => {
            event.accepted = true
            switch (event.key) {
            case Qt.Key_Shift: root._shiftDown = true; break
            case Qt.Key_Down: root._moveKey(1); break
            case Qt.Key_Up: root._moveKey(-1); break
            case Qt.Key_Right:
                if (root._keyIndex >= 0 && root.items[root._keyIndex].submenu) root._activate(root._keyIndex, false)
                break
            case Qt.Key_Left: root.navigateBack(); break
            default:
                event.accepted = Nav.isActivate(event)
                if (event.accepted) root._activate(root._keyIndex, (event.modifiers & Qt.ShiftModifier) !== 0)
            }
        }
        Keys.onReleased: (event) => { if (event.key === Qt.Key_Shift) root._shiftDown = false }

        Repeater {
            id: itemRepeater
            model: root.items

            delegate: Loader {
                id: itemLoader
                required property var modelData
                required property int index

                sourceComponent: modelData.separator ? separatorComponent : menuItemComponent

                Component {
                    id: menuItemComponent
                    Rectangle {
                        width: root.itemWidth
                        height: 32
                        radius: Theme.radius.sm
                        // danger items tint red, normal items use ~2x the old 0.08 alpha so light-mode hovers are actually visible
                        color: root._keyIndex === itemLoader.index && !itemLoader.modelData.disabled
                            ? (itemLoader.modelData.danger
                                ? Theme.alpha(Theme.error, 0.18)
                                : itemLoader.modelData.accent
                                    ? Theme.alpha(Theme.accent, 0.18)
                                    : Theme.alpha(Theme.text, 0.14))
                            : "transparent"

                        Behavior on color {
                            ColorAnimation { duration: 80 }
                        }

                        Text {
                            id: itemText
                            anchors.left: parent.left
                            anchors.leftMargin: 12
                            anchors.verticalCenter: parent.verticalCenter
                            text: (itemLoader.modelData.shiftText && root._shiftDown && root._keyIndex === itemLoader.index)
                                ? itemLoader.modelData.shiftText
                                : itemLoader.modelData.text
                            color: itemLoader.modelData.disabled
                                ? Theme.textSubtle
                                : itemLoader.modelData.danger
                                ? Theme.error
                                : itemLoader.modelData.accent
                                    ? Theme.accent
                                    : Theme.text
                            font.pixelSize: Theme.type.label.size
                        }

                        SvgIcon {
                            visible: !!itemLoader.modelData.submenu
                            anchors.right: parent.right
                            anchors.rightMargin: 8
                            anchors.verticalCenter: parent.verticalCenter
                            name: "chevron_left"
                            size: 16
                            color: Theme.textMuted
                            rotation: root._submenuIndex === itemLoader.index && root._submenuSide === -1 ? 0 : 180

                            Behavior on rotation {
                                NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
                            }
                        }

                        MouseArea {
                            id: hoverArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: itemLoader.modelData.disabled ? Qt.ArrowCursor : Qt.PointingHandCursor
                            onEntered: {
                                root._keyIndex = itemLoader.index
                                if (itemLoader.modelData.submenu) root._scheduleSubmenu(itemLoader.index, parent)
                                else root._closeSubmenu()
                            }
                            onExited: if (root._keyIndex === itemLoader.index) root._keyIndex = -1
                            onPositionChanged: (mouse) => root._shiftDown = (mouse.modifiers & Qt.ShiftModifier) !== 0
                            onClicked: (mouse) => root._activate(itemLoader.index, (mouse.modifiers & Qt.ShiftModifier) !== 0)
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
                            color: Theme.divider
                        }
                    }
                }
            }
        }
    }
}
