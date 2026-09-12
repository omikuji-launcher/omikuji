import QtQuick
import omikuji 1.0

import "../controls"

Chip {
    id: root

    property string kind: "install"

    readonly property var labels: ({
        install: qsTr("Install"),
        update: qsTr("Update"),
        repair: qsTr("Repair"),
        "import": qsTr("Import")
    })

    tone: root.kind === "repair" ? Theme.warning : Theme.accent
    text: root.labels[root.kind] || root.kind
}
