pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property real value: 0.0
    Behavior on value {
        NumberAnimation { duration: 300; easing.type: Easing.OutCubic }
    }

    property string style: Theme.progressStyle
    property bool paused: false
    property bool animate: true

    property color fillColor: Theme.accent
    property color trackColor: Theme.alpha(Theme.text, 0.18)
    property real trackWidth: 4
    property real handleWidth: 3
    property real handleHeight: 24
    property real handleMargins: 4

    readonly property real position: Math.max(0, Math.min(1, value))
    readonly property color shownFill: paused ? Theme.alpha(Theme.text, 0.3) : fillColor

    readonly property var styles: ({
        wavy: { track: linearTrack, wave: true },
        flat: { track: linearTrack, wave: false },
        chunky: { track: pillTrack },
        segmented: { track: blockTrack },
        striped: { track: stripeTrack },
        rope: { track: ropeTrack }
    })
    readonly property var spec: styles[style] || styles.rope

    implicitWidth: 160
    implicitHeight: handleHeight

    Loader {
        anchors.fill: parent
        sourceComponent: root.spec.track
    }

    Component {
        id: linearTrack

        LinearTrack {
            value: root.position
            wavy: root.spec.wave === true && !root.paused
            animate: root.animate && !root.paused
            fillColor: root.shownFill
            trackColor: root.trackColor
            trackWidth: root.trackWidth
            handleWidth: root.handleWidth
            handleHeight: root.handleHeight
            handleMargins: root.handleMargins
        }
    }

    Component {
        id: pillTrack

        PillTrack {
            value: root.position
            fillColor: root.shownFill
            trackColor: root.trackColor
        }
    }

    Component {
        id: blockTrack

        BlockTrack {
            value: root.position
            fillColor: root.shownFill
            trackColor: root.trackColor
        }
    }

    Component {
        id: stripeTrack

        StripeTrack {
            value: root.position
            animate: root.animate && !root.paused
            fillColor: root.shownFill
            trackColor: root.trackColor
        }
    }

    Component {
        id: ropeTrack

        RopeTrack {
            value: root.position
            paused: root.paused
            animate: root.animate
            spiritColor: root.fillColor
            trackColor: root.trackColor
            trackWidth: root.trackWidth
        }
    }
}
