import QtQuick
import "../controls"

DialogCard {
    id: root
    sizeKey: "template_vars"

    property var uiSettings: null
    property var gameModel: null

    maxWidth: 620
    title: qsTr("Template literals")

    onCloseRequested: close()

    body: Column {
        width: parent.width
        spacing: theme.space.md

        Text {
            width: parent.width
            text: qsTr("Custom ${variable} tokens, usable in launch fields, prefix, install paths, scripts and image overrides. Values may reference the built-ins below.")
            color: theme.textSubtle
            font.pixelSize: theme.type.label.size
            wrapMode: Text.WordWrap
        }

        Text {
            width: parent.width
            text: "${exe}              " + qsTr("game executable") + "\n"
                + "${game_dir}         " + qsTr("folder containing the executable") + "\n"
                + "${game_prefix}      " + qsTr("the game's resolved prefix") + "\n"
                + "${game_id}          " + qsTr("internal game id") + "\n"
                + "${game_name}        " + qsTr("game name") + "\n"
                + "${home}             " + qsTr("home folder") + "\n"
                + "${prefixes_path}    " + qsTr("prefixes root") + "\n"
                + "${cache_path}       " + qsTr("cache root") + "\n"
                + "${scripts_path}     " + qsTr("install scripts root") + "\n"
                + "${data_path}        " + qsTr("omikuji data root") + "\n"
                + "${gachas_path}      " + qsTr("gacha manifests root") + "\n"
                + "${components_path}  " + qsTr("components root") + "\n"
                + "${runners_path}     " + qsTr("runners root") + "\n"
                + "${layers_path}      " + qsTr("layers root") + "\n"
                + "${logs_path}        " + qsTr("logs root") + "\n"
                + "${runtime_path}     " + qsTr("runtime root")
            color: theme.textMuted
            font.pixelSize: theme.type.caption.size
            font.family: "monospace"
        }

        KeyValueTable {
            width: parent.width
            json: root.uiSettings ? root.uiSettings.templateVarsJson() : "{}"
            keyPlaceholder: "my_var"
            valuePlaceholder: "/some/path or ${prefixes_path}"
            addLabel: qsTr("Add variable")
            gameModel: root.gameModel
            onChanged: (j) => { if (root.uiSettings) root.uiSettings.applyTemplateVarsJson(j) }
        }
    }

    actions: M3Button {
        text: qsTr("Close")
        variant: "tonal"
        onClicked: root.close()
    }
}
