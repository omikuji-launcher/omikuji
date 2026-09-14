import QtQuick

MouseArea {
    acceptedButtons: Qt.NoButton
    onWheel: (wheel) => wheel.accepted = true
}
