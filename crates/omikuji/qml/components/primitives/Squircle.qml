import QtQuick
import QtQuick.Shapes
import "../lib/Squircle.js" as SquircleShape

Item {
    id: root

    property color fillColor: "transparent"
    property color borderColor: "transparent"
    property real borderWidth: 0
    property real radius: 16
    property real smoothing: SquircleShape.DEFAULT_SMOOTHING

    Shape {
        anchors.fill: parent
        antialiasing: true
        preferredRendererType: Shape.CurveRenderer

        ShapePath {
            fillColor: root.fillColor
            strokeColor: root.borderWidth > 0 ? root.borderColor : "transparent"
            strokeWidth: root.borderWidth
            joinStyle: ShapePath.RoundJoin
            PathPolyline { path: SquircleShape.outline(root.width, root.height, root.radius, root.smoothing) }
        }
    }
}
