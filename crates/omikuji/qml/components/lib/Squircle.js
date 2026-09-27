.pragma library

var DEFAULT_SMOOTHING = 0.75

function outline(w, h, r, s) {
    if (w <= 0 || h <= 0)
        return [Qt.point(0, 0)]
    var minSide = Math.min(w, h)
    var rr = Math.max(0, Math.min(r, minSide / 2))
    if (rr <= 0.5)
        return [Qt.point(0, 0), Qt.point(w, 0), Qt.point(w, h), Qt.point(0, h), Qt.point(0, 0)]

    var sm = Math.max(0, Math.min(1, s))
    var g = 0.55 + sm * 0.45
    var reach = Math.min(rr * (1 + sm * 0.6), minSide / 2 * 0.82)
    var m = reach * (1 - g)

    var pts = []
    var steps = 16
    function cubic(p0, p1, p2, p3) {
        for (var i = 0; i <= steps; i++) {
            var t = i / steps
            var u = 1 - t
            pts.push(Qt.point(
                u*u*u*p0.x + 3*u*u*t*p1.x + 3*u*t*t*p2.x + t*t*t*p3.x,
                u*u*u*p0.y + 3*u*u*t*p1.y + 3*u*t*t*p2.y + t*t*t*p3.y))
        }
    }
    cubic(Qt.point(0, reach),     Qt.point(0, m),     Qt.point(m, 0),     Qt.point(reach, 0))
    cubic(Qt.point(w - reach, 0), Qt.point(w - m, 0), Qt.point(w, m),     Qt.point(w, reach))
    cubic(Qt.point(w, h - reach), Qt.point(w, h - m), Qt.point(w - m, h), Qt.point(w - reach, h))
    cubic(Qt.point(reach, h),     Qt.point(m, h),     Qt.point(0, h - m), Qt.point(0, h - reach))
    pts.push(pts[0])
    return pts
}
