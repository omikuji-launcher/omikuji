import QtQuick
import QtQuick.Controls

QtObject {
    id: zoomer

    required property Popup target

    readonly property Scale xform: Scale {
        xScale: theme.uiScale
        yScale: theme.uiScale
    }

    readonly property Connections conn: Connections {
        target: zoomer.target
        function onAboutToShow() {
            let host = zoomer.target.contentItem.parent
            if (host && host.transform.length === 0) host.transform = zoomer.xform
        }
    }
}
