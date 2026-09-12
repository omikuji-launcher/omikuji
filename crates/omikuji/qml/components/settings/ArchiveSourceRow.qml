import QtQuick
import omikuji 1.0

import "../lib/RunnerGrouping.js" as RG
import "../controls"
import "../primitives"

Item {
    id: root

    property string sourceName: ""
    property string sourceKind: ""
    property int    installedCount: 0

    readonly property var kindLabels: ({
        proton: "Proton",
        wine: "Wine",
        dxvk: "DXVK",
        vkd3d: "VKD3D",
        dxvk_nvapi: "DXVK-NVAPI"
    })

    property bool   showDefaultVersion: false
    property var    installedVersions: []
    property string activeVersion: ""

    signal manageClicked()
    signal defaultVersionSelected(string tag)

    height: showDefaultVersion ? 100 : 56

    Squircle {
        anchors.fill: parent
        radius: Theme.radius.md
        fillColor: Theme.cardBg
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
                        color: Theme.text
                        font.pixelSize: Theme.type.body.size
                        font.weight: Font.DemiBold
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Row {
                        spacing: Theme.space.xs
                        anchors.verticalCenter: parent.verticalCenter

                        Rectangle {
                            width: 4
                            height: 4
                            radius: width / 2
                            color: Theme.accent
                            anchors.verticalCenter: parent.verticalCenter
                        }

                        Text {
                            text: root.kindLabels[root.sourceKind] || root.sourceKind
                            color: Theme.textMuted
                            font.pixelSize: Theme.type.label.size
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                }

                Text {
                    text: root.installedCount === 0
                        ? qsTr("No versions installed")
                        : qsTr("%n version(s) installed", "", root.installedCount)
                    color: root.installedCount > 0 ? Theme.success : Theme.textSubtle
                    font.pixelSize: Theme.type.caption.size
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
        visible: root.showDefaultVersion
        anchors.left: parent.left
        anchors.leftMargin: 14
        anchors.right: parent.right
        anchors.rightMargin: 14
        anchors.top: topRow.bottom
        height: 1
        color: Theme.separator
    }

    Item {
        id: defaultVersionRow
        visible: root.showDefaultVersion
        anchors.top: topRow.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        Text {
            id: defaultVersionLabel
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Default version")
            color: Theme.text
            font.pixelSize: Theme.type.label.size
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
                if (value !== root.activeVersion) root.defaultVersionSelected(value)
            }

            TextMetrics {
                id: labelMetrics
                font.pixelSize: Theme.type.body.size
                text: root.activeVersion === "" ? qsTr("Disabled") : root.activeVersion
            }
        }
    }

}
