pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Row {
    id: bar

    property real uiScale: 1.0
    property string controllerKind: "xbox"
    readonly property real _scale: Math.max(0.85, Math.min(uiScale, 1.5))

    readonly property var _glyphTables: ({
        "xbox":     { "south": "A", "east": "B", "west": "X", "north": "Y" },
        "ps":       { "south": "✕", "east": "○", "west": "□", "north": "△" },
        "nintendo": { "south": "B", "east": "A", "west": "Y", "north": "X" },
        "steam":    { "south": "A", "east": "B", "west": "X", "north": "Y" }
    })

    readonly property var _g: _glyphTables[controllerKind] || _glyphTables.xbox

    readonly property var hints: [
        { glyph: _g.south, label: qsTr("Launch") },
        { glyph: _g.east,  label: qsTr("Back") },
        { glyph: _g.north, label: qsTr("Search") },
        { glyph: "✥",      label: qsTr("Navigate") }
    ]

    spacing: 28 * _scale

    Repeater {
        model: bar.hints
        delegate: Row {
            id: hint
            required property var modelData
            spacing: 8 * bar._scale

            Item {
                anchors.verticalCenter: parent.verticalCenter
                width: 28 * bar._scale
                height: 28 * bar._scale

                layer.enabled: true
                layer.smooth: true
                layer.textureSize: Qt.size(width * 2, height * 2)

                Rectangle {
                    anchors.fill: parent
                    radius: width / 2
                    color: Theme.surface
                    border.width: 1
                    border.color: Theme.alpha(Theme.text, 0.15)
                    antialiasing: true
                }

                Text {
                    anchors.fill: parent
                    text: hint.modelData.glyph
                    color: Theme.text
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    font.pixelSize: 14 * bar._scale
                    font.weight: Font.Bold
                }
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: hint.modelData.label
                color: Theme.textMuted
                font.pixelSize: 14 * bar._scale
                font.weight: Font.Medium
            }
        }
    }
}
