pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"

DialogCard {
    id: root

    property var archiveManager: null
    property string runnerName: ""
    property string runnerKind: ""
    property string runnerOrigin: ""
    property string runnerDir: ""
    property var dllOptions: ({})
    property string errorText: ""

    readonly property bool canMoveToSteam: runnerOrigin === "Omikuji" && runnerKind === "proton"

    signal moveToSteamRequested(string dir, string name)

    maxWidth: 660
    title: ""

    function show(name, kind, dir, origin) {
        runnerName = name
        runnerKind = kind
        runnerOrigin = origin || ""
        errorText = ""
        try { dllOptions = JSON.parse(archiveManager.runnerDllOptions()) } catch (e) { dllOptions = ({}) }
        runnerDir = ""
        runnerDir = dir
        open()
    }

    onCloseRequested: close()

    body: Item {
        width: parent.width
        implicitHeight: bodyHeader.height + bodyDivider.height + Theme.space.lg + contentCol.implicitHeight

        Item {
            id: bodyHeader
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 64

            Column {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2

                Row {
                    spacing: 10
                    Text {
                        text: root.runnerName
                        color: Theme.text
                        font.pixelSize: Theme.type.headline.size
                        font.weight: Font.DemiBold
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Rectangle {
                        height: 18
                        width: kindText.width + 14
                        radius: 9
                        color: Theme.alpha(Theme.accent, 0.15)
                        anchors.verticalCenter: parent.verticalCenter
                        Text {
                            id: kindText
                            anchors.centerIn: parent
                            text: root.runnerKind
                            color: Theme.accent
                            font.pixelSize: Theme.type.micro.size
                            font.weight: Font.Medium
                            font.capitalization: Font.AllUppercase
                            font.letterSpacing: 0.6
                        }
                    }
                }

                Text {
                    width: parent.width
                    text: root.runnerDir
                    color: Theme.accent
                    font.pixelSize: Theme.type.caption.size
                    font.family: "monospace"
                    elide: Text.ElideMiddle
                }
            }
        }

        Rectangle {
            id: bodyDivider
            anchors.top: bodyHeader.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: 1
            color: Theme.separator
        }

        Column {
            id: contentCol
            anchors.top: bodyDivider.bottom
            anchors.topMargin: Theme.space.lg
            anchors.left: parent.left
            anchors.right: parent.right
            spacing: Theme.space.sm

            Text {
                visible: root.runnerKind === "proton"
                width: parent.width
                text: qsTr("Replace this Proton's bundled translation layers with versions installed in omikuji. Default restores what Proton ships.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }

            RunnerDllOverride {
                width: parent.width
                archiveManager: root.archiveManager
                runnerDir: root.runnerDir
                options: root.dllOptions
                onErrorRaised: (m) => root.errorText = m
            }

            Text {
                visible: root.errorText !== ""
                width: parent.width
                text: root.errorText
                color: Theme.error
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }
        }
    }

    footerLeft: M3Button {
        text: qsTr("Move to Steam")
        variant: "tonal"
        visible: root.canMoveToSteam
        onClicked: {
            root.moveToSteamRequested(root.runnerDir, root.runnerName)
            root.close()
        }
    }

    actions: Row {
        M3Button {
            text: qsTr("Close")
            variant: "tonal"
            onClicked: root.close()
        }
    }
}
