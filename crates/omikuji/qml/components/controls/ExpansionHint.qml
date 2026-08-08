import QtQuick
import omikuji 1.0

Text {
    id: root

    property string source: ""
    property var resolver: null

    readonly property bool active: resolver !== null && source.indexOf("${") !== -1

    visible: active
    text: active ? resolver(source) : ""
    color: Theme.accent
    font.pixelSize: Theme.type.micro.size
    font.family: "monospace"
    elide: Text.ElideMiddle
}
