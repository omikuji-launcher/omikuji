pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"

M3FileField {
    id: root

    property string kind: ""
    property string gameId: ""
    property var config: ({})
    property var updateField: function(key, value) {}

    readonly property string configKey: "meta." + kind
    property string resolved: ""

    function refreshResolved() {
        resolved = gameModel ? gameModel.resolve_media(gameId, text, kind) : ""
    }

    onTextChanged: refreshResolved()
    onGameIdChanged: refreshResolved()
    onGameModelChanged: refreshResolved()
    Component.onCompleted: refreshResolved()

    Connections {
        target: root.gameModel
        function onMediaChanged(gameId) {
            if (gameId === root.gameId) root.refreshResolved()
        }
    }

    placeholder: qsTr("empty = auto-fetch from SGDB")
    filter: "*.png *.jpg *.jpeg *.webp"
    text: config[configKey] || ""
    expandWith: gameModel ? (t) => gameModel.expandGlobalVars(t) : null

    signal previewRequested(string source, string caption)
    signal pickRequested(string kind)

    onTextEdited: (t) => root.updateField(root.configKey, t)

    trailingActions: [
        FieldButton {
            icon: "search"
            tooltip: qsTr("Search on SGDB")
            blocked: root.gameId === ""
            onClicked: root.pickRequested(root.kind)
        },
        FieldButton {
            icon: "image"
            tooltip: qsTr("Preview image")
            blocked: root.resolved === ""
            onClicked: root.previewRequested(root.resolved, root.resolved)
        }
    ]
}
