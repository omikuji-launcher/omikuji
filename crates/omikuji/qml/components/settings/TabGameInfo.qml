import QtQuick
import QtQuick.Layouts

import "."
import "../widgets"

Item {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var gameModel: null

    signal refetchMediaRequested()

    implicitHeight: content.height

    Column {
        id: content
        width: parent.width
        spacing: 24

        SettingsSection {
            label: "Metadata"
            icon: "sports_esports"
            width: parent.width

            M3TextField {
                label: "Name"
                text: config["meta.name"] || ""
                width: parent.width
                onTextEdited: updateField("meta.name", text)
            }

            M3TextField {
                label: "Sort Name"
                placeholder: "optional, for custom sort order"
                text: config["meta.sort_name"] || ""
                width: parent.width
                onTextEdited: updateField("meta.sort_name", text)
            }

            M3TextField {
                label: "Slug"
                placeholder: "for API lookups (auto-derived from name)"
                text: config["meta.slug"] || ""
                width: parent.width
                onTextEdited: updateField("meta.slug", text)
            }

            M3Dropdown {
                label: "Runner"
                width: parent.width
                options: [
                    { label: "Wine", value: "wine" },
                    { label: "Steam", value: "steam" },
                    { label: "Native", value: "native" },
                    { label: "Flatpak", value: "flatpak" }
                ]
                currentIndex: {
                    let t = config["runner.type"] || "wine"
                    for (let i = 0; i < options.length; i++) {
                        if (options[i].value === t) return i
                    }
                    return 0
                }
                onSelected: (val) => updateField("runner.type", val)
            }

            Text {
                width: parent.width
                visible: gameModel ? gameModel.is_flatpak() : false
                text: "It seems you're using a flatpak build, cutie. Make sure omikuji has the proper extra permissions set to run native or flatpak applications."
                color: theme.warning
                font.pixelSize: 12
                font.weight: Font.Medium
                wrapMode: Text.WordWrap
            }
        }

        SettingsSection {
            label: "Images"
            icon: "image"
            width: parent.width

            M3FileField {
                label: "Banner Override"
                placeholder: "empty = auto-fetch from SGDB"
                text: config["meta.banner"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("meta.banner", t)
            }

            M3FileField {
                label: "Cover Art Override"
                placeholder: "empty = auto-fetch from SGDB"
                text: config["meta.coverart"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("meta.coverart", t)
            }

            M3FileField {
                label: "Icon Override"
                placeholder: "empty = auto-fetch from SGDB"
                text: config["meta.icon"] || ""
                width: parent.width
                gameModel: root.gameModel
                onTextEdited: (t) => updateField("meta.icon", t)
            }

            Item {
                id: refetchBtn
                width: refetchRow.implicitWidth + 24
                height: 32

                Rectangle {
                    anchors.fill: parent
                    radius: 8
                    color: refetchMouse.containsPress
                        ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.14)
                        : (refetchMouse.containsMouse
                            ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.08)
                            : "transparent")
                    border.width: 1
                    border.color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.18)

                    Behavior on color { ColorAnimation { duration: 100 } }
                }

                Row {
                    id: refetchRow
                    anchors.centerIn: parent
                    spacing: 6

                    SvgIcon {
                        name: "sync"
                        size: 14
                        color: theme.text
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Text {
                        text: "Refetch art"
                        color: theme.text
                        font.pixelSize: 12
                        font.weight: Font.Medium
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }

                MouseArea {
                    id: refetchMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.refetchMediaRequested()
                }
            }

            M3TextField {
                label: "Color"
                placeholder: "#1a1a2e"
                text: config["meta.color"] || ""
                width: parent.width
                onTextEdited: updateField("meta.color", text)
            }
        }
    }
}
