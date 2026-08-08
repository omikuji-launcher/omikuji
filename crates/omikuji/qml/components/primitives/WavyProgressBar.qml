pragma ComponentBehavior: Bound

import QtQuick

Item {
    id: root

    property real value: 0.0

    // smooth incoming progress so the handle slides rather than jumping between backend ticks
    Behavior on value {
        NumberAnimation { duration: 300; easing.type: Easing.OutCubic }
    }

    property bool wavy: true
    property bool animate: true

    property color fillColor: "#685496"
    property color trackColor: "#F1D3F9"
    property color handleColor: fillColor

    property real trackWidth: 4
    property real handleWidth: 3
    property real handleHeight: 24
    property real handleMargins: 4
    property real trackDotSize: 3

    // fixed pixel wavelength so wider rows get more waves not stretched ones, the rice uses fixed cycle count becuase their slider is always narrow
    property real waveWavelength: 32
    property real waveAmplitudeMultiplier: 0.5
    property real phaseDivisor: 400.0

    implicitHeight: handleHeight
    implicitWidth: 160

    readonly property real visualPosition: Math.max(0, Math.min(1, value))
    readonly property real effectiveWidth: Math.max(0, width - 2 * handleMargins)
    readonly property real handleX: handleMargins + visualPosition * effectiveWidth - handleWidth / 2
    readonly property real halfHandleGap: handleWidth / 2

    // canvas rasterized once per size/color change, motion comes from x-translation, sliding one wavelength equals advancing phase by 2pi at near-zero per-frame cost
    Item {
        id: fillClip
        anchors.verticalCenter: parent.verticalCenter
        x: root.handleMargins
        width: Math.max(0, root.handleX - root.handleMargins - root.halfHandleGap)
        height: root.trackWidth * 6
        clip: true
        visible: root.wavy

        Canvas {
            id: wave
            // one extra wavelength on the right so the x-slide never exposes empty space before wrapping
            width: root.width + root.waveWavelength
            height: parent.height

            readonly property int cycleMs: Math.max(16, Math.round(2 * Math.PI * root.phaseDivisor))

            NumberAnimation on x {
                from: 0
                to: -root.waveWavelength
                duration: wave.cycleMs
                loops: Animation.Infinite
                running: root.animate && root.wavy
            }

            onPaint: {
                const ctx = getContext("2d")
                ctx.clearRect(0, 0, width, height)
                if (width <= 0) return

                const lineW = root.trackWidth
                const amp = lineW * root.waveAmplitudeMultiplier
                const cy = height / 2

                ctx.strokeStyle = root.fillColor
                ctx.lineWidth = lineW
                ctx.lineCap = "round"
                ctx.beginPath()
                const twoPiOverL = 2 * Math.PI / root.waveWavelength
                // 2px steps halve JS iterations and are visually indistuingishbale on a 4px wave
                for (let x = lineW / 2; x <= width - lineW / 2; x += 2) {
                    const y = cy + amp * Math.sin(twoPiOverL * x)
                    if (x === lineW / 2) ctx.moveTo(x, y)
                    else ctx.lineTo(x, y)
                }
                ctx.stroke()
            }

            Connections {
                target: root
                function onFillColorChanged() { wave.requestPaint() }
                function onWavyChanged() { wave.requestPaint() }
            }
            onWidthChanged: requestPaint()
        }
    }

    Loader {
        anchors.verticalCenter: parent.verticalCenter
        x: root.handleMargins
        width: Math.max(0, root.handleX - root.handleMargins - root.halfHandleGap)
        height: root.trackWidth
        active: !root.wavy
        sourceComponent: Rectangle {
            color: root.fillColor
            radius: height / 2
            Behavior on width { NumberAnimation { duration: 200; easing.type: Easing.OutCubic } }
        }
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        x: root.handleX + root.handleWidth + root.halfHandleGap
        width: Math.max(0, root.width - root.handleMargins - (root.handleX + root.handleWidth + root.halfHandleGap))
        height: root.trackWidth
        radius: height / 2
        color: root.trackColor
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        x: root.handleX
        width: root.handleWidth
        height: root.handleHeight
        radius: width / 2
        color: root.handleColor
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        x: root.handleMargins + root.effectiveWidth - root.trackDotSize / 2
        width: root.trackDotSize
        height: root.trackDotSize
        radius: width / 2
        color: root.visualPosition >= 1 ? root.fillColor : Qt.rgba(root.fillColor.r, root.fillColor.g, root.fillColor.b, 0.9)
        visible: (root.effectiveWidth - root.visualPosition * root.effectiveWidth) > root.handleWidth
    }
}
