import QtQuick
import "../lib/Nav.js" as Nav

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

    function revealItem(item, margin) {
        const to = Nav.revealY(root.flick, item, {
            margin: margin,
            alignTall: true,
            fromY: root._anim.running ? root._anim.to : undefined
        })
        if (to === null) return
        root._anim.stop()
        root._anim.to = to
        root._anim.start()
    }
}
