pragma Singleton

import QtQuick

QtObject {
    function label(entry) {
        if (entry.auto_name) {
            switch (entry.kind) {
            case "all":       return qsTr("All Games")
            case "favourite": return qsTr("Favourites")
            case "hidden":    return qsTr("Hidden")
            case "recent":    return qsTr("Recent")
            case "runner":
                if (entry.value === "wine")   return qsTr("Wine", "runner name, a proper noun; leave untranslated")
                if (entry.value === "native") return qsTr("Native")
            }
        }
        return entry.name || ""
    }

    function subtitle(entry) {
        switch (entry.kind) {
        case "all":       return qsTr("all games")
        case "favourite": return qsTr("favourites")
        case "hidden":    return qsTr("hidden games")
        case "recent":    return qsTr("recent (top 10)")
        case "runner":    return qsTr("runner: %1").arg(entry.value || "")
        case "tag":       return qsTr("tag: %1").arg(entry.value || "")
        }
        return entry.kind || ""
    }
}
