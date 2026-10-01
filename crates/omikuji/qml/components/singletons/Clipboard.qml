pragma Singleton
import QtQuick

QtObject {
    readonly property TextEdit buffer: TextEdit {}

    function copy(text) {
        buffer.text = text
        buffer.selectAll()
        buffer.copy()
        buffer.text = ""
    }
}
