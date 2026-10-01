pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

IconButton {
    id: btn

    property var envAsShell: function() { return "" }

    icon: "content_copy"
    size: 32
    onClicked: Clipboard.copy([btn.envAsShell(), "%command%"].filter(s => s).join(" "))

    Tooltip {
        text: qsTr("Copy current envs and dlls keys and values in your clipboard in Steam's format")
        tipVisible: btn.hovered
        y: parent.height + 8
    }
}
