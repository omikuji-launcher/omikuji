import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "."
import "../controls"

Item {
    id: root

    property var gameModel: null

    readonly property string appVersion: gameModel ? gameModel.app_version() : ""
    readonly property string repoUrl: "https://github.com/reakjra/omikuji"
    readonly property string assetsRepoUrl: "https://github.com/reakjra/omikuji-assets"
    readonly property string docsUrl: "https://reakjra.github.io/omikuji"

    implicitHeight: content.height

    function _loadSystemInfo() {
        if (gameModel) sysText.text = gameModel.system_info()
    }
    onGameModelChanged: _loadSystemInfo()
    Component.onCompleted: _loadSystemInfo()

    Column {
        id: content
        width: parent.width
        spacing: theme.space.xxl

        SettingsSection {
            label: "omikuji"
            width: parent.width

            Column {
                width: parent.width
                spacing: 6

                Text {
                    text: qsTr("A Qt/QML based wine apps launcher for Linux.")
                    color: theme.text
                    font.pixelSize: 15
                }
                Text {
                    text: qsTr("Version %1").arg(root.appVersion)
                    color: theme.textMuted
                    font.pixelSize: 13
                    font.family: "monospace"
                }
            }
        }

        SettingsSection {
            label: qsTr("License")
            width: parent.width

            Text {
                text: qsTr("GPL-3.0-or-later. omikuji is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.")
                color: theme.textSubtle
                font.pixelSize: 12
                width: parent.width
                wrapMode: Text.WordWrap
                lineHeight: 1.4
            }
        }

        SettingsSection {
            label: qsTr("Links")
            width: parent.width

            Column {
                width: parent.width
                spacing: 10

                Repeater {
                    model: [
                        { label: qsTr("Source"), url: root.repoUrl },
                        { label: qsTr("Assets"), url: root.assetsRepoUrl },
                        { label: qsTr("Docs"), url: root.docsUrl }
                    ]

                    Row {
                        required property var modelData
                        width: parent.width
                        spacing: 12
                        Text {
                            text: modelData.label
                            color: theme.textMuted
                            font.pixelSize: 13
                            width: 80
                            anchors.verticalCenter: parent.verticalCenter
                        }
                        Text {
                            text: "<a href='" + modelData.url + "' style='color:" + theme.accent + "'>" + modelData.url + "</a>"
                            color: theme.accent
                            font.pixelSize: 13
                            font.family: "monospace"
                            textFormat: Text.RichText
                            onLinkActivated: (link) => Qt.openUrlExternally(link)
                            anchors.verticalCenter: parent.verticalCenter
                            HoverHandler { cursorShape: Qt.PointingHandCursor }
                        }
                    }
                }
            }
        }

        SettingsSection {
            label: qsTr("System")
            width: parent.width

            Column {
                width: parent.width
                spacing: 10

                TextArea {
                    id: sysText
                    width: parent.width
                    readOnly: true
                    wrapMode: TextArea.Wrap
                    selectByMouse: true
                    color: theme.text
                    font.family: "monospace"
                    font.pixelSize: 13
                    leftPadding: 12
                    rightPadding: 12
                    topPadding: 10
                    bottomPadding: 10
                    text: ""

                    background: FieldSurface {}
                }

                M3Button {
                    text: qsTr("Copy")
                    variant: "tonal"
                    onClicked: {
                        sysText.selectAll()
                        sysText.copy()
                        sysText.deselect()
                    }
                }
            }
        }
    }
}
