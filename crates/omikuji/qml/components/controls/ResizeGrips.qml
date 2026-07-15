import QtQuick

Item {
    id: grips

    property string sizeKey: ""
    property real minWidth: 320
    property real minHeight: 220
    property real frameMargin: theme.space.xxl * 2
    // user size lives as a fraction of the available frame so it scales with the window
    property real fracW: 0
    property real fracH: 0

    property Item frame: parent ? parent.parent : null

    anchors.fill: parent
    z: 10

    function widthFor(fallback) {
        let avail = frame ? frame.width - frameMargin : fallback
        if (fracW > 0) return Math.min(Math.max(minWidth, Math.round(fracW * avail)), avail)
        return Math.min(fallback, avail)
    }

    function heightFor(fallback) {
        let avail = frame ? frame.height - frameMargin : fallback
        if (fracH > 0) return Math.min(Math.max(minHeight, Math.round(fracH * avail)), avail)
        return Math.min(fallback, avail)
    }

    function loadSize() {
        let e = _sizesMap()[sizeKey]
        if (e) { fracW = Math.min(1, e[0]); fracH = Math.min(1, e[1]) }
    }

    function resetSize() {
        fracW = 0
        fracH = 0
        if (sizeKey === "") return
        let m = _sizesMap()
        delete m[sizeKey]
        uiSettings.applyDialogSizesJson(JSON.stringify(m))
    }

    function _sizesMap() {
        try { return JSON.parse(uiSettings.dialogSizesJson()) } catch (err) { return {} }
    }

    function _save() {
        if (sizeKey === "" || (fracW <= 0 && fracH <= 0)) return
        let m = _sizesMap()
        m[sizeKey] = [fracW, fracH]
        uiSettings.applyDialogSizesJson(JSON.stringify(m))
    }

    component Grip: MouseArea {
        property int edgeX: 0
        property int edgeY: 0
        property real _startW: 0
        property real _startH: 0
        property point _startPos: Qt.point(0, 0)
        preventStealing: true
        cursorShape: edgeX !== 0 && edgeY !== 0
            ? (edgeX === edgeY ? Qt.SizeFDiagCursor : Qt.SizeBDiagCursor)
            : (edgeX !== 0 ? Qt.SizeHorCursor : Qt.SizeVerCursor)
        onPressed: (mouse) => {
            _startW = grips.parent.width
            _startH = grips.parent.height
            _startPos = mapToItem(grips.frame, mouse.x, mouse.y)
        }
        onPositionChanged: (mouse) => {
            if (!grips.frame) return
            let p = mapToItem(grips.frame, mouse.x, mouse.y)
            if (edgeX !== 0) {
                let availW = grips.frame.width - grips.frameMargin
                let px = Math.min(Math.max(grips.minWidth, _startW + (p.x - _startPos.x) * 2 * edgeX), availW)
                grips.fracW = px / availW
            }
            if (edgeY !== 0) {
                let availH = grips.frame.height - grips.frameMargin
                let px = Math.min(Math.max(grips.minHeight, _startH + (p.y - _startPos.y) * 2 * edgeY), availH)
                grips.fracH = px / availH
            }
        }
        onReleased: grips._save()
        onDoubleClicked: grips.resetSize()
    }

    Grip { edgeX: -1; width: 8; anchors { left: parent.left; top: parent.top; bottom: parent.bottom; leftMargin: -3; topMargin: 10; bottomMargin: 10 } }
    Grip { edgeX: 1; width: 8; anchors { right: parent.right; top: parent.top; bottom: parent.bottom; rightMargin: -3; topMargin: 10; bottomMargin: 10 } }
    Grip { edgeY: -1; height: 8; anchors { top: parent.top; left: parent.left; right: parent.right; topMargin: -3; leftMargin: 10; rightMargin: 10 } }
    Grip { edgeY: 1; height: 8; anchors { bottom: parent.bottom; left: parent.left; right: parent.right; bottomMargin: -3; leftMargin: 10; rightMargin: 10 } }
    Grip { edgeX: -1; edgeY: -1; width: 13; height: 13; anchors { left: parent.left; top: parent.top; leftMargin: -3; topMargin: -3 } }
    Grip { edgeX: 1; edgeY: -1; width: 13; height: 13; anchors { right: parent.right; top: parent.top; rightMargin: -3; topMargin: -3 } }
    Grip { edgeX: -1; edgeY: 1; width: 13; height: 13; anchors { left: parent.left; bottom: parent.bottom; leftMargin: -3; bottomMargin: -3 } }
    Grip { edgeX: 1; edgeY: 1; width: 13; height: 13; anchors { right: parent.right; bottom: parent.bottom; rightMargin: -3; bottomMargin: -3 } }
}
