import QtQuick
import QtQuick.Controls
import omikuji 1.0
import "../lib/Nav.js" as Nav

Item {
    id: halo

    readonly property real gap: 3 * Theme.uiScale
    readonly property Item owner: {
        if (!InputMode.keyboard || !Window.window) return null
        const item = Nav.navigableOwner(Window.window.activeFocusItem, null)
        return item && item.visible && item.enabled && item.opacity > 0 && typeof item.navRingRadius === "number" ? item : null
    }
    property rect ownerRect: Qt.rect(0, 0, 0, 0)
    property real ownerRadius: 0
    property color ringColor: Theme.focusAccent

    function _sync() {
        if (!owner) return
        const inner = typeof owner.navRectItem === "function" ? owner.navRectItem() : null
        const src = inner || owner
        ownerRect = src.mapToItem(halo, 0, 0, src.width, src.height)
    }

    parent: Overlay.overlay
    anchors.fill: parent
    z: 1000000
    onOwnerChanged: {
        if (!owner) return
        ownerRadius = owner.navRingRadius
        ringColor = owner.danger === true ? Theme.error : Theme.focusAccent
        _sync()
    }

    FrameAnimation {
        running: halo.owner !== null
        onTriggered: halo._sync()
    }

    Squircle {
        x: halo.ownerRect.x - halo.gap
        y: halo.ownerRect.y - halo.gap
        width: halo.ownerRect.width + halo.gap * 2
        height: halo.ownerRect.height + halo.gap * 2
        radius: halo.ownerRadius * Theme.uiScale + halo.gap
        fillColor: "transparent"
        borderColor: halo.ringColor
        borderWidth: 2 * Theme.uiScale
        visible: halo.owner !== null
    }
}
