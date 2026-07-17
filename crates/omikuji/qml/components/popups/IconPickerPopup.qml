import QtQuick
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects
import "../primitives"

Item {
    id: popup

    property var icons: {
        try { return JSON.parse(uiSettings.availableIconsJson()) }
        catch (e) { return [] }
    }
    property string selected: ""

    signal picked(string name)

    visible: false
    z: 2500

    function show(current) {
        popup.selected = current || ""
        popup.visible = true
        popup.forceActiveFocus()
    }
    function hide() { popup.visible = false }

    Keys.onEscapePressed: (event) => { popup.hide(); event.accepted = true }

    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, 0.55)
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.AllButtons
            onClicked: (mouse) => { if (mouse.button === Qt.LeftButton) popup.hide() }
            onWheel: (wheel) => wheel.accepted = true
            cursorShape: Qt.ArrowCursor
        }
    }

    Rectangle {
        id: card
        anchors.centerIn: parent
        width: Math.min(parent.width - 80, 460)
        height: Math.min(parent.height - 120, 420)
        radius: theme.radius.xl
        color: theme.surface
        border.width: 1
        border.color: theme.alpha(theme.text, 0.08)

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            onClicked: {}
            onWheel: (wheel) => wheel.accepted = true
        }

        layer.enabled: true
        layer.effect: DropShadow {
            radius: 24
            samples: 32
            color: Qt.rgba(0, 0, 0, 0.4)
            horizontalOffset: 0
            verticalOffset: 6
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 22
            spacing: 14

            Text {
                text: qsTr("Pick an icon")
                color: theme.text
                font.pixelSize: theme.type.title.size
                font.weight: Font.DemiBold
            }

            Flickable {
                Layout.fillWidth: true
                Layout.fillHeight: true
                contentWidth: parent.width
                contentHeight: grid.height
                clip: true
                boundsBehavior: Flickable.StopAtBounds

                Grid {
                    id: grid
                    columns: Math.max(1, Math.floor(parent.width / 52))
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
}
