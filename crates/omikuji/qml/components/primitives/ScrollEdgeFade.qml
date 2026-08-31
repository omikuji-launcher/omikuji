pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property Flickable flickable: null
    property color surfaceColor: Theme.surface
    property real extent: Theme.space.xxl
    property bool fade: true
    property bool chevrons: true
    property real chevronOffset: 0

    readonly property real _above: root.flickable ? root.flickable.contentY - root.flickable.originY : 0
    readonly property real _below: root.flickable ? root.flickable.contentHeight - root.flickable.height - root._above : 0

    function _ramp(distance) {
        return distance <= 2 ? 0 : Math.min(1, distance / 12)
    }

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: root.extent
        visible: root.fade
        opacity: root._ramp(root._above)
        gradient: Gradient {
            GradientStop { position: 0.0; color: root.surfaceColor }
            GradientStop { position: 0.45; color: Theme.alpha(root.surfaceColor, 0.6) }
            GradientStop { position: 1.0; color: Theme.alpha(root.surfaceColor, 0) }
        }
        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
    }

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: root.extent
        visible: root.fade
        opacity: root._ramp(root._below)
        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.alpha(root.surfaceColor, 0) }
            GradientStop { position: 0.55; color: Theme.alpha(root.surfaceColor, 0.6) }
            GradientStop { position: 1.0; color: root.surfaceColor }
        }
        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
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
        opacity: root._ramp(root._above)
        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
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
        opacity: root._ramp(root._below)
        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
    }
}
