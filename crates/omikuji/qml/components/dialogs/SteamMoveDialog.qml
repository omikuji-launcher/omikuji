import QtQuick
import "../controls"
import "../primitives"

DialogCard {
    id: root

    property var archiveManager: null
    property string sourceName: ""
    property string version: ""
    property var roots: []
    property var checkedPaths: ({})
    property string errorText: ""
    property bool working: false

    maxWidth: 440
    title: qsTr("Move to Steam")

    function show(src, ver) {
        sourceName = src
        version = ver
        errorText = ""
        working = false
        try { roots = JSON.parse(archiveManager.listSteamRoots()) } catch (e) { roots = [] }
        var c = {}
        for (var i = 0; i < roots.length; i++) c[roots[i][1]] = (i === 0)
        checkedPaths = c
        open()
    }

    function selectedPaths() {
        var sel = []
        for (var k in checkedPaths) if (checkedPaths[k] === true) sel.push(k)
        return sel
    }

    onCloseRequested: if (!working) close()

    Connections {
        target: root.archiveManager
        function onMoveToSteamDone(tag, error) {
            if (tag !== root.version) return
            root.working = false
            if (error && error.length > 0) root.errorText = error
            else root.close()
        }
    }

    body: Column {
        width: parent.width
        spacing: theme.space.md

        Text {
            width: parent.width
            text: qsTr("Moves %1 from omikuji's runners folder into the selected Steam installations.").arg(root.version)
            color: theme.textSubtle
            font.pixelSize: theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        Repeater {
            model: root.roots

            Item {
                id: rootRow
                required property var modelData
                readonly property bool selected: root.checkedPaths[modelData[1]] === true

                width: parent.width
                height: 40

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width
                    spacing: theme.space.md

                    SvgIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: rootRow.selected ? "check_box" : "check_box_outline_blank"
                        size: 20
                        color: rootRow.selected ? theme.accent : theme.alpha(theme.text, 0.55)
                    }

                    Column {
                        width: parent.width - 20 - theme.space.md
                        spacing: 1

                        Text {
                            width: parent.width
                            text: rootRow.modelData[0]
                            color: theme.textSubtle
                            font.pixelSize: theme.type.caption.size
                        }

                        Text {
                            width: parent.width
                            text: rootRow.modelData[1]
                            color: theme.text
                            font.pixelSize: theme.type.body.size
                            font.family: "monospace"
                            elide: Text.ElideMiddle
                        }
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        var c = root.checkedPaths
                        c[rootRow.modelData[1]] = !(c[rootRow.modelData[1]] === true)
                        root.checkedPaths = c
                    }
                }
            }
        }

        Text {
            visible: root.roots.length === 0
            width: parent.width
            text: qsTr("No Steam installation found.")
            color: theme.textSubtle
            font.pixelSize: theme.type.caption.size
        }

        Text {
            visible: root.roots.length > 0
            width: parent.width
            text: qsTr("Steam lists new compatibility tools after a restart.")
            color: theme.textSubtle
            font.pixelSize: theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        Text {
            visible: root.errorText !== ""
            width: parent.width
            text: root.errorText
            color: theme.error
            font.pixelSize: theme.type.caption.size
            wrapMode: Text.WordWrap
        }
    }

    actions: Row {
        spacing: theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "tonal"
            enabled: !root.working
            onClicked: root.close()
        }
        M3Button {
            text: root.working ? qsTr("Moving…") : qsTr("Move")
            variant: "filled"
            enabled: !root.working && root.selectedPaths().length > 0
            onClicked: {
                root.working = true
                root.errorText = ""
                root.archiveManager.moveToSteam(root.sourceName, root.version, JSON.stringify(root.selectedPaths()))
            }
        }
    }
}
