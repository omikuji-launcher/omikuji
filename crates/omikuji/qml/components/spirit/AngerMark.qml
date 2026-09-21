import QtQuick
import QtQuick.Shapes
import omikuji 1.0

Item {
    id: root

    property color markColor: Theme.error
    property real tilt: 36

    implicitWidth: 28
    implicitHeight: 28

    readonly property real _cx: root.width / 2
    readonly property real _cy: root.height / 2
    readonly property real _pull: root.width * 0.56
    readonly property real _bend: root.width * 0.30
    readonly property real _sweep: 124

    function _arcX(deg) { return root._cx + root._pull * Math.cos(deg * Math.PI / 180) }
    function _arcY(deg) { return root._cy + root._pull * Math.sin(deg * Math.PI / 180) }
    function _start(deg) { return deg + 180 - root._sweep / 2 }

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        rotation: root.tilt

        ShapePath {
            strokeColor: root.markColor
            strokeWidth: root.width * 0.115
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap

            PathAngleArc {
                moveToStart: true
                centerX: root._arcX(0); centerY: root._arcY(0)
                radiusX: root._bend; radiusY: root._bend
                startAngle: root._start(0); sweepAngle: root._sweep
            }
            PathAngleArc {
                moveToStart: true
                centerX: root._arcX(90); centerY: root._arcY(90)
                radiusX: root._bend; radiusY: root._bend
                startAngle: root._start(90); sweepAngle: root._sweep
            }
            PathAngleArc {
                moveToStart: true
                centerX: root._arcX(180); centerY: root._arcY(180)
                radiusX: root._bend; radiusY: root._bend
                startAngle: root._start(180); sweepAngle: root._sweep
            }
            PathAngleArc {
                moveToStart: true
                centerX: root._arcX(270); centerY: root._arcY(270)
                radiusX: root._bend; radiusY: root._bend
                startAngle: root._start(270); sweepAngle: root._sweep
            }
        }
    }
}
