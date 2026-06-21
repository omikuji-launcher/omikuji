import QtQuick
import QtQuick.Layouts

import "."
import "../widgets"

Item {
    id: root

    // hand-maintained, bind from a build-info bridge invokable if a release pipeline ever lands (TODO: do ts)
    readonly property string appVersion: "0.3.0"
    readonly property string repoUrl: "https://github.com/reakjra/omikuji"
    readonly property string assetsRepoUrl: "https://github.com/reakjra/omikuji-assets"

    implicitHeight: content.height

    Column {
        id: content
        width: parent.width
        spacing: 20

        SettingsSection {
            label: "omikuji"
            width: parent.width

            Column {
                width: parent.width
                spacing: 6

                Text {
                    text: "A Qt/QML based wine apps launcher for Linux."
                    color: theme.text
                    font.pixelSize: 15
                }
                Text {
                    text: "Version " + root.appVersion
                    color: theme.textMuted
                    font.pixelSize: 13
                    font.family: "monospace"
                }
            }
        }

        SettingsSection {
            label: "License"
            width: parent.width

            Text {
                text: "GPL-3.0-or-later. omikuji is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version."
                color: theme.textSubtle
                font.pixelSize: 12
                width: parent.width
                wrapMode: Text.WordWrap
                lineHeight: 1.4
            }
        }

        SettingsSection {
            label: "Links"
            width: parent.width

            Column {
                width: parent.width
                spacing: 10

                Row {
                    width: parent.width
                    spacing: 12
                    Text {
                        text: "Source"
                        color: theme.textMuted
                        font.pixelSize: 13
                        width: 80
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: "<a href='" + root.repoUrl + "' style='color:" + theme.accent + "'>" + root.repoUrl + "</a>"
                        color: theme.accent
                        font.pixelSize: 13
                        font.family: "monospace"
                        textFormat: Text.RichText
                        onLinkActivated: (link) => Qt.openUrlExternally(link)
                        anchors.verticalCenter: parent.verticalCenter
                        HoverHandler { cursorShape: Qt.PointingHandCursor }
                    }
                }

                Row {
                    width: parent.width
                    spacing: 12
                    Text {
                        text: "Assets"
                        color: theme.textMuted
                        font.pixelSize: 13
                        width: 80
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: "<a href='" + root.assetsRepoUrl + "' style='color:" + theme.accent + "'>" + root.assetsRepoUrl + "</a>"
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
}
