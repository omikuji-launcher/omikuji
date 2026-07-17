import QtQuick
import "../controls"
import "../primitives"


DialogCard {
    sizeKey: "prefix_detail"
    id: root

    property var ofudaBridge: null
    property var prefix: ({})

    readonly property var games: prefix.games || []

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
        spacing: theme.space.md
        width: parent.width

        Squircle {
            width: parent.width
            height: pathText.implicitHeight + theme.space.md
            radius: theme.radius.sm
            fillColor: theme.alpha(theme.text, 0.06)

            Text {
                id: pathText
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.leftMargin: theme.space.md
                anchors.rightMargin: theme.space.md
                text: root.prefix.path || ""
                color: theme.accent
                font.pixelSize: theme.type.caption.size
                font.family: "monospace"
                wrapMode: Text.WrapAnywhere
            }
        }

        Column {
            width: parent.width
            spacing: 3
            visible: root.games.length > 0

            Text {
                text: qsTr("Used by")
                color: theme.textSubtle
                font.pixelSize: theme.type.micro.size
                font.weight: Font.DemiBold
            }
            Repeater {
                model: root.games
                delegate: Text {
                    required property var modelData
                    width: parent.width
                    text: modelData
                    color: theme.textMuted
                    font.pixelSize: theme.type.label.size
                    elide: Text.ElideRight
                }
            }
        }

        Text {
            width: parent.width
            visible: root.games.length === 0
            text: qsTr("Orphan prefix, no game uses it.")
            color: theme.textSubtle
            font.pixelSize: theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        Rectangle {
            width: parent.width
            height: 1
            color: theme.alpha(theme.text, 0.1)
        }

        Column {
            width: parent.width
            spacing: 2

            Repeater {
                model: root.tools

                delegate: Item {
                    required property var modelData
                    width: parent.width
                    height: 40

                    Squircle {
                        anchors.fill: parent
                        radius: theme.radius.sm
                        fillColor: toolMouse.containsMouse ? theme.stateHover : "transparent"
                    }

                    Row {
                        anchors.left: parent.left
                        anchors.leftMargin: 12
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 12

                        SvgIcon {
                            anchors.verticalCenter: parent.verticalCenter
                            name: modelData.icon
                            size: 18
                            color: theme.icon
                        }
                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: modelData.label
                            color: theme.text
                            font.pixelSize: theme.type.label.size
                        }
                    }

                    MouseArea {
                        id: toolMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.invokeTool(modelData.act)
                    }
                }
            }
        }
    }

    footerLeft: M3Button {
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
