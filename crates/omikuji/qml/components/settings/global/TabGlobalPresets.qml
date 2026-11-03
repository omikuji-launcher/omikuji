import QtQuick
import omikuji 1.0
import QtQuick.Layouts

Item {
    id: root

    signal manageSetsRequested(string kind)

    implicitHeight: content.height

    component ManageRow: RowLayout {
        id: row

        required property string text
        signal manageClicked()

        width: parent.width

        Text {
            Layout.fillWidth: true
            text: row.text
            color: Theme.text
            font.pixelSize: Theme.type.subtitle.size
            wrapMode: Text.WordWrap
        }

        M3Button {
            text: qsTr("Manage")
            variant: "tonal"
            onClicked: row.manageClicked()
        }
    }

    Column {
        id: content
        width: parent.width
        spacing: Theme.space.xxl

        SettingsSection {
            label: qsTr("Environment Sets")
            icon: "view_list"
            width: parent.width

            ManageRow {
                text: qsTr("Reusable env sets, applied or copied per game")
                onManageClicked: root.manageSetsRequested("env")
            }
        }

        SettingsSection {
            label: qsTr("DLL Override Sets")
            icon: "view_list"
            width: parent.width

            ManageRow {
                text: qsTr("Reusable DLL override sets, applied or copied per game")
                onManageClicked: root.manageSetsRequested("dll")
            }
        }

        SettingsSection {
            label: qsTr("Template Literals")
            icon: "code"
            width: parent.width

            ManageRow {
                text: qsTr("Custom ${variable} tokens for launch fields, prefix, install paths, scripts and image overrides")
                onManageClicked: root.manageSetsRequested("vars")
            }
        }
    }
}
