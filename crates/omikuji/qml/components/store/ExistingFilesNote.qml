pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"
import "../lib/Format.js" as Format

NoteChip {
    id: root

    property real bytes: 0
    property bool hasResume: false

    icon: "info"
    tone: Theme.accent
    visible: text !== ""

    text: {
        if (root.bytes <= 0 && !root.hasResume) return ""
        if (root.hasResume) {
            return root.bytes > 0
                ? qsTr("Found %1 of existing files with resume data, the download continues where it left off.")
                    .arg(Format.formatBytesShort(root.bytes))
                : qsTr("Found resume data, the download continues where it left off.")
        }
        return qsTr("Found %1 of existing files at this path.").arg(Format.formatBytesShort(root.bytes))
    }
}
