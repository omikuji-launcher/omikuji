pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Controls

DialogCard {
    id: popup

    property var icons: {
        try { return JSON.parse(appSettings.availableIconsJson()) }
        catch (e) { return [] }
    }
    property string selected: ""

    signal picked(string name)

    title: qsTr("Pick an icon")
    sizeKey: "icon_picker"
    maxWidth: 460
    preferredHeight: 420
    scrollable: false
    fillHeight: true
    z: 2500

    function show(current) {
        popup.selected = current || ""
        popup.open()
    }
    function hide() { popup.close() }

    onCloseRequested: close()

    body: Item {
        height: parent.height

        Flickable {
            id: iconFlick
            anchors.fill: parent
            contentWidth: width
            contentHeight: grid.height
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ThinScrollBar {}

            Grid {
                id: grid

                readonly property int cell: 46

                x: (iconFlick.width - width) / 2
                columns: Math.max(1, Math.floor((iconFlick.width + spacing) / (cell + spacing)))
                spacing: 6

                Repeater {
                    model: popup.icons

                    Item {
                        id: iconCell
                        required property string modelData

                        width: grid.cell
                        height: grid.cell

                        Rectangle {
                            anchors.fill: parent
                            radius: 10
                            color: popup.selected === iconCell.modelData
                                ? Theme.alpha(Theme.accent, 0.18)
                                : tapArea.containsMouse
                                    ? Theme.stateHover
                                    : "transparent"
                            border.width: popup.selected === iconCell.modelData ? 1 : 0
                            border.color: Theme.accent
                            Behavior on color { ColorAnimation { duration: Theme.dur.xs } }
                        }

                        SvgIcon {
                            anchors.centerIn: parent
                            name: iconCell.modelData
                            size: 22
                            color: popup.selected === iconCell.modelData ? Theme.accent : Theme.icon
                        }

                        PressArea {
                            id: tapArea
                            anchors.fill: parent
                            hoverEnabled: true
                            ringRadius: 10
                            onActivated: {
                                popup.picked(iconCell.modelData)
                                popup.hide()
                            }
                        }
                    }
                }
            }
        }
    }
}
