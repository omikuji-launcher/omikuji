pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import omikuji 1.0

Item {
    id: root

    property bool shown: false
    property var anchorRect: null
    property bool centered: false
    property var content: null

    signal backClicked()
    signal nextClicked()
    signal closeClicked()
    signal contentShown()

    readonly property Item nextButton: nextBtn

    property var _view: ({ stepText: "", title: "", body: "", note: "", hint: "", canBack: false, canNext: false, isLast: false, art: null })
    on_ViewChanged: contentShown()

    function _sameView(a, b) {
        return Object.keys(a).every(k => a[k] === b[k])
    }

    onContentChanged: {
        if (!content || _sameView(content, _view)) return
        if (opacity < 1) {
            _view = content
            return
        }
        swap.restart()
    }

    SequentialAnimation {
        id: swap
        NumberAnimation { target: layout; property: "opacity"; to: 0; duration: Theme.dur.xfast; easing.type: Easing.InCubic }
        ScriptAction { script: if (root.content) root._view = root.content }
        NumberAnimation { target: layout; property: "opacity"; to: 1; duration: Theme.dur.fast; easing.type: Theme.ease.standard }
    }

    readonly property real _gap: Theme.space.lg
    readonly property real _margin: Theme.space.lg

    function _place(r, w, h, W, H) {
        const m = _margin
        if (centered) return Qt.point((W - w) / 2, (H - h) / 2)
        const bottomCenter = Qt.point((W - w) / 2, H - h - Theme.space.xxl)
        if (!r) return bottomCenter
        const cx = Math.max(m, Math.min(W - w - m, r.x + r.width / 2 - w / 2))
        const cy = Math.max(m, Math.min(H - h - m, r.y + r.height / 2 - h / 2))
        if (r.y + r.height + _gap + h + m <= H) return Qt.point(cx, r.y + r.height + _gap)
        if (r.y - _gap - h >= m) return Qt.point(cx, r.y - _gap - h)
        if (r.x + r.width + _gap + w + m <= W) return Qt.point(r.x + r.width + _gap, cy)
        if (r.x - _gap - w >= m) return Qt.point(r.x - _gap - w, cy)
        return bottomCenter
    }

    readonly property point _pos: parent ? _place(anchorRect, width, height, parent.width, parent.height) : Qt.point(0, 0)

    function _follow() {
        if (!shown) return
        x = _pos.x
        y = _pos.y
    }

    on_PosChanged: _follow()
    onShownChanged: _follow()

    width: 380
    height: layout.implicitHeight + Theme.space.lg * 2
    opacity: shown ? 1 : 0
    scale: shown ? 1 : 0.96
    visible: opacity > 0.01

    Behavior on opacity { NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.standard } }
    Behavior on scale { NumberAnimation { duration: Theme.dur.med; easing.type: Theme.ease.emphasized; easing.overshoot: Theme.ease.overshoot } }
    Behavior on x { enabled: root.visible; NumberAnimation { duration: Theme.dur.slow; easing.type: Easing.InOutCubic } }
    Behavior on y { enabled: root.visible; NumberAnimation { duration: Theme.dur.slow; easing.type: Easing.InOutCubic } }

    RectangularShadow {
        anchors.fill: surface
        blur: 26
        radius: surface.radius
        color: Theme.shadow
    }

    Squircle {
        id: surface
        anchors.fill: parent
        radius: Theme.radius.xl
        fillColor: Theme.surface
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        onWheel: (wheel) => wheel.accepted = true
    }

    ColumnLayout {
        id: layout
        anchors.fill: parent
        anchors.margins: Theme.space.lg
        anchors.leftMargin: Theme.space.xl
        spacing: Theme.space.sm

        RowLayout {
            Layout.fillWidth: true

            Text {
                Layout.fillWidth: true
                text: root._view.stepText
                color: Theme.textMuted
                font.pixelSize: Theme.type.caption.size
            }

            IconButton {
                icon: "close"
                size: 28
                rounded: true
                onClicked: root.closeClicked()
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space.md

            Loader {
                id: art
                Layout.preferredHeight: 60
                Layout.preferredWidth: art.item ? art.item.implicitWidth * 60 / art.item.implicitHeight : 0
                Layout.alignment: Qt.AlignVCenter
                active: root._view.art !== null
                visible: active
                sourceComponent: root._view.art
            }

            Text {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter
                text: root._view.title
                color: Theme.text
                font.pixelSize: Theme.type.headline.size
                font.weight: Theme.type.headline.weight
                wrapMode: Text.Wrap
            }
        }

        Text {
            Layout.fillWidth: true
            text: root._view.body
            color: Theme.text
            font.pixelSize: Theme.type.subtitle.size
            wrapMode: Text.Wrap
        }

        Text {
            Layout.fillWidth: true
            visible: text !== ""
            text: root._view.note
            color: Theme.textMuted
            font.pixelSize: Theme.type.body.size
            wrapMode: Text.Wrap
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.topMargin: Theme.space.sm
            spacing: Theme.space.sm

            Text {
                Layout.fillWidth: true
                text: root._view.hint
                color: Theme.accent
                font.pixelSize: Theme.type.label.size
                font.weight: Theme.type.label.weight
                wrapMode: Text.Wrap
            }

            M3Button {
                visible: root._view.canBack
                text: qsTr("Back")
                variant: "tonal"
                small: true
                onClicked: root.backClicked()
            }

            M3Button {
                id: nextBtn
                visible: root._view.canNext
                text: root._view.isLast ? qsTr("Done") : qsTr("Next")
                small: true
                onClicked: root.nextClicked()
            }
        }
    }
}
