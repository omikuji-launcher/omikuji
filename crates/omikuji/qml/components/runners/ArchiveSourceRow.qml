import QtQuick
import omikuji 1.0

import "../lib/RunnerGrouping.js" as RG

Item {
    id: root

    property string sourceName: ""
    property string sourceKind: ""
    property int    installedCount: 0

    property bool   showPrefixInstall: false
    property var    installedVersions: []
    property string prefixInstallVersion: ""

    signal manageClicked()
    signal prefixInstallVersionSelected(string tag)

    height: showPrefixInstall ? 56 + versionRows.height : 56

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

                Text {
                    text: root.sourceName
                    color: Theme.text
                    font.pixelSize: Theme.type.body.size
                    font.weight: Font.DemiBold
                }

                Text {
                    text: root.installedCount === 0
                        ? qsTr("No versions installed")
                        : qsTr("%n version(s) installed", "", root.installedCount)
                    color: root.installedCount > 0 ? Theme.textMuted : Theme.textSubtle
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

    component VersionRow: Item {
        id: versionRow

        required property string label
        required property var versions
        required property string value
        signal picked(string tag)

        width: parent.width
        height: 44

        Text {
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            text: versionRow.label
            color: Theme.text
            font.pixelSize: Theme.type.label.size
        }

        M3Dropdown {
            anchors.right: parent.right
            anchors.rightMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            width: Math.min(240, valueMetrics.width + 56)
            fieldHeight: 32
            options: [{ label: qsTr("Disabled"), value: "" }].concat(
                versionRow.versions.map(tag => ({ label: tag, value: tag })))
            currentIndex: Math.max(0, RG.indexOfValue(options, versionRow.value))
            onSelected: (tag) => {
                if (tag !== versionRow.value) versionRow.picked(tag)
            }

            TextMetrics {
                id: valueMetrics
                font.pixelSize: Theme.type.body.size
                text: versionRow.value === "" ? qsTr("Disabled") : versionRow.value
            }
        }
    }

    Column {
        id: versionRows
        visible: root.showPrefixInstall
        anchors.top: topRow.bottom
        anchors.left: parent.left
        anchors.right: parent.right

        VersionRow {
            label: qsTr("Install into new prefixes")
            versions: root.installedVersions
            value: root.prefixInstallVersion
            onPicked: (tag) => root.prefixInstallVersionSelected(tag)
        }
    }
}
