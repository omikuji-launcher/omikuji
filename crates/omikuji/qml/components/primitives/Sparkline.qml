import QtQuick
import omikuji 1.0

Item {
    id: root

    property var samples: []
    property var samples2: []
    property color line: Theme.accent
    property color line2: Theme.alpha(Theme.text, 0.45)

    onSamplesChanged: cv.requestPaint()
    onSamples2Changed: cv.requestPaint()
    onLineChanged: cv.requestPaint()
    onLine2Changed: cv.requestPaint()

    Canvas {
        id: cv
        width: root.width * 2
        height: root.height * 2
        scale: 0.5
        transformOrigin: Item.TopLeft

        function drawSeries(ctx, pts, mx, col, fillAlpha) {
            var w = width
            var h = height
            var step = w / (pts.length - 1)
            ctx.beginPath()
            ctx.moveTo(0, h - (pts[0] / mx) * h)
            for (var i = 1; i < pts.length; i++)
                ctx.lineTo(i * step, h - (pts[i] / mx) * h)
            ctx.strokeStyle = String(col)
            ctx.lineWidth = 4
            ctx.lineJoin = "round"
            ctx.stroke()
            if (fillAlpha > 0) {
                ctx.lineTo(w, h)
                ctx.lineTo(0, h)
                ctx.closePath()
                ctx.fillStyle = Qt.rgba(col.r, col.g, col.b, fillAlpha)
                ctx.fill()
            }
        }

        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()
            if (!root.samples || root.samples.length < 2) return
            var all = root.samples.concat(root.samples2 || [])
            var mx = Math.max(Math.max.apply(null, all), 1) * 1.2
            if (root.samples2 && root.samples2.length > 1) drawSeries(ctx, root.samples2, mx, root.line2, 0)
            drawSeries(ctx, root.samples, mx, root.line, 0.10)
        }
    }
}
