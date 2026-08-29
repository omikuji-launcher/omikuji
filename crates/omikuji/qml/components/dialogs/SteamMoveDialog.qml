pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"
import "../primitives"

DialogCard {
    id: root

    property var archiveManager: null
    property string runnerDir: ""
    property string runnerName: ""
    property var roots: []
    property var checkedPaths: ({})
    property bool working: false

    maxWidth: 440
    title: qsTr("Move to Steam")

    function show(dir, name) {
        runnerDir = dir
        runnerName = name
        root.errorText = ""
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
        function onMoveToSteamDone(name, error) {
            if (name !== root.runnerName) return
            root.working = false
            if (error && error.length > 0) root.errorText = error
            else root.close()
        }
    }

    body: Column {
        width: parent.width
        spacing: Theme.space.md

        Text {
            width: parent.width
            text: qsTr("Moves %1 from omikuji's runners folder into the selected Steam installations.").arg(root.runnerName)
            color: Theme.textSubtle
            font.pixelSize: Theme.type.caption.size
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
                    spacing: Theme.space.md

                    SvgIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: rootRow.selected ? "check_box" : "check_box_outline_blank"
                        size: 20
                        color: rootRow.selected ? Theme.accent : Theme.alpha(Theme.text, 0.55)
                    }

                    Column {
                        width: parent.width - 20 - Theme.space.md
                        spacing: 1

                        Text {
                            width: parent.width
                            text: rootRow.modelData[0]
                            color: Theme.textSubtle
                            font.pixelSize: Theme.type.caption.size
                        }

                        Text {
                            width: parent.width
                            text: rootRow.modelData[1]
                            color: Theme.text
                            font.pixelSize: Theme.type.body.size
                            font.family: "monospace"
                            elide: Text.ElideMiddle
                        }
                    }
                }

                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        var c = Object.assign({}, root.checkedPaths)
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
            color: Theme.textSubtle
            font.pixelSize: Theme.type.caption.size
        }

        Text {
            visible: root.roots.length > 0
            width: parent.width
            text: qsTr("Steam lists new compatibility tools after a restart.")
            color: Theme.textSubtle
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }

    }

    actions: Row {
        spacing: Theme.space.sm

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
                root.archiveManager.moveToSteamAt(root.runnerDir, JSON.stringify(root.selectedPaths()))
            }
        }
    }
}
