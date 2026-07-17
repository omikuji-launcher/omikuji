import QtQuick
import QtQuick.Controls

Popup {
    id: root

    property string text: ""
    property bool tipVisible: false

    readonly property int padH: 9
    readonly property int padTop: 3
    readonly property int padBottom: 4
    readonly property real maxLineWidth: 560
    readonly property real gap: 8

    padding: 0
    margins: 4
    closePolicy: Popup.NoAutoClose

    implicitWidth: label.implicitWidth
    implicitHeight: label.implicitHeight

    x: parent ? (parent.width - width) / 2 : 0
    y: -height - gap

    Timer {
        id: showTimer
        interval: 180
        onTriggered: root.visible = true
    }
    Timer {
        id: hideTimer
        interval: 150
        onTriggered: root.visible = false
    }
    onTipVisibleChanged: {
        if (tipVisible) {
            hideTimer.stop()
            if (!visible) showTimer.restart()
        } else {
            showTimer.stop()
            if (visible) hideTimer.restart()
        }
    }

    Text {
        id: sizer
        visible: false
        text: root.text
        font.pixelSize: theme.type.caption.size
        font.hintingPreference: Font.PreferNoHinting
    }

    background: Rectangle {
        color: theme.tooltipBg
        radius: 7
    }

    PopupZoom { target: root }

    contentItem: Text {
        id: label
        text: root.text
        color: theme.tooltipText
        font.pixelSize: theme.type.caption.size
        font.hintingPreference: Font.PreferNoHinting
        leftPadding: root.padH
        rightPadding: root.padH
        topPadding: root.padTop
        bottomPadding: root.padBottom
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignLeft
        width: Math.min(sizer.implicitWidth, root.maxLineWidth)
            + leftPadding + rightPadding
    }

    enter: Transition {
        ParallelAnimation {
            NumberAnimation {
                property: "opacity"
                from: 0; to: 1
                duration: 120
                easing.type: Easing.BezierSpline
                easing.bezierCurve: [0.34, 0.80, 0.34, 1.00, 1, 1]
            }
            NumberAnimation {
                property: "scale"
                from: 0.88; to: 1.0
                duration: 120
                easing.type: Easing.BezierSpline
                easing.bezierCurve: [0.34, 0.80, 0.34, 1.00, 1, 1]
            }
        }
    }
    exit: Transition {
        ParallelAnimation {
            NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 120 }
            NumberAnimation { property: "scale"; from: 1.0; to: 0.92; duration: 120 }
        }
    }
}
