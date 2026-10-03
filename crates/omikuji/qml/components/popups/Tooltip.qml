import QtQuick
import omikuji 1.0
import QtQuick.Controls

Popup {
    id: root

    property string text: ""
    property bool tipVisible: false
    property int showDelay: 180

    readonly property int padH: 9
    readonly property int padTop: 3
    readonly property int padBottom: 4
    readonly property real maxLineWidth: 560
    readonly property real gap: 8
    readonly property bool _ownerShown: parent ? parent.visible : false

    padding: 0
    margins: 4
    closePolicy: Popup.NoAutoClose

    implicitWidth: label.implicitWidth
    implicitHeight: label.implicitHeight

    x: parent ? (parent.width - width) / 2 : 0
    y: -height - gap

    Timer {
        id: showTimer
        interval: root.showDelay
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
    on_OwnerShownChanged: {
        if (_ownerShown) return
        showTimer.stop()
        hideTimer.stop()
        Qt.callLater(() => { if (!root._ownerShown) root.visible = false })
    }

    Text {
        id: sizer
        visible: false
        text: root.text
        font.pixelSize: Theme.type.caption.size
        font.hintingPreference: Font.PreferNoHinting
    }

    background: Rectangle {
        color: Theme.tooltipBg
        radius: 7
    }

    PopupZoom { target: root }

    contentItem: Text {
        id: label
        text: root.text
        color: Theme.tooltipText
        font.pixelSize: Theme.type.caption.size
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
    exit: _ownerShown ? fadeOut : null

    Transition {
        id: fadeOut
        ParallelAnimation {
            NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 120 }
            NumberAnimation { property: "scale"; from: 1.0; to: 0.92; duration: 120 }
        }
    }
}
