pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property Flickable flickable: null
    property int orientation: Qt.Vertical
    property color surfaceColor: Theme.surface
    property real extent: Theme.space.xxl
    property bool fade: true
    property bool chevrons: true
    property real chevronOffset: 0

    readonly property bool _vertical: root.orientation === Qt.Vertical
    readonly property real _viewport: !root.flickable ? 0 : (root._vertical ? root.flickable.height : root.flickable.width)
    readonly property bool _laidOut: root._viewport > 0
    readonly property real _before: !root._laidOut ? 0
        : (root._vertical ? root.flickable.contentY - root.flickable.originY : root.flickable.contentX - root.flickable.originX)
    readonly property real _after: !root._laidOut ? 0
        : (root._vertical ? root.flickable.contentHeight : root.flickable.contentWidth) - root._viewport - root._before

    function _ramp(distance) {
        return distance <= 2 ? 0 : Math.min(1, distance / 12)
    }

    DelayedFlag {
        id: settled
        source: root._laidOut
    }

    Item {
        anchors.centerIn: parent
        width: root._vertical ? root.width : root.height
        height: root._vertical ? root.height : root.width
        rotation: root._vertical ? 0 : -90

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            height: root.extent
            visible: root.fade
            opacity: root._ramp(root._before)
            gradient: Gradient {
                GradientStop { position: 0.0; color: root.surfaceColor }
                GradientStop { position: 0.45; color: Theme.alpha(root.surfaceColor, 0.6) }
                GradientStop { position: 1.0; color: Theme.alpha(root.surfaceColor, 0) }
            }
            Behavior on opacity { enabled: settled.value; NumberAnimation { duration: Theme.dur.fast } }
        }

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: root.extent
            visible: root.fade
            opacity: root._ramp(root._after)
            gradient: Gradient {
                GradientStop { position: 0.0; color: Theme.alpha(root.surfaceColor, 0) }
                GradientStop { position: 0.55; color: Theme.alpha(root.surfaceColor, 0.6) }
                GradientStop { position: 1.0; color: root.surfaceColor }
            }
            Behavior on opacity { enabled: settled.value; NumberAnimation { duration: Theme.dur.fast } }
        }

        SvgIcon {
            anchors.top: parent.top
            anchors.topMargin: Theme.space.xs
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.horizontalCenterOffset: root.chevronOffset
            visible: root.chevrons
            name: "chevron_left"
            size: 18
            rotation: 90
            color: Theme.textMuted
            opacity: root._ramp(root._before)
            Behavior on opacity { enabled: settled.value; NumberAnimation { duration: Theme.dur.fast } }
        }

        SvgIcon {
            anchors.bottom: parent.bottom
            anchors.bottomMargin: Theme.space.xs
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.horizontalCenterOffset: root.chevronOffset
            visible: root.chevrons
            name: "chevron_left"
            size: 18
            rotation: -90
            color: Theme.textMuted
            opacity: root._ramp(root._after)
            Behavior on opacity { enabled: settled.value; NumberAnimation { duration: Theme.dur.fast } }
        }
    }
}
