import QtQuick
import omikuji 1.0
import QtQuick.Layouts
import "../controls"
import "../downloads"

ColumnLayout {
    id: root

    property var details: null
    property string kind: "about"
    property string title: ""

    readonly property var reqs: details && details.reqs ? details.reqs : []
    readonly property string description: details && details.description ? details.description : ""
    readonly property bool hasRec: {
        for (let i = 0; i < reqs.length; i++) {
            if (reqs[i].recommended && reqs[i].recommended !== "") return true
        }
        return false
    }

    spacing: Theme.space.md

    CapsLabel {
        text: root.kind === "about" ? qsTr("About") : qsTr("System requirements")
    }

    NoteChip {
        visible: root.kind === "about" && root.title !== ""
        Layout.fillWidth: true
        icon: "orbit"
        tone: protonDbHover.containsMouse ? Theme.accent : Theme.textMuted
        text: qsTr("Look it up on ProtonDB")

        MouseArea {
            id: protonDbHover
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: Qt.openUrlExternally(
                "https://www.protondb.com/search?q=" + encodeURIComponent(root.title))
        }
    }

    Text {
        visible: root.kind === "about"
        Layout.fillWidth: true
        text: root.description
        color: Theme.textMuted
        font.pixelSize: Theme.type.label.size
        wrapMode: Text.WordWrap
        textFormat: Text.PlainText
    }

    GridLayout {
        visible: root.kind === "reqs"
        Layout.fillWidth: true
        flow: GridLayout.TopToBottom
        rows: root.reqs.length + 1
        columnSpacing: Theme.space.lg
        rowSpacing: Theme.space.sm

        Item { Layout.preferredWidth: 1; Layout.preferredHeight: 1 }

        Repeater {
            model: root.kind === "reqs" ? root.reqs : []

            Text {
                required property var modelData
                Layout.alignment: Qt.AlignTop
                text: modelData.title
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
            }
        }

        Text {
            Layout.alignment: Qt.AlignTop
            text: qsTr("Minimum")
            color: Theme.text
            font.pixelSize: Theme.type.caption.size
            font.weight: Font.Medium
        }

        Repeater {
            model: root.kind === "reqs" ? root.reqs : []

            Text {
                required property var modelData
                Layout.fillWidth: true
                Layout.preferredWidth: 10
                Layout.alignment: Qt.AlignTop
                text: modelData.minimum
                color: Theme.textMuted
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }
        }

        Text {
            visible: root.hasRec
            Layout.alignment: Qt.AlignTop
            text: qsTr("Recommended")
            color: Theme.text
            font.pixelSize: Theme.type.caption.size
            font.weight: Font.Medium
        }

        Repeater {
            model: (root.kind === "reqs" && root.hasRec) ? root.reqs : []

            Text {
                required property var modelData
                Layout.fillWidth: true
                Layout.preferredWidth: 10
                Layout.alignment: Qt.AlignTop
                text: modelData.recommended
                color: Theme.textMuted
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }
        }
    }

}
