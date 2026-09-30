import QtQuick
import omikuji 1.0

Chip {
    id: root

    property string kind: "install"

    readonly property var labels: ({
        install: qsTr("Install"),
        update: qsTr("Update"),
        predownload: qsTr("Pre-download"),
        repair: qsTr("Repair"),
        "import": qsTr("Import"),
        add_pack: qsTr("Add pack")
    })

    tone: root.kind === "repair" ? Theme.warning : Theme.secondary
    text: root.labels[root.kind] || root.kind
}
