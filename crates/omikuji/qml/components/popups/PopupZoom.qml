import QtQuick
import omikuji 1.0
import QtQuick.Controls

QtObject {
    id: zoomer

    required property Popup target

    readonly property Scale xform: Scale {
        xScale: Theme.uiScale
        yScale: Theme.uiScale
    }

    readonly property Connections conn: Connections {
        target: zoomer.target
        function onAboutToShow() {
            let host = zoomer.target.contentItem.parent
            if (host && host.transform.length === 0) host.transform = zoomer.xform
        }
    }
}
