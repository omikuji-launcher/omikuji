import QtQuick
import QtQuick.Layouts

import "."
import "../controls"

Item {
    id: root

    signal manageSetsRequested(string kind)

    implicitHeight: content.height

    Column {
        id: content
        width: parent.width
        spacing: theme.space.xxl

        SettingsSection {
            label: qsTr("Environment Sets")
            icon: "view_list"
            width: parent.width

            RowLayout {
                width: parent.width

                Text {
                    Layout.fillWidth: true
                    text: qsTr("Create and edit reusable env sets, applied or copied per-game.")
                    color: theme.textSubtle
                    font.pixelSize: theme.type.label.size
                    wrapMode: Text.WordWrap
                }

                M3Button {
                    text: qsTr("Manage")
                    variant: "tonal"
                    onClicked: root.manageSetsRequested("env")
                }
            }
        }

        SettingsSection {
            label: qsTr("DLL Override Sets")
            icon: "view_list"
            width: parent.width

            RowLayout {
                width: parent.width

                Text {
                    Layout.fillWidth: true
                    text: qsTr("Create and edit reusable DLL override sets, applied or copied per-game.")
                    color: theme.textSubtle
                    font.pixelSize: theme.type.label.size
                    wrapMode: Text.WordWrap
                }

                M3Button {
                    text: qsTr("Manage")
                    variant: "tonal"
                    onClicked: root.manageSetsRequested("dll")
                }
            }
        }

        SettingsSection {
            label: qsTr("Template Literals")
            icon: "code"
            width: parent.width

            RowLayout {
                width: parent.width

                Text {
                    Layout.fillWidth: true
                    text: qsTr("Custom ${variable} tokens, usable in launch fields, prefix, install paths, scripts and image overrides.")
                    color: theme.textSubtle
                    font.pixelSize: theme.type.label.size
                    wrapMode: Text.WordWrap
                }

                M3Button {
                    text: qsTr("Manage")
                    variant: "tonal"
                    onClicked: root.manageSetsRequested("vars")
                }
            }
        }
    }
}
