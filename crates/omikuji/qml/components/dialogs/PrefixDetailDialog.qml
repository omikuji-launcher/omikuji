pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"
import "../primitives"


DialogCard {
    sizeKey: "prefix_detail"
    id: root

    property var ofudaBridge: null
    property var prefix: ({})

    readonly property var games: prefix.games || []
    readonly property bool isSteam: (prefix.kind || "") === "steam"

    signal deleteRequested(var prefix)
    signal runCommandRequested(var prefix)

    maxWidth: 540
    title: prefix.name || ""

    function show(p) {
        prefix = p
        open()
    }

    function runTool(t) {
        if (ofudaBridge) ofudaBridge.runTool(prefix.path || "", t, prefix.runner || "")
    }

    function invokeTool(act) {
        if (act === "open") {
            if (ofudaBridge) ofudaBridge.openFolder(prefix.path || "")
        } else if (act === "run_command") {
            runCommandRequested(prefix)
        } else {
            runTool(act)
        }
    }

    readonly property var tools: [
        { icon: "settings", label: "Winecfg",                  act: "winecfg" },
        { icon: "download", label: "Winetricks",               act: "winetricks" },
        { icon: "terminal", label: qsTr("Run wine command"),   act: "run_command" },
        { icon: "desktop_windows", label: qsTr("Console (wineconsole)"), act: "cmd" },
        { icon: "folder",   label: qsTr("Open folder"),        act: "open" },
        { icon: "close",    label: qsTr("Kill wineserver"),    act: "kill" }
    ]

    onCloseRequested: close()

    body: Column {
        spacing: Theme.space.lg
        width: parent.width

        Column {
            width: parent.width
            spacing: Theme.space.sm

            Squircle {
                width: parent.width
                height: pathText.implicitHeight + Theme.space.md
                radius: Theme.radius.sm
                fillColor: Theme.alpha(Theme.text, 0.06)

                Text {
                    id: pathText
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.leftMargin: Theme.space.md
                    anchors.rightMargin: Theme.space.md
                    text: root.prefix.path || ""
                    color: Theme.accent
                    font.pixelSize: Theme.type.caption.size
                    font.family: "monospace"
                    wrapMode: Text.WrapAnywhere
                }
            }

            Text {
                width: parent.width
                visible: root.isSteam
                text: qsTr("This is a Steam prefix. Steam owns its files, omikuji only runs tools inside it.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }
        }

        DialogSection {
            width: parent.width
            label: qsTr("Used by")
            contentSpacing: 3

            Repeater {
                model: root.games
                delegate: Text {
                    id: gameRow
                    required property var modelData
                    width: parent.width
                    text: modelData
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.label.size
                    elide: Text.ElideRight
                }
            }

            Text {
                width: parent.width
                visible: root.games.length === 0
                text: qsTr("Orphan prefix, no game uses it.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }
        }

        DialogSection {
            width: parent.width
            label: qsTr("Tools")

            Grid {
                width: parent.width
                columns: 2
                spacing: Theme.space.sm

                Repeater {
                    model: root.tools

                    delegate: Item {
                        id: toolCell
                        required property var modelData
                        width: (parent.width - Theme.space.sm) / 2
                        height: 46

                        Squircle {
                            anchors.fill: parent
                            radius: Theme.radius.md
                            fillColor: Theme.alpha(Theme.text, toolMouse.containsMouse ? 0.14 : 0.06)
                        }

                        Row {
                            anchors.left: parent.left
                            anchors.leftMargin: Theme.space.md
                            anchors.right: parent.right
                            anchors.rightMargin: Theme.space.sm
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: Theme.space.md

                            SvgIcon {
                                anchors.verticalCenter: parent.verticalCenter
                                name: toolCell.modelData.icon
                                size: 18
                                color: Theme.icon
                            }
                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: toolCell.modelData.label
                                color: Theme.text
                                font.pixelSize: Theme.type.label.size
                                elide: Text.ElideRight
                                width: Math.min(implicitWidth, parent.width - 18 - Theme.space.md)
                            }
                        }

                        MouseArea {
                            id: toolMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.invokeTool(toolCell.modelData.act)
                        }
                    }
                }
            }
        }
    }

    footerLeft: M3Button {
        visible: !root.isSteam
        text: qsTr("Delete prefix")
        variant: "tonal"
        danger: true
        onClicked: root.deleteRequested(root.prefix)
    }

    actions: M3Button {
        text: qsTr("Close")
        variant: "tonal"
        onClicked: root.close()
    }
}
