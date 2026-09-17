pragma Singleton
import QtQuick
import omikuji 1.0

QtObject {
    readonly property InputModeBridge bridge: InputModeBridge {
        Component.onCompleted: start()
    }

    readonly property bool keyboard: bridge.keyboard

    function keyFocus(item) {
        return keyboard && !!item && item.activeFocus
    }
}
