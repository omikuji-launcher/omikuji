import QtQuick

import "../lib/RunnerGrouping.js" as RG
import "../controls"
import "../primitives"

Item {
    id: root

    property string sourceName: ""
    property string sourceKind: ""
    property int    installedCount: 0

    property bool   showAutoInject: false
    property var    installedVersions: []
    property string activeVersion: ""

    signal manageClicked()
    signal autoInjectChanged(string tag)

    height: showAutoInject ? 100 : 56

    Squircle {
        anchors.fill: parent
        radius: theme.radius.md
        fillColor: theme.cardBg
    }

    Item {
        id: topRow
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 56

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.right: manageBtn.left
            anchors.rightMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            spacing: 12

            Column {
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2

                Row {
                    spacing: 8
                    Text {
                        text: root.sourceName
                        color: theme.text
                        font.pixelSize: theme.type.body.size
                        font.weight: Font.DemiBold
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Rectangle {
                        height: 16
                        width: kindLabel.width + 12
                        radius: theme.radius.sm
                        color: theme.alpha(theme.accent, 0.13)
                        anchors.verticalCenter: parent.verticalCenter
                        Text {
                            id: kindLabel
                            anchors.centerIn: parent
                            text: root.sourceKind
                            color: theme.accent
                            font.pixelSize: theme.type.micro.size
                            font.weight: Font.Medium
                            font.capitalization: Font.AllUppercase
                            font.letterSpacing: 0.6
                        }
                    }
                }

                Text {
                    text: root.installedCount === 0
                        ? qsTr("No versions installed")
                        : root.installedCount === 1
                            ? qsTr("1 version installed")
                            : qsTr("%1 versions installed").arg(root.installedCount)
                    color: root.installedCount > 0 ? theme.success : theme.textSubtle
                    font.pixelSize: theme.type.caption.size
                }
            }
        }

        M3Button {
            id: manageBtn
            anchors.right: parent.right
            anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Manage")
            variant: "tonal"
            onClicked: root.manageClicked()
        }
    }

    Rectangle {
        visible: root.showAutoInject
        anchors.left: parent.left
        anchors.leftMargin: 14
        anchors.right: parent.right
        anchors.rightMargin: 14
        anchors.top: topRow.bottom
        height: 1
        color: theme.separator
    }

    Item {
        id: autoInjectRow
        visible: root.showAutoInject
        anchors.top: topRow.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        Text {
            id: autoInjectLabel
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Auto install on prefix")
            color: theme.text
            font.pixelSize: theme.type.label.size
        }

        M3Dropdown {
            anchors.right: parent.right
            anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(240, labelMetrics.width + 56)
            fieldHeight: 32
            options: {
                let opts = [{ label: qsTr("Disabled"), value: "" }]
                for (let i = 0; i < root.installedVersions.length; i++) {
                    let tag = root.installedVersions[i]
                    opts.push({ label: tag, value: tag })
                }
                return opts
            }
            currentIndex: {
                let idx = RG.indexOfValue(options, root.activeVersion)
                return idx >= 0 ? idx : 0
            }
            onSelected: (value) => {
                if (value !== root.activeVersion) root.autoInjectChanged(value)
            }

            TextMetrics {
                id: labelMetrics
                font.pixelSize: theme.type.body.size
                text: root.activeVersion === "" ? qsTr("Disabled") : root.activeVersion
            }
        }
    }

}
