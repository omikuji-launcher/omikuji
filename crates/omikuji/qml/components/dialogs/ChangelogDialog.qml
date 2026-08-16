pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Controls
import QtQuick.Layouts
import "../controls"
import "../primitives"

DialogCard {
    id: root
    sizeKey: "changelog"

    property string fromVersion: ""
    property string toVersion: ""
    property string bodyText: ""

    signal dismissed()

    maxWidth: 640

    function show(payload) {
        if (payload) {
            fromVersion = payload.from || ""
            toVersion = payload.to || ""
            bodyText = payload.body || ""
        }
        open()
    }

    onCloseRequested: { root.dismissed(); root.close() }

    function _hex2(n) {
        let s = Math.round(n * 255).toString(16)
        return s.length < 2 ? "0" + s : s
    }
    function _css(c) { return "#" + _hex2(c.r) + _hex2(c.g) + _hex2(c.b) }
    function _esc(s) {
        return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
    }
    function _sectionColor(key) {
        switch (key.toLowerCase()) {
        case "added": return _css(Theme.accent)
        case "fixed": case "fixes": case "fix": return _css(Theme.warning)
        case "removed": case "removal": return _css(Theme.error)
        case "changed": case "change": return "#6ea8fe"
        default: return _css(Theme.text)
        }
    }
    function _html(body) {
        let out = ""
        for (let raw of body.split("\n")) {
            let line = raw.replace(/\s+$/, "")
            let vh = line.match(/^##\s+(.+)$/)
            if (vh) {
                out += '<span style="color:' + _css(Theme.textMuted) + ';font-weight:bold">' + _esc(vh[1]) + '</span><br>'
                continue
            }
            let sec = line.match(/^([A-Za-z]+):(.*)$/)
            if (sec) {
                out += '<span style="color:' + root._sectionColor(sec[1]) + ';font-weight:bold">' + _esc(sec[1]) + ':</span>' + _esc(sec[2]) + '<br>'
                continue
            }
            out += _esc(line) + '<br>'
        }
        return out
    }

    body: ColumnLayout {
        width: parent.width
        spacing: Theme.space.lg

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space.sm

            Text {
                text: qsTr("Omikuji")
                color: Theme.text
                font.pixelSize: Theme.type.title.size
                font.weight: Font.DemiBold
            }
            Text {
                text: root.fromVersion
                color: Theme.textMuted
                font.pixelSize: Theme.type.body.size
                font.family: "monospace"
                Layout.alignment: Qt.AlignBaseline
            }
            Text {
                text: "→"
                color: Theme.textMuted
                font.pixelSize: Theme.type.body.size
                Layout.alignment: Qt.AlignBaseline
            }
            Text {
                text: root.toVersion
                color: Theme.accent
                font.pixelSize: Theme.type.body.size
                font.weight: Font.DemiBold
                font.family: "monospace"
                Layout.alignment: Qt.AlignBaseline
            }
            Item { Layout.fillWidth: true }
        }

        Rectangle {
            Layout.fillWidth: true
            radius: Theme.radius.sm
            color: Theme.bgAlt
            implicitHeight: Math.min(Math.max(notes.implicitHeight + Theme.space.md * 2, 320), 480)

            Flickable {
                id: notesFlick
                anchors.fill: parent
                anchors.margins: Theme.space.md
                contentHeight: notes.implicitHeight
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ThinScrollBar {}

                Text {
                    id: notes
                    width: notesFlick.width
                    text: root._html(root.bodyText)
                    textFormat: Text.RichText
                    color: Theme.text
                    font.pixelSize: Theme.type.body.size
                    wrapMode: Text.Wrap
                    lineHeight: 1.3
                }
            }
        }
    }

    footerLeft: M3Button {
        text: qsTr("Open repository")
        variant: "tonal"
        onClicked: Qt.openUrlExternally("https://github.com/reakjra/omikuji")
    }

    actions: Row {
        spacing: Theme.space.sm
        M3Button {
            text: qsTr("Got it")
            variant: "filled"
            onClicked: { root.dismissed(); root.close() }
        }
    }
}
