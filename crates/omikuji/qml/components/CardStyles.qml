pragma Singleton

import QtQuick

QtObject {
    function options() {
        return [
            { label: qsTr("Normal"), value: "normal" },
            { label: qsTr("Fit"), value: "fit" },
            { label: qsTr("Frameless"), value: "frameless" },
            { label: qsTr("Vignette"), value: "vignette" }
        ]
    }
}
