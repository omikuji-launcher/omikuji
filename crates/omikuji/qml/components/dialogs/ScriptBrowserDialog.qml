pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Controls
import Qt5Compat.GraphicalEffects
import "../controls"
import "../primitives"

DialogCard {
    sizeKey: "script_browser"
    id: root

    property var scriptsBridge: null

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

    FilePicker {
        id: scriptPicker
        title: qsTr("Select script")
        filter: "*.toml"
        startFolder: "/home"
        onPicked: (path) => {
            root.scriptChosen(path)
            root.close()
        }
    }

    function openFilePicker() {
        scriptPicker.open()
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
            anchors.topMargin: Theme.space.xs
            anchors.left: parent.left
            anchors.right: parent.right
            visible: root.errorText !== ""
            text: root.errorText
            color: Theme.error
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        ListView {
            id: scriptList
            anchors.top: root.errorText !== "" ? errorLabel.bottom : searchField.bottom
            anchors.topMargin: Theme.space.md
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: root.filtered
            spacing: Theme.space.xs
            ScrollBar.vertical: ThinScrollBar {}

            delegate: Rectangle {
                id: scriptCard
                required property var modelData

                width: ListView.view.width
                height: 56
                radius: Theme.radius.md
                color: rowArea.containsMouse ? Theme.alpha(Theme.text, 0.08) : "transparent"

                Rectangle {
                    id: iconBox
                    width: 36
                    height: 36
                    radius: Theme.radius.sm
                    color: iconImg.visible ? "transparent" : Theme.alpha(Theme.accent, 0.15)
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.space.sm
                    anchors.verticalCenter: parent.verticalCenter

                    Image {
                        id: iconImg
                        anchors.fill: parent
                        visible: scriptCard.modelData.iconSource !== "" && status === Image.Ready
                        source: scriptCard.modelData.iconSource
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
                        text: scriptCard.modelData.name.charAt(0).toUpperCase()
                        color: Theme.accent
                        font.pixelSize: Theme.type.title.size
                        font.weight: Font.DemiBold
                    }
                }

                Column {
                    anchors.left: iconBox.right
                    anchors.leftMargin: Theme.space.md
                    anchors.right: meta.left
                    anchors.rightMargin: Theme.space.md
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2

                    Row {
                        spacing: Theme.space.xs
                        Text {
                            text: scriptCard.modelData.name
                            color: Theme.text
                            font.pixelSize: Theme.type.body.size
                            font.weight: Font.DemiBold
                            elide: Text.ElideRight
                        }
                        SvgIcon {
                            visible: scriptCard.modelData.hasShell === true
                            name: "warning"
                            size: 14
                            color: Theme.warning
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        SvgIcon {
                            visible: scriptCard.modelData.remote === true
                            name: "download"
                            size: 14
                            color: Theme.textMuted
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                    Text {
                        width: parent.width
                        text: scriptCard.modelData.description
                        visible: scriptCard.modelData.description !== ""
                        color: Theme.textMuted
                        font.pixelSize: Theme.type.caption.size
                        elide: Text.ElideRight
                    }
                }

                Column {
                    id: meta
                    anchors.right: deleteBtn.left
                    anchors.rightMargin: Theme.space.sm
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2

                    Text {
                        anchors.right: parent.right
                        text: scriptCard.modelData.author
                        color: Theme.textMuted
                        font.pixelSize: Theme.type.caption.size
                    }
                    Text {
                        anchors.right: parent.right
                        text: scriptCard.modelData.modified
                        color: Theme.textSubtle
                        font.pixelSize: Theme.type.micro.size
                    }
                }

                MouseArea {
                    id: rowArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: {
                        if (scriptCard.modelData.remote) {
                            if (root.installingRemote) return
                            root.errorText = ""
                            root.installingRemote = true
                            root.scriptsBridge.installRemote(JSON.stringify(scriptCard.modelData.raw))
                        } else {
                            root.scriptChosen(scriptCard.modelData.toml)
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
                    anchors.rightMargin: Theme.space.sm
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !scriptCard.modelData.remote
                    opacity: rowArea.containsMouse || hovered ? 1 : 0
                    Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
                    onClicked: {
                        if (root.scriptsBridge.removeScript(scriptCard.modelData.dir))
                            root.entries = JSON.parse(root.scriptsBridge.listJson())
                        else
                            root.errorText = qsTr("Couldn't remove the script.")
                    }
                }
            }
        }

        ScrollEdgeFade {
            anchors.fill: scriptList
            flickable: scriptList
        }

        Text {
            anchors.centerIn: parent
            visible: root.filtered.length === 0
            text: root.searchText.trim() === ""
                ? qsTr("No scripts installed yet.\nSearch for community scripts, or use a local file.")
                : qsTr("No scripts match your search.")
            horizontalAlignment: Text.AlignHCenter
            color: Theme.textMuted
            font.pixelSize: Theme.type.label.size
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
