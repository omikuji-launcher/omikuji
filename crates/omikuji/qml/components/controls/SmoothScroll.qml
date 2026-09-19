import QtQuick

QtObject {
    id: root

    property Flickable flick: null
    property int duration: 160

    readonly property NumberAnimation _anim: NumberAnimation {
        target: root.flick
        property: "contentY"
        duration: root.duration
        easing.type: Easing.OutCubic
    }

    function revealRect(y, height, margin) {
        if (!root.flick) return
        const pad = margin === undefined ? 8 : margin
        const minY = root.flick.originY
        const maxY = Math.max(minY, minY + root.flick.contentHeight - root.flick.height)
        const from = root._anim.running ? root._anim.to : root.flick.contentY
        let target = from
        if (y < from) target = Math.max(minY, y - pad)
        else if (y + height > from + root.flick.height) target = Math.min(maxY, y + height - root.flick.height + pad)
        if (target === from) return
        root._anim.stop()
        root._anim.to = target
        root._anim.start()
    }

    function revealItem(item, margin) {
        if (!item || !root.flick) return
        const r = item.mapToItem(root.flick.contentItem, 0, 0, item.width, item.height)
        root.revealRect(r.y, r.height, margin)
    }
}
