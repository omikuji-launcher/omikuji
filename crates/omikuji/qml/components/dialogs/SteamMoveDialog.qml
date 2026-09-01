pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"

DialogCard {
    id: root

    property var archiveManager: null
    property string runnerDir: ""
    property string runnerName: ""
    property string mode: "move"
    property var roots: []
    property var checkedPaths: ({})
    property bool working: false

    readonly property bool linking: mode === "link"

    signal linksApplied(string name, string error)

    maxWidth: 440
    title: root.linking ? qsTr("Link to Steam") : qsTr("Move to Steam")

    function show(dir, name, action) {
        runnerDir = dir
        runnerName = name
        mode = action || "move"
        root.errorText = ""
        working = false
        try { roots = JSON.parse(archiveManager.listSteamRoots()) } catch (e) { roots = [] }

        var linked = []
        if (linking) {
            try { linked = JSON.parse(archiveManager.steamLinkedRoots(dir)) } catch (e) { linked = [] }
        }
        var c = {}
        for (var i = 0; i < roots.length; i++)
            c[roots[i][1]] = linking && linked.indexOf(roots[i][1]) >= 0
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
        enabled: root.archiveManager !== null && root.working
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
            visible: !root.linking
            width: parent.width
            text: qsTr("Moves %1 from omikuji's runners folder into the selected Steam installations.").arg(root.runnerName)
            color: Theme.textSubtle
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        NoteChip {
            visible: root.linking
            width: parent.width
            icon: "info"
            text: qsTr("%1 stays in omikuji and keeps auto-updating; Steam follows the symlink. Unchecking one removes its link.").arg(root.runnerName)
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
            text: root.linking
                ? qsTr("Apply")
                : (root.working ? qsTr("Moving…") : qsTr("Move"))
            variant: "filled"
            enabled: !root.working && (root.linking || root.selectedPaths().length > 0)
            onClicked: {
                root.errorText = ""
                const paths = JSON.stringify(root.selectedPaths())
                if (root.linking) {
                    const err = root.archiveManager.setSteamLinks(root.runnerDir, paths)
                    if (err !== "") root.errorText = err
                    else root.close()
                    root.linksApplied(root.runnerName, err)
                    return
                }
                root.working = true
                root.archiveManager.moveToSteamAt(root.runnerDir, paths)
            }
        }
    }
}
