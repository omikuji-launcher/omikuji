import QtQuick
import QtQuick.Shapes
import omikuji 1.0

Item {
    id: root

    property color lineColor: Theme.text
    property color fillColor: Theme.surface
    property real strokeWidth: 3

    implicitWidth: 44
    implicitHeight: 52

    readonly property real lidHeight: root.height * 0.216
    readonly property real shoulderY: root.height * 0.16
    readonly property real topInset: root.width * 0.10
    readonly property real footRadius: root.width * 0.12

    function edgeTop(fraction) {
        return root.topInset + (root.width - root.topInset * 2) * fraction
    }

    Shape {
        anchors.fill: parent
        antialiasing: true
        preferredRendererType: Shape.CurveRenderer

        ShapePath {
            fillColor: root.fillColor
            strokeColor: root.lineColor
            strokeWidth: root.strokeWidth
            joinStyle: ShapePath.RoundJoin

            startX: root.topInset
            startY: root.shoulderY

            PathLine { x: root.width - root.topInset; y: root.shoulderY }
            PathLine { x: root.width; y: root.height - root.footRadius }
            PathQuad {
                x: root.width - root.footRadius; y: root.height
                controlX: root.width; controlY: root.height
            }
            PathLine { x: root.footRadius; y: root.height }
            PathQuad {
                x: 0; y: root.height - root.footRadius
                controlX: 0; controlY: root.height
            }
            PathLine { x: root.topInset; y: root.shoulderY }
        }

        ShapePath {
            strokeColor: Theme.alpha(root.lineColor, 0.45)
            strokeWidth: root.strokeWidth * 0.6
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap

            startX: root.edgeTop(0.28)
            startY: root.shoulderY

            PathLine { x: root.width * 0.28; y: root.height - root.footRadius * 0.5 }
            PathMove { x: root.edgeTop(0.72); y: root.shoulderY }
            PathLine { x: root.width * 0.72; y: root.height - root.footRadius * 0.5 }
        }
    }

    Rectangle {
        id: lid

        width: parent.width
        height: root.lidHeight
        radius: height * 0.35
        color: root.lineColor
    }

    Rectangle {
        anchors.horizontalCenter: lid.horizontalCenter
        y: lid.height * 0.3
        width: root.width * 0.18
        height: lid.height * 0.4
        radius: width / 2
        color: root.fillColor
    }
}
