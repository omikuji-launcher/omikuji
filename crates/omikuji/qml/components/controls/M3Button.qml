import QtQuick
import omikuji 1.0
import "../primitives"

Item {
    id: root

    property string text: ""
    property string icon: ""
    property string variant: "filled"
    property bool danger: false
    property bool success: false
    property bool small: false
    property real radius: small ? Theme.radius.md : Theme.radius.lg

    signal clicked()

    readonly property color _accent: danger ? Theme.error : (success ? Theme.success : Theme.accent)
    readonly property bool _filled: variant === "filled"
    readonly property bool _tonal: variant === "tonal"
    readonly property bool _outlined: variant === "outlined"

    readonly property color _fg: _filled
        ? (danger || success ? (_accent.hslLightness > 0.6 ? "#000000" : "#ffffff") : Theme.accentOn)
        : (_outlined ? Theme.text : _accent)
    readonly property color _bg: _filled ? _accent
        : (_tonal ? Theme.alpha(_accent, 0.15) : "transparent")
    readonly property color _stateOn: _filled ? _fg : _accent

    implicitHeight: small ? 28 : 36
    implicitWidth: Math.max(small ? 0 : 72, content.implicitWidth + (small ? Theme.space.md : Theme.space.lg) * 2)
    opacity: enabled ? 1 : 0.45

    Squircle {
        id: bg
        anchors.fill: parent
        radius: root.radius
        fillColor: root._bg
        borderColor: root._outlined ? Theme.outlineStrong : "transparent"
        borderWidth: root._outlined ? 1 : 0
        scale: area.pressed ? 0.97 : 1.0

        Behavior on scale { NumberAnimation { duration: Theme.dur.xfast; easing.type: Theme.ease.standard } }
        Behavior on fillColor { ColorAnimation { duration: Theme.dur.fast } }

        Squircle {
            anchors.fill: parent
            radius: root.radius
            fillColor: Theme.alpha(root._stateOn, area.pressed ? 0.14 : (area.containsMouse ? 0.08 : 0.0))
            Behavior on fillColor { ColorAnimation { duration: Theme.dur.fast } }
        }

        Row {
            id: content
            anchors.centerIn: parent
            spacing: Theme.space.sm

            SvgIcon {
                anchors.verticalCenter: parent.verticalCenter
                name: root.icon
                size: root.small ? 14 : 18
                color: root._fg
                visible: root.icon !== ""
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.text
                color: root._fg
                font.pixelSize: root.small ? Theme.type.label.size : Theme.type.subtitle.size
                font.weight: root.small ? Font.Medium : Font.DemiBold
                visible: root.text !== ""
            }
        }
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
