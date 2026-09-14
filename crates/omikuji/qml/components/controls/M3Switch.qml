import QtQuick
import omikuji 1.0

Item {
    id: root

    property bool checked: false
    signal toggled(bool value)

    implicitWidth: 44
    implicitHeight: 26

    property real _progress: checked ? 1 : 0
    property real _press: mouseArea.pressed ? 1 : 0

    Behavior on _progress {
        NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.standard }
    }
    Behavior on _press {
        NumberAnimation { duration: Theme.dur.fast; easing.type: Theme.ease.standard }
    }

    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: Theme.mix(Theme.alpha(Theme.accent, 0), Theme.accent, root._progress)
        border.width: 2
        border.color: Theme.mix(Theme.alpha(Theme.text, 0.25), Theme.accent, root._progress)
    }

    Rectangle {
        readonly property int pad: 3
        readonly property int offSize: 14
        readonly property int onSize: 20
        readonly property int travelStretch: 8
        readonly property int pressStretch: 6
        readonly property real travel: (root.width - onSize) / 2 - pad
        readonly property real side: root._progress * 2 - 1
        readonly property real stretch: travelStretch * Math.sin(Math.PI * root._progress) + pressStretch * root._press

        anchors.verticalCenter: parent.verticalCenter
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.horizontalCenterOffset: side * (travel - stretch / 2)
        height: offSize + (onSize - offSize) * root._progress
        width: height + stretch
        radius: height / 2
        color: Theme.mix(Theme.alpha(Theme.text, 0.45), Theme.accentText, root._progress)
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        anchors.margins: -4
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            root.checked = !root.checked
            root.toggled(root.checked)
        }
    }
}
