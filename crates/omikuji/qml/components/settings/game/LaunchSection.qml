import QtQuick
import omikuji 1.0
import "../../lib/RunnerGrouping.js" as RG

SettingsSection {
    id: root

    property var config: ({})
    property var updateField: function(key, value) {}
    property var gameModel: null
    property string exeLabel: qsTr("Executable")
    property string exePlaceholder: ""

    icon: "terminal"

    M3FileField {
        id: exeField
        label: root.exeLabel
        placeholder: root.exePlaceholder
        text: root.config["meta.exe"] || ""
        width: parent.width
        gameModel: root.gameModel
        expandHint: false
        onTextEdited: (t) => root.updateField("meta.exe", t)

        SpotlightTarget { key: "game.exe"; target: exeField }
    }

    M3FileField {
        label: qsTr("Working Directory")
        placeholder: qsTr("executable's folder (auto)")
        text: root.config["launch.working_dir"] || ""
        selectFolder: true
        width: parent.width
        gameModel: root.gameModel
        onTextEdited: (t) => root.updateField("launch.working_dir", t)
    }

    M3TextField {
        label: qsTr("Arguments")
        placeholder: '--skip-intro --windowed --name "Jane Doe"'
        text: root.config["launch.args"] || ""
        width: parent.width
        gameModel: root.gameModel
        onTextEdited: (t) => root.updateField("launch.args", t)
    }

    M3TextField {
        label: qsTr("Command Prefix")
        placeholder: qsTr("prepended to command (e.g. custom wrapper)")
        text: root.config["launch.command_prefix"] || ""
        width: parent.width
        gameModel: root.gameModel
        onTextEdited: (t) => root.updateField("launch.command_prefix", t)
    }

    M3Dropdown {
        readonly property var storeOptions: root.gameModel
            ? JSON.parse(root.gameModel.launch_options(root.gameModel.index_of_id(root.config["meta.id"] || "")))
            : []

        visible: storeOptions.length > 0
        label: qsTr("Launch Option")
        width: parent.width
        options: [{ label: qsTr("Ask every time"), value: "" }]
            .concat(storeOptions.map(o => ({ label: o.title, value: o.id })))
        currentIndex: Math.max(0, RG.indexOfValue(options, root.config["launch.store_option"] || ""))
        onSelected: (v) => root.updateField("launch.store_option", v)
    }
}
