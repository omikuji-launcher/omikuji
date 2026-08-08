import QtQuick
import omikuji 1.0

Rectangle {
    id: root

    property string kind: "install"

    readonly property color tone: kind === "repair" ? Theme.warning : Theme.accent
    readonly property var labels: ({
        install: qsTr("Install"),
        update: qsTr("Update"),
        repair: qsTr("Repair"),
        "import": qsTr("Import")
    })

    width: label.implicitWidth + 16
    height: 22
    radius: Theme.radius.xs
    color: Theme.alpha(tone, 0.16)

    Text {
        id: label
        anchors.centerIn: parent
        text: root.labels[root.kind] || root.kind
        color: root.tone
        font.pixelSize: Theme.type.micro.size
        font.weight: Font.DemiBold
        font.capitalization: Font.AllUppercase
        font.letterSpacing: 0.5
    }
}
