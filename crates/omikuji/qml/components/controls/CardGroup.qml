import QtQuick
import omikuji 1.0

Squircle {
    id: root

    default property alias content: rows.data

    width: parent ? parent.width : 400
    height: rows.implicitHeight
    radius: Theme.radius.md
    fillColor: Theme.cardBg

    Column {
        id: rows
        width: parent.width
    }
}
