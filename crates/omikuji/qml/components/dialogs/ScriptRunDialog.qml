pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Controls
import "../controls"
import "../primitives"
import "../lib/RunnerGrouping.js" as RG

DialogCard {
    sizeKey: "script_run"
    id: root

    property var scriptsBridge: null
    property var gameModel: null
    property var ofudaBridge: null

    signal installed(string gameId, string gameName)

    readonly property bool busy: scriptsBridge ? scriptsBridge.running : false
    property string tomlPath: ""
    property var detail: ({})
    property var values: ({})
    property int valuesRev: 0
    property string errorText: ""
    property string outputText: ""
    property bool succeeded: false
    property bool sourceExpanded: false
    property bool exeMissing: false
    property string pendingGameJson: ""
    property var prefixOptions: []

    readonly property bool formReady: {
        valuesRev
        let inputs = detail.inputs || []
        for (let i = 0; i < inputs.length; i++) {
            if (inputs[i].kind === "bool") continue
            let v = values[inputs[i].id]
            if (!v || v.trim() === "") return false
        }
        return true
    }

    maxWidth: 640
    title: detail.name || qsTr("Install script")

    function show(path) {
        tomlPath = path
        errorText = ""
        outputText = ""
        succeeded = false
        sourceExpanded = false
        exeMissing = false
        pendingGameJson = ""
        let d = {}
        try { d = JSON.parse(scriptsBridge.loadJson(path)) } catch (e) { d = { error: String(e) } }
        detail = d
        if (d.error) errorText = d.error
        let vals = {}
        for (let input of (d.inputs || []))
            vals[input.id] = input.default || (input.kind === "bool" ? "false" : "")
        values = vals
        valuesRev++
        prefixOptions = ofudaBridge
            ? JSON.parse(ofudaBridge.listJson()).map(p => ({ label: p.name, value: p.path }))
            : []
        open()
    }

    function setValue(id, v) {
        values[id] = v
        valuesRev++
    }

    onCloseRequested: if (!busy) close()

    Connections {
        target: root.scriptsBridge
        enabled: root.scriptsBridge !== null
        function onRunOutput(line) {
            root.outputText += (root.outputText.length ? "\n" : "") + line
        }
        function onRunFinished(ok, error, gameJson, exeMissing) {
            if (ok) {
                if (gameJson === "") root.succeeded = true
                else root.registerGame(gameJson)
            } else {
                root.errorText = (error && error.length > 0) ? error : qsTr("Script failed.")
                root.exeMissing = exeMissing
                root.pendingGameJson = exeMissing ? gameJson : ""
            }
        }
    }

    function registerGame(gameJson) {
        let id = gameModel ? gameModel.register_game_json(gameJson) : ""
        if (id && id.length > 0) {
            succeeded = true
            exeMissing = false
            errorText = ""
            installed(id, detail.gameName || detail.name || "")
        } else {
            errorText = qsTr("Script finished but the game could not be registered.")
        }
    }

    FilePicker {
        id: exePicker
        title: qsTr("Locate the game exe")
        startFolder: "/home"
        onPicked: (path) => {
            if (root.pendingGameJson === "") return
            let game = JSON.parse(root.pendingGameJson)
            game.exe = path
            root.registerGame(JSON.stringify(game))
        }
    }

    function locateExe() {
        exePicker.open()
    }

    body: Column {
        width: parent.width
        spacing: Theme.space.md

        Text {
            width: parent.width
            visible: (root.detail.description || "") !== ""
            text: root.detail.description || ""
            color: Theme.textMuted
            font.pixelSize: Theme.type.label.size
            wrapMode: Text.WordWrap
        }

        Rectangle {
            width: parent.width
            visible: (root.detail.note || "") !== ""
            radius: Theme.radius.md
            color: Theme.alpha(Theme.accent, 0.10)
            height: noteRow.implicitHeight + Theme.space.md * 2

            Row {
                id: noteRow
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.space.md
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.space.sm

                SvgIcon {
                    name: "info"
                    size: 18
                    color: Theme.accent
                    anchors.verticalCenter: parent.verticalCenter
                }
                Text {
                    width: parent.width - 18 - Theme.space.sm
                    text: root.detail.note || ""
                    color: Theme.text
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.WordWrap
                }
            }
        }

        Rectangle {
            width: parent.width
            visible: root.detail.hasShell === true
            radius: Theme.radius.md
            color: Theme.alpha(Theme.error, 0.12)
            height: shellRow.implicitHeight + Theme.space.md * 2

            Row {
                id: shellRow
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.space.md
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.space.sm

                SvgIcon {
                    name: "warning"
                    size: 18
                    color: Theme.error
                    anchors.verticalCenter: parent.verticalCenter
                }
                Text {
                    width: parent.width - 18 - Theme.space.sm
                    text: qsTr("This script runs arbitrary shell commands. Review the source below before installing.")
                    color: Theme.error
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.WordWrap
                }
            }
        }

        Column {
            width: parent.width
            spacing: Theme.space.md
            enabled: !root.busy && !root.succeeded

            Repeater {
                model: root.detail.inputs || []
                delegate: Loader {
                    width: parent.width
                    required property var modelData
                    property var input: modelData
                    sourceComponent: {
                        switch (modelData.kind) {
                        case "file":
                        case "directory": return fileComp
                        case "choice": return choiceComp
                        case "bool": return boolComp
                        case "prefix": return modelData.picker === "path" ? prefixPathComp : prefixListComp
                        case "runner": return runnerComp
                        default: return textComp
                        }
                    }
                     onLoaded: item.input = input
                }
            }
        }

        Component {
            id: textComp
            M3TextField {
                property var input
                label: input.label
                text: root.values[input.id] || ""
                onTextEdited: (t) => root.setValue(input.id, t)
            }
        }
        Component {
            id: fileComp
            M3FileField {
                property var input
                label: input.label
                gameModel: root.gameModel
                selectFolder: input.kind === "directory"
                filter: input.filter || ""
                text: { root.valuesRev; return root.values[input.id] || "" }
                onTextEdited: (t) => root.setValue(input.id, t)
            }
        }
        Component {
            id: choiceComp
            M3Dropdown {
                property var input
                label: input.label
                options: input.options.map(o => ({ label: o, value: o }))
                currentIndex: Math.max(0, input.options.indexOf(root.values[input.id]))
                onSelected: (v) => root.setValue(input.id, v)
                Component.onCompleted: if (!root.values[input.id]) root.setValue(input.id, currentValue)
            }
        }
        Component {
            id: boolComp
            LabeledSwitch {
                property var input
                label: input.label
                checked: root.values[input.id] === "true"
                onToggled: (v) => root.setValue(input.id, v ? "true" : "false")
            }
        }
        Component {
            id: prefixListComp
            Column {
                spacing: Theme.space.xs
                M3Dropdown {
                    property var input
                    width: parent.width
                    label: input.label
                    options: root.prefixOptions
                    enabled: root.prefixOptions.length > 0
                    onSelected: (v) => root.setValue(input.id, v)
                    Component.onCompleted: if (root.prefixOptions.length > 0 && !root.values[input.id]) root.setValue(input.id, currentValue)
                }
                Text {
                    visible: root.prefixOptions.length === 0
                    text: qsTr("No prefixes yet - create one in Settings > Ofuda first.")
                    color: Theme.warning
                    font.pixelSize: Theme.type.caption.size
                }
            }
        }
        Component {
            id: runnerComp
            M3Dropdown {
                property var input
                label: input.label
                options: RG.groupRunners(JSON.parse(root.gameModel ? root.gameModel.list_runners() : "[]"))
                currentIndex: {
                    let f = RG.firstNonHeader(options)
                    return f >= 0 ? f : 0
                }
                onSelected: (v) => root.setValue(input.id, v)
                Component.onCompleted: if (!root.values[input.id]) root.setValue(input.id, currentValue)
            }
        }
        Component {
            id: prefixPathComp
            M3FileField {
                property var input
                label: input.label
                selectFolder: true
                gameModel: root.gameModel
                placeholder: qsTr("/path/to/prefix (created if missing)")
                text: { root.valuesRev; return root.values[input.id] || "" }
                onTextEdited: (t) => root.setValue(input.id, t)
            }
        }

        Column {
            width: parent.width
            spacing: Theme.space.xs

            M3Button {
                small: true
                variant: "tonal"
                icon: "code"
                text: root.sourceExpanded ? qsTr("Hide source") : qsTr("Show source")
                onClicked: root.sourceExpanded = !root.sourceExpanded
            }

            FieldSurface {
                width: parent.width
                height: 220
                visible: root.sourceExpanded

                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.NoButton
                    onWheel: (wheel) => wheel.accepted = true
                }

                Flickable {
                    id: srcFlick
                    anchors.fill: parent
                    anchors.margins: Theme.space.sm
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    contentWidth: Math.max(width, srcText.implicitWidth)
                    contentHeight: srcText.implicitHeight

                    TextEdit {
                        id: srcText
                        text: root.detail.toml || ""
                        readOnly: true
                        selectByMouse: true
                        color: Theme.text
                        font.family: "monospace"
                        font.pixelSize: Theme.type.caption.size
                    }

                    ScrollBar.vertical: ThinScrollBar {}
                    ScrollBar.horizontal: ThinScrollBar {}
                }
            }
        }

        Column {
            width: parent.width
            spacing: Theme.space.xs
            visible: root.busy || root.outputText.length > 0

            Text {
                visible: root.busy
                text: qsTr("Installing…")
                color: Theme.accent
                font.pixelSize: Theme.type.caption.size
            }
            OutputLog {
                width: parent.width
                height: 200
                text: root.outputText
            }
        }

        Rectangle {
            width: parent.width
            visible: root.succeeded
            radius: Theme.radius.md
            color: Theme.alpha(Theme.success, 0.12)
            height: doneRow.implicitHeight + Theme.space.md * 2

            Row {
                id: doneRow
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.space.md
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.space.sm

                SvgIcon {
                    name: "check_circle"
                    size: 18
                    color: Theme.success
                    anchors.verticalCenter: parent.verticalCenter
                }
                Text {
                    width: parent.width - 18 - Theme.space.sm
                    text: root.detail.isUtility === true
                        ? qsTr("Script finished.")
                        : qsTr("%1 was added to your library.").arg(root.detail.gameName || root.detail.name || qsTr("The game"))
                    color: Theme.success
                    font.pixelSize: Theme.type.caption.size
                    wrapMode: Text.WordWrap
                }
            }
        }

        Text {
            width: parent.width
            visible: root.errorText !== ""
            text: root.errorText
            color: Theme.error
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        M3Button {
            visible: root.exeMissing && !root.succeeded
            small: true
            variant: "tonal"
            icon: "folder"
            text: qsTr("Locate game exe…")
            onClicked: root.locateExe()
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Close")
            variant: root.succeeded ? "filled" : "tonal"
            enabled: !root.busy
            onClicked: root.close()
        }
        M3Button {
            visible: !root.succeeded
            text: root.busy ? qsTr("Installing…") : qsTr("Install")
            variant: "filled"
            danger: root.detail.hasShell === true
            enabled: !root.busy && !root.detail.error && root.formReady
            onClicked: {
                root.errorText = ""
                root.outputText = ""
                root.scriptsBridge.run(root.tomlPath, JSON.stringify(root.values))
            }
        }
    }
}
