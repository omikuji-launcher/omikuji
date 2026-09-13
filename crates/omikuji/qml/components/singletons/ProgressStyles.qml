pragma Singleton

import QtQuick

QtObject {
    function options() {
        return [
            { label: qsTr("Wavy"), value: "wavy" },
            { label: qsTr("Flat"), value: "flat" },
            { label: qsTr("Chunky"), value: "chunky" },
            { label: qsTr("Segmented"), value: "segmented" },
            { label: qsTr("Striped"), value: "striped" },
            { label: qsTr("Rope"), value: "rope" }
        ]
    }
}
