pragma Singleton

import QtQuick

QtObject {
    function options() {
        return [
            { label: qsTr("Floating"), value: "floating" },
            { label: qsTr("Docked"), value: "docked" }
        ]
    }
}
