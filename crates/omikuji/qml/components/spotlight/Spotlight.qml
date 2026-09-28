pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Shapes
import omikuji 1.0
import "../lib/Squircle.js" as SquircleShape
import "../lib/Nav.js" as Nav

Item {
    id: root

    anchors.fill: parent
    visible: shape.opacity > 0.01

    property var keys: []
    property bool shown: false
    property bool passThrough: false
    property bool highlighted: false
    property real padding: Theme.space.sm
    property real radius: Theme.radius.lg

    readonly property var holes: _slots.filter(r => r !== null)
    property var targetRects: []

    property real _tint: highlighted ? 0.16 : 0
    Behavior on _tint { NumberAnimation { duration: Theme.dur.fast; easing.type: Theme.ease.standard } }

    property var _slots: []
    property var _from: []
    property real _t: 1

    onKeysChanged: {
        _from = _slots
        _t = 0
    }

    onShownChanged: {
        if (!shown) return
        _slots = []
        targetRects = []
    }

    function _goal(key) {
        const marker = SpotlightTargets.find(key)
        if (!marker) return null
        const item = marker.target
        const pad = marker.padding >= 0 ? marker.padding : padding
        const shown = Nav.shownRect(item, null)
        const r = shown ? root.mapFromItem(null, shown.x, shown.y, shown.width, shown.height)
            : item.mapToItem(root, 0, 0, item.width, item.height)
        const x0 = Math.max(padding, r.x - pad)
        const y0 = Math.max(padding, r.y - pad)
        const x1 = Math.min(width - padding, r.x + r.width + pad)
        const y1 = Math.min(height - padding, r.y + r.height + pad)
        return Qt.rect(x0, y0, Math.max(0, x1 - x0), Math.max(0, y1 - y0))
    }

    function _lerpRect(a, b, t) {
        return Qt.rect(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t,
            a.width + (b.width - a.width) * t, a.height + (b.height - a.height) * t)
    }

    function _sameRect(a, b) {
        return a === b || (a !== null && b !== null
            && a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height)
    }

    function _easeInOut(t) {
        return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2
    }

    function _step(dt) {
        const goals = keys.map(_goal)
        if (goals.every(g => g !== null)) {
            _t = Math.min(1, _t + dt * 1000 / Theme.dur.slow)
            if (goals.length !== targetRects.length || goals.some((g, i) => !_sameRect(g, targetRects[i]))) targetRects = goals
        }
        const e = _easeInOut(_t)
        const next = goals.map((g, i) => {
            const from = i < _from.length ? _from[i] : null
            if (g === null) return i < _slots.length ? _slots[i] : from
            if (_t >= 1) return g
            return _lerpRect(from || Qt.rect(g.x + g.width / 2, g.y + g.height / 2, 0, 0), g, e)
        })
        if (next.length !== _slots.length || next.some((r, i) => !_sameRect(r, _slots[i]))) _slots = next
    }

    function _inHole(p) {
        return holes.some(h => p.x >= h.x && p.x <= h.x + h.width && p.y >= h.y && p.y <= h.y + h.height)
    }

    function _cornerRadius(rect) {
        return Math.min(radius, Math.min(rect.width, rect.height) * 0.15)
    }

    readonly property var _holePaths: holes.map(h =>
        SquircleShape.outline(h.width, h.height, _cornerRadius(h), SquircleShape.DEFAULT_SMOOTHING)
            .map(p => Qt.point(p.x + h.x, p.y + h.y)))

    readonly property var _paths: [[Qt.point(0, 0), Qt.point(width, 0), Qt.point(width, height), Qt.point(0, height), Qt.point(0, 0)]]
        .concat(_holePaths)

    FrameAnimation {
        running: root.shown
        onTriggered: root._step(frameTime)
    }

    Shape {
        id: shape
        anchors.fill: parent
        antialiasing: true
        preferredRendererType: Shape.CurveRenderer
        opacity: root.shown ? 1 : 0

        Behavior on opacity { NumberAnimation { duration: Theme.dur.med } }

        ShapePath {
            fillColor: Theme.scrim
            fillRule: ShapePath.OddEvenFill
            strokeWidth: -1
            PathMultiline { paths: root._paths }
        }

        ShapePath {
            fillColor: Theme.alpha(Theme.accent, root._tint)
            strokeWidth: -1
            PathMultiline { paths: root._holePaths }
        }

        ShapePath {
            fillColor: "transparent"
            strokeColor: Theme.focusAccent
            strokeWidth: 2
            joinStyle: ShapePath.RoundJoin
            PathMultiline { paths: root._holePaths }
        }
    }

    MouseArea {
        anchors.fill: parent
        enabled: root.shown
        hoverEnabled: true
        acceptedButtons: Qt.AllButtons
        containmentMask: QtObject {
            function contains(point: point): bool {
                return !root.passThrough || !root._inHole(point)
            }
        }
        onWheel: (wheel) => wheel.accepted = true
    }
}
