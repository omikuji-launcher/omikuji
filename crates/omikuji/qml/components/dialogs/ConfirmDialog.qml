pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"

DialogCard {
    id: root

    property string message: ""
    property string detail: ""
    property string confirmText: qsTr("Confirm")
    property string cancelText: qsTr("Cancel")
    property bool destructive: false
    property var payload: null

    signal confirmed(var payload)
    signal cancelled(var payload)

    maxWidth: 420

    function show(payload_) {
        payload = payload_ === undefined ? null : payload_
        open()
    }
    function hide() { close() }

    onCloseRequested: { root.cancelled(root.payload); root.close() }

    body: Column {
        width: parent.width
        spacing: Theme.space.sm

        Text {
            width: parent.width
            text: root.detail
            color: Theme.accent
            font.pixelSize: Theme.type.caption.size
            font.family: "monospace"
            wrapMode: Text.WrapAnywhere
            visible: text.length > 0
        }
        Text {
            width: parent.width
            text: root.message
            color: Theme.textMuted
            font.pixelSize: Theme.type.body.size
            wrapMode: Text.Wrap
            visible: text.length > 0
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: root.cancelText
            variant: "text"
            onClicked: { root.cancelled(root.payload); root.close() }
        }
        M3Button {
            text: root.confirmText
            variant: "filled"
            danger: root.destructive
            onClicked: { root.confirmed(root.payload); root.close() }
        }
    }
}
