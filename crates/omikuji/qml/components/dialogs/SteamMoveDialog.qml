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

                Rectangle {
                    anchors.fill: parent
                    radius: Theme.radius.sm
                    color: rowArea.containsMouse ? Theme.alpha(Theme.text, 0.06) : "transparent"
                    Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
                }

                Row {
                    anchors.left: parent.left
                    anchors.leftMargin: 10
                    anchors.right: parent.right
                    anchors.rightMargin: 10
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: Theme.space.md

                    M3Checkbox {
                        id: rootCheck
                        anchors.verticalCenter: parent.verticalCenter
                        checked: rootRow.selected
                    }

                    Column {
                        width: parent.width - rootCheck.width - Theme.space.md
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
                    id: rowArea
                    anchors.fill: parent
                    hoverEnabled: true
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
