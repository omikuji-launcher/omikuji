pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"

DialogCard {
    id: root

    property string source: ""
    property string caption: ""

    maxWidth: 720
    scrollable: false

    function show(source_, caption_) {
        source = ""
        source = source_
        caption = caption_ === undefined ? "" : caption_
        open()
    }

    onCloseRequested: root.close()

    body: Column {
        id: col
        width: parent.width
        spacing: Theme.space.md

        Image {
            id: preview

            readonly property real maxUpscale:
                Math.max(1, 256 / Math.max(1, implicitWidth, implicitHeight))
            readonly property real fit: Math.min(
                col.width / Math.max(1, implicitWidth),
                480 / Math.max(1, implicitHeight),
                maxUpscale)

            anchors.horizontalCenter: parent.horizontalCenter
            width: implicitWidth * fit
            height: implicitHeight * fit
            source: root.source
            fillMode: Image.PreserveAspectFit
            smooth: true
            asynchronous: true
            cache: false
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: preview.implicitWidth + " x " + preview.implicitHeight
            color: Theme.textSubtle
            font.pixelSize: Theme.type.micro.size
            visible: preview.implicitWidth > 0
        }

        Text {
            width: parent.width
            text: root.caption
            color: Theme.textMuted
            font.pixelSize: Theme.type.caption.size
            font.family: "monospace"
            wrapMode: Text.WrapAnywhere
            visible: text.length > 0
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Close")
            variant: "text"
            onClicked: root.close()
        }
    }
}
