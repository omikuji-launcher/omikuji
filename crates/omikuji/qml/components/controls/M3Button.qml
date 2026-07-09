import QtQuick
import "../primitives"

Item {
    id: root

    property string text: ""
    property string icon: ""
    property string variant: "filled"
    property bool danger: false
    property bool success: false
    property bool small: false
    property real radius: small ? theme.radius.md : theme.radius.lg

    signal clicked()

    readonly property color _accent: danger ? theme.error : (success ? theme.success : theme.accent)
    readonly property bool _filled: variant === "filled"
    readonly property bool _tonal: variant === "tonal"
    readonly property bool _outlined: variant === "outlined"

    readonly property color _fg: _filled
        ? (danger || success ? (_accent.hslLightness > 0.6 ? "#000000" : "#ffffff") : theme.accentOn)
        : (_outlined ? theme.text : _accent)
    readonly property color _bg: _filled ? _accent
        : (_tonal ? theme.alpha(_accent, 0.15) : "transparent")
    readonly property color _stateOn: _filled ? _fg : _accent

    implicitHeight: small ? 28 : 36
    implicitWidth: Math.max(small ? 0 : 72, content.implicitWidth + (small ? theme.space.md : theme.space.lg) * 2)
    opacity: enabled ? 1 : 0.45

    Squircle {
        id: bg
        anchors.fill: parent
        radius: root.radius
        fillColor: root._bg
        borderColor: root._outlined ? theme.outlineStrong : "transparent"
        borderWidth: root._outlined ? 1 : 0
        scale: area.pressed ? 0.97 : 1.0

        Behavior on scale { NumberAnimation { duration: theme.dur.xfast; easing.type: theme.ease.standard } }
        Behavior on fillColor { ColorAnimation { duration: theme.dur.fast } }

        Squircle {
            anchors.fill: parent
            radius: root.radius
            fillColor: theme.alpha(root._stateOn, area.pressed ? 0.14 : (area.containsMouse ? 0.08 : 0.0))
            Behavior on fillColor { ColorAnimation { duration: theme.dur.fast } }
        }

        Row {
            id: content
            anchors.centerIn: parent
            spacing: theme.space.sm

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
                font.pixelSize: root.small ? theme.type.label.size : theme.type.subtitle.size
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
