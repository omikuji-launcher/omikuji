import QtQuick
import omikuji 1.0

MouseArea {
    id: root

    property real ringRadius: Theme.radius.sm

    signal activated()

    readonly property bool navigable: enabled
    readonly property real navRingRadius: ringRadius
    function navActivate() { activated() }

    cursorShape: Qt.PointingHandCursor
    onClicked: activated()
}
