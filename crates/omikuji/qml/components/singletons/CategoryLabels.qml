pragma Singleton

import QtQuick

QtObject {
    function label(entry) {
        if (entry.auto_name) {
            switch (entry.kind) {
            case "all":       return qsTr("All Games")
            case "favourite": return qsTr("Favourites")
            case "recent":    return qsTr("Recent")
            case "runner":
                if (entry.value === "wine")   return qsTr("Wine", "runner name, a proper noun; leave untranslated")
                if (entry.value === "native") return qsTr("Native")
            }
        }
        return entry.name || ""
    }
}
