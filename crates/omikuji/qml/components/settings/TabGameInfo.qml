import QtQuick
import QtQuick.Layouts

import "."
import "../controls"
import "../primitives"

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
            label: qsTr("Metadata")
            icon: "sports_esports"
            width: parent.width

            M3TextField {
                label: qsTr("Name")
                text: config["meta.name"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("meta.name", t)
            }

            M3TextField {
                label: qsTr("Sort Name")
                placeholder: qsTr("optional, for custom sort order")
                text: config["meta.sort_name"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("meta.sort_name", t)
            }

            M3TextField {
                label: qsTr("Slug")
                placeholder: qsTr("for API lookups (auto-derived from name)")
                text: config["meta.slug"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("meta.slug", t)
            }

            M3Dropdown {
                label: qsTr("Runner")
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
                text: qsTr("It seems you're using a flatpak build, cutie. Make sure omikuji has the proper extra permissions set to run native or flatpak applications.")
                color: theme.warning
                font.pixelSize: theme.type.micro.size
                font.weight: Font.Medium
                wrapMode: Text.WordWrap
            }
        }

        SettingsSection {
            label: qsTr("Images")
            icon: "image"
            width: parent.width

            M3FileField {
                label: qsTr("Banner Override")
                placeholder: qsTr("empty = auto-fetch from SGDB")
                text: config["meta.banner"] || ""
                width: parent.width
                gameModel: root.gameModel
                expandWith: root.gameModel ? (t) => root.gameModel.expandGlobalVars(t) : null
                onTextEdited: (t) => updateField("meta.banner", t)
            }

            M3FileField {
                label: qsTr("Cover Art Override")
                placeholder: qsTr("empty = auto-fetch from SGDB")
                text: config["meta.coverart"] || ""
                width: parent.width
                gameModel: root.gameModel
                expandWith: root.gameModel ? (t) => root.gameModel.expandGlobalVars(t) : null
                onTextEdited: (t) => updateField("meta.coverart", t)
            }

            M3FileField {
                label: qsTr("Icon Override")
                placeholder: qsTr("empty = auto-fetch from SGDB")
                text: config["meta.icon"] || ""
                width: parent.width
                gameModel: root.gameModel
                expandWith: root.gameModel ? (t) => root.gameModel.expandGlobalVars(t) : null
                onTextEdited: (t) => updateField("meta.icon", t)
            }

            M3Button {
                small: true
                variant: "tonal"
                icon: "sync"
                text: qsTr("Refetch art")
                onClicked: root.refetchMediaRequested()
            }

            M3TextField {
                label: qsTr("Color")
                placeholder: "#1a1a2e"
                text: config["meta.color"] || ""
                width: parent.width
                onTextEdited: (t) => updateField("meta.color", t)
            }
        }
    }
}
