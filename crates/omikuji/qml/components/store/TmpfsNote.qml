pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

NoteChip {
    id: root

    property var gameModel: null
    property string path: ""

    readonly property string trimmedPath: path.trim()

    icon: "warning"
    tone: Theme.warning
    visible: gameModel !== null && trimmedPath !== "" && gameModel.is_tmpfs(trimmedPath)
    text: qsTr("This folder lives in RAM. Anything installed here can disappear when Omikuji closes or on reboot.")
}
