import QtQuick
import QtQuick.Controls
import Qt5Compat.GraphicalEffects
import "../controls"
import "../primitives"

DialogCard {
    sizeKey: "script_browser"
    id: root

    property var scriptsBridge: null
    property var gameModel: null

    signal scriptChosen(string tomlPath)

    property string searchText: ""
    property var entries: []
    property var remoteEntries: []
    property bool installingRemote: false
    property string errorText: ""

    function _match(e, q) {
        return e.name.toLowerCase().includes(q)
            || e.description.toLowerCase().includes(q)
            || e.author.toLowerCase().includes(q)
    }

    readonly property var filtered: {
        let q = searchText.trim().toLowerCase()
        let out = []
        for (let e of entries) {
            if (q !== "" && !_match(e, q)) continue
            out.push({
                name: e.name, description: e.description, author: e.author,
                modified: e.modified, hasShell: e.hasShell,
                iconSource: e.icon !== "" ? "file://" + e.icon : "",
                remote: false, toml: e.toml, dir: e.dir
            })
        }
        if (q !== "") {
            let have = new Set(entries.map(e => e.author + "/" + e.dir.split("/").pop()))
            for (let r of remoteEntries) {
                if (!_match(r, q)) continue
                if (have.has(r.author + "/" + r.slug)) continue
                out.push({
                    name: r.name, description: r.description, author: r.author,
                    modified: r.modified, hasShell: r.has_shell,
                    iconSource: r.iconUrl || "",
                    remote: true, raw: r
                })
            }
        }
        return out
    }

    maxWidth: 720
    scrollable: false
    fillHeight: true
    title: qsTr("Install script")

    function show() {
        searchText = ""
        errorText = ""
        installingRemote = false
        remoteEntries = []
        entries = scriptsBridge ? JSON.parse(scriptsBridge.listJson()) : []
        if (scriptsBridge) scriptsBridge.refreshRemote()
        open()
    }

    Connections {
        target: root.scriptsBridge
        enabled: root.scriptsBridge !== null
        function onRemoteListed(ok, json, error) {
            root.remoteEntries = ok ? JSON.parse(json) : []
        }
        function onRemoteInstalled(ok, tomlPath, error) {
            root.installingRemote = false
            if (ok) {
                root.scriptChosen(tomlPath)
                root.close()
            } else {
                root.errorText = error || qsTr("Couldn't fetch the script.")
            }
        }
    }

    onCloseRequested: close()

    property string _dialogRequestId: ""

    Connections {
        target: root.gameModel
        enabled: root._dialogRequestId !== ""
        function onFile_dialog_result(requestId, path) {
            if (requestId !== root._dialogRequestId) return
            root._dialogRequestId = ""
            if (path && path !== "") {
                root.scriptChosen(path)
                root.close()
            }
        }
    }

    function openFilePicker() {
        if (!gameModel) return
        let id = Date.now().toString(36) + Math.random().toString(36).substring(2, 8)
        _dialogRequestId = id
        gameModel.open_file_dialog(id, false, qsTr("Select script"), "/home", "*.toml")
    }

    body: Item {
        height: parent.height

        M3TextField {
            id: searchField
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            placeholder: qsTr("Search scripts…")
            text: root.searchText
            onTextEdited: (t) => root.searchText = t
        }

        Text {
            id: errorLabel
            anchors.top: searchField.bottom
            anchors.topMargin: theme.space.xs
            anchors.left: parent.left
            anchors.right: parent.right
            visible: root.errorText !== ""
            text: root.errorText
            color: theme.error
            font.pixelSize: 12
            wrapMode: Text.WordWrap
        }

        ListView {
            anchors.top: root.errorText !== "" ? errorLabel.bottom : searchField.bottom
            anchors.topMargin: theme.space.md
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            clip: true
            model: root.filtered
            spacing: theme.space.xs
            ScrollBar.vertical: ThinScrollBar {}

            delegate: Rectangle {
                width: ListView.view.width
                height: 56
                radius: theme.radius.md
                color: rowArea.containsMouse ? theme.alpha(theme.text, 0.08) : "transparent"

                Rectangle {
                    id: iconBox
                    width: 36
                    height: 36
                    radius: theme.radius.sm
                    color: theme.alpha(theme.accent, 0.15)
                    anchors.left: parent.left
                    anchors.leftMargin: theme.space.sm
                    anchors.verticalCenter: parent.verticalCenter

                    Image {
                        id: iconImg
                        anchors.fill: parent
                        visible: modelData.iconSource !== "" && status === Image.Ready
                        source: modelData.iconSource
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                        cache: false
                        sourceSize.width: 72
                        sourceSize.height: 72
                        layer.enabled: visible
                        layer.smooth: true
                        layer.effect: OpacityMask {
                            maskSource: Rectangle {
                                width: iconBox.width
                                height: iconBox.height
                                radius: iconBox.radius
                            }
                        }
                    }
                    Text {
                        anchors.centerIn: parent
                        visible: !iconImg.visible
                        text: modelData.name.charAt(0).toUpperCase()
                        color: theme.accent
                        font.pixelSize: 16
                        font.weight: Font.DemiBold
                    }
                }

                Column {
                    anchors.left: iconBox.right
                    anchors.leftMargin: theme.space.md
                    anchors.right: meta.left
                    anchors.rightMargin: theme.space.md
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2

                    Row {
                        spacing: theme.space.xs
                        Text {
                            text: modelData.name
                            color: theme.text
                            font.pixelSize: 14
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight
                        }
                        SvgIcon {
                            visible: modelData.hasShell === true
                            name: "warning"
                            size: 14
                            color: theme.warning
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        SvgIcon {
                            visible: modelData.remote === true
                            name: "download"
                            size: 14
                            color: theme.textMuted
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    Text {
                        width: parent.width
                        text: modelData.description
                        visible: modelData.description !== ""
                        color: theme.textMuted
                        font.pixelSize: 12
                        elide: Text.ElideRight
                    }
                }

                Column {
                    id: meta
                    anchors.right: deleteBtn.left
                    anchors.rightMargin: theme.space.sm
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2

                    Text {
                        anchors.right: parent.right
                        text: modelData.author
                        color: theme.textMuted
                        font.pixelSize: 12
                    }
                    Text {
                        anchors.right: parent.right
                        text: modelData.modified
                        color: theme.textSubtle
                        font.pixelSize: 11
                    }
                }

                MouseArea {
                    id: rowArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: {
                        if (modelData.remote) {
                            if (root.installingRemote) return
                            root.errorText = ""
                            root.installingRemote = true
                            root.scriptsBridge.installRemote(JSON.stringify(modelData.raw))
                        } else {
                            root.scriptChosen(modelData.toml)
                            root.close()
                        }
                    }
                }

                IconButton {
                    id: deleteBtn
                    icon: "close"
                    size: 24
                    danger: true
                    z: 2
                    anchors.right: parent.right
                    anchors.rightMargin: theme.space.sm
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !modelData.remote
                    opacity: rowArea.containsMouse || hovered ? 1 : 0
                    Behavior on opacity { NumberAnimation { duration: theme.dur.fast } }
                    onClicked: {
                        if (root.scriptsBridge.removeScript(modelData.dir))
                            root.entries = JSON.parse(root.scriptsBridge.listJson())
                        else
                            root.errorText = qsTr("Couldn't remove the script.")
                    }
                }
            }
        }

        Text {
            anchors.centerIn: parent
            visible: root.filtered.length === 0
            text: root.searchText.trim() === ""
                ? qsTr("No scripts installed yet.\nSearch for community scripts, or use a local file.")
                : qsTr("No scripts match your search.")
            horizontalAlignment: Text.AlignHCenter
            color: theme.textMuted
            font.pixelSize: 13
        }
    }

    footerLeft: M3Button {
        text: qsTr("Use local")
        variant: "tonal"
        onClicked: root.openFilePicker()
    }

    actions: M3Button {
        text: qsTr("Close")
        variant: "tonal"
        onClicked: root.close()
    }
}
