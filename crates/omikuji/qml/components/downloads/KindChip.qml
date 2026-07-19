import QtQuick

Rectangle {
    id: root

    property string kind: "install"

    readonly property color tone: kind === "repair" ? theme.warning : theme.accent
    readonly property var labels: ({
        install: qsTr("Install"),
        update: qsTr("Update"),
        repair: qsTr("Repair"),
        "import": qsTr("Import")
    })

    width: label.implicitWidth + 16
    height: 22
    radius: theme.radius.xs
    color: theme.alpha(tone, 0.16)

    Text {
        id: label
        anchors.centerIn: parent
        text: root.labels[root.kind] || root.kind
        color: root.tone
        font.pixelSize: theme.type.micro.size
        font.weight: Font.DemiBold
        font.capitalization: Font.AllUppercase
        font.letterSpacing: 0.5
    }
}
