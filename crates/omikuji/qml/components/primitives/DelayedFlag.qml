import QtQuick

QtObject {
    id: root

    property bool source: false
    property int delay: 120
    property bool value: false

    function settle() {
        _timer.stop()
        value = source
    }

    onSourceChanged: {
        if (source) {
            _timer.restart()
            return
        }
        _timer.stop()
        value = false
    }

    readonly property Timer _timer: Timer {
        interval: root.delay
        onTriggered: root.value = true
    }
}
