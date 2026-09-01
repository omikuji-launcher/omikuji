pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

import "."
import "../controls"

Item {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var gameModel: null
    property string gameId: ""

    readonly property bool canRefetchMedia: gameId !== ""

    signal refetchMediaRequested()
    signal previewImageRequested(string source, string caption)

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
                text: root.config["meta.name"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("meta.name", t)
            }

            M3TextField {
                label: qsTr("Sort Name")
                placeholder: qsTr("optional, for custom sort order")
                text: root.config["meta.sort_name"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("meta.sort_name", t)
            }

            M3TextField {
                label: qsTr("Slug")
                placeholder: qsTr("for API lookups (auto-derived from name)")
                text: root.config["meta.slug"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("meta.slug", t)
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
                    let t = root.config["runner.type"] || "wine"
                    for (let i = 0; i < options.length; i++) {
                        if (options[i].value === t) return i
                    }
                    return 0
                }
                onSelected: (val) => root.updateField("runner.type", val)
            }

            NoteChip {
                width: parent.width
                visible: root.gameModel ? root.gameModel.is_flatpak() : false
                icon: "warning"
                tone: Theme.warning
                text: qsTr("It seems you're using a flatpak build, cutie. Make sure omikuji has the proper extra permissions set to run native or flatpak applications.")
            }
        }

        SettingsSection {
            label: qsTr("Images")
            icon: "image"
            width: parent.width

            Repeater {
                model: [
                    { kind: "banner",   label: qsTr("Banner Override") },
                    { kind: "coverart", label: qsTr("Cover Art Override") },
                    { kind: "icon",     label: qsTr("Icon Override") }
                ]

                ImageOverrideField {
                    required property var modelData

                    kind: modelData.kind
                    label: modelData.label
                    width: root.width
                    gameId: root.gameId
                    gameModel: root.gameModel
                    config: root.config
                    updateField: root.updateField
                    onPreviewRequested: (src, caption) => root.previewImageRequested(src, caption)
                }
            }

            M3Button {
                small: true
                variant: "tonal"
                icon: "sync"
                text: qsTr("Refetch art")
                enabled: root.canRefetchMedia
                onClicked: root.refetchMediaRequested()
            }

            M3TextField {
                label: qsTr("Color")
                placeholder: "#1a1a2e"
                text: root.config["meta.color"] || ""
                width: parent.width
                onTextEdited: (t) => root.updateField("meta.color", t)
            }
        }
    }
}
