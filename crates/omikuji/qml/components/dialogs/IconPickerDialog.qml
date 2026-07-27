import QtQuick
import QtQuick.Controls
import "../primitives"

DialogCard {
    id: popup

    property var icons: {
        try { return JSON.parse(uiSettings.availableIconsJson()) }
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
                columns: Math.max(1, Math.floor(iconFlick.width / 52))
                spacing: 6

                Repeater {
                    model: popup.icons

                    Item {
                        required property string modelData

                        width: 46
                        height: 46

                        Rectangle {
                            anchors.fill: parent
                            radius: 10
                            color: popup.selected === modelData
                                ? theme.alpha(theme.accent, 0.18)
                                : tapArea.containsMouse
                                    ? theme.alpha(theme.text, 0.08)
                                    : "transparent"
                            border.width: popup.selected === modelData ? 1 : 0
                            border.color: theme.accent
                            Behavior on color { ColorAnimation { duration: 100 } }
                        }

                        SvgIcon {
                            anchors.centerIn: parent
                            name: modelData
                            size: 22
                            color: popup.selected === modelData ? theme.accent : theme.icon
                        }

                        MouseArea {
                            id: tapArea
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                popup.picked(modelData)
                                popup.hide()
                            }
                        }
                    }
                }
            }
        }
    }
}
