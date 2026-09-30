pragma Singleton

import QtQuick

QtObject {
    function section(kind) {
        switch (kind) {
        case "voice":   return qsTr("Voice Packs")
        case "texture": return qsTr("Texture Packs")
        }
        return qsTr("Packs")
    }
}
