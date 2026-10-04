pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

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
        { icon: "settings",        label: "Winecfg",                hint: qsTr("Wine configuration"),                      act: "winecfg" },
        { icon: "download",        label: "Winetricks",             hint: qsTr("Install Windows components"),              act: "winetricks" },
        { icon: "terminal",        label: qsTr("Run wine command"), hint: qsTr("Any command inside the prefix"),           act: "run_command" },
        { icon: "desktop_windows", label: qsTr("Console"),          hint: qsTr("Opens wineconsole"),                       act: "cmd" },
        { icon: "folder",          label: qsTr("Open folder"),      hint: qsTr("Browse the prefix files"),                 act: "open" },
        { icon: "skull",           label: qsTr("Kill wineserver"),  hint: qsTr("Stops everything running in this prefix"), act: "kill", danger: true }
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

                PathText {
                    id: pathText
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.leftMargin: Theme.space.md
                    anchors.rightMargin: Theme.space.md
                    text: root.prefix.path || ""
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

        CardGroup {
            width: parent.width

            Repeater {
                model: root.tools

                delegate: TileRow {
                    id: toolRow
                    required property var modelData
                    width: parent.width
                    tileSize: 32
                    icon: modelData.icon
                    title: modelData.label
                    subtitle: modelData.hint
                    danger: modelData.danger === true
                    onActivated: root.invokeTool(modelData.act)

                    IconTile {
                        width: 28
                        height: 28
                        iconSize: 20
                        icon: "chevron_left"
                        rotation: 180
                        tint: toolRow.tint
                    }
                }
            }
        }

        DialogSection {
            width: parent.width
            label: qsTr("Used by")

            Item {
                width: parent.width
                height: 28
                visible: root.games.length > 0

                ListView {
                    id: gameList
                    anchors.fill: parent
                    orientation: ListView.Horizontal
                    spacing: Theme.space.xs
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    keyNavigationEnabled: false
                    currentIndex: -1
                    model: root.games

                    readonly property bool navigable: contentWidth > width
                    readonly property real navRingRadius: Theme.radius.sm

                    Keys.onPressed: (event) => {
                        const dir = event.key === Qt.Key_Right ? 1 : (event.key === Qt.Key_Left ? -1 : 0)
                        const end = Math.max(0, contentWidth - width)
                        const target = Math.max(0, Math.min(end, contentX + dir * width * 0.6))
                        event.accepted = target !== contentX
                        if (!event.accepted) return
                        scrollAnim.to = target
                        scrollAnim.restart()
                    }

                    delegate: Squircle {
                        id: gameTag
                        required property var modelData
                        width: gameName.implicitWidth + Theme.space.md * 2
                        height: 28
                        radius: Theme.radius.sm
                        fillColor: Theme.alpha(Theme.text, 0.06)

                        Text {
                            id: gameName
                            anchors.centerIn: parent
                            text: gameTag.modelData
                            color: Theme.text
                            font.pixelSize: Theme.type.caption.size
                        }
                    }
                }

                NumberAnimation {
                    id: scrollAnim
                    target: gameList
                    property: "contentX"
                    duration: Theme.dur.med
                    easing.type: Theme.ease.standard
                }

                ScrollEdgeFade {
                    anchors.fill: parent
                    flickable: gameList
                    orientation: Qt.Horizontal
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
