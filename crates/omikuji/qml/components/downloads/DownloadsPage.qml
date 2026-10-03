pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts

Item {
    id: root

    property var downloadModel: null

    // paused when off-screen so the wave bar doesnt run the scene graph hot at 60fps behind a hidden panel (thanks for having me to do this manually. fabolous.)
    property bool pageVisible: true

    // bubbled to main so the confirm dialog dims the whole window not just this pane
    signal cancelRequested(string id, string displayName)
    signal pauseRequested(string id, string displayName, string atRisk)

    component SectionHeader: CapsLabel {
        color: Theme.textMuted
        size: 11
    }

    Item {
        anchors.fill: parent
        anchors.margins: 24
        visible: !root.downloadModel || root.downloadModel.count === 0

        ColumnLayout {
            anchors.centerIn: parent
            spacing: 12

            SvgIcon {
                Layout.alignment: Qt.AlignHCenter
                name: "download"
                size: 48
                color: Theme.textFaint
            }
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: qsTr("No active downloads")
                color: Theme.textMuted
                font.pixelSize: Theme.type.title.size
                font.weight: Font.Medium
            }
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: qsTr("Install a game from one of the connected stores to see it here.")
                color: Theme.textFaint
                font.pixelSize: Theme.type.label.size
            }
        }
    }

    Flickable {
        id: listFlick
        anchors.fill: parent
        anchors.margins: 24
        clip: true
        contentHeight: listCol.implicitHeight
        boundsBehavior: Flickable.StopAtBounds
        flickDeceleration: 3000
        visible: root.downloadModel && root.downloadModel.count > 0

        ColumnLayout {
            id: listCol
            width: parent.width
            spacing: 10

            SectionHeader {
                text: root.downloadModel && root.downloadModel.runningCount > 0 ? qsTr("Now downloading") : qsTr("Paused")
                visible: root.downloadModel && root.downloadModel.heroId !== ""
            }

            Repeater {
                model: root.downloadModel
                delegate: HeroCard {
                    id: heroItem
                    Layout.fillWidth: true
                    downloadModel: root.downloadModel
                    pageVisible: root.pageVisible
                    visible: root.downloadModel && heroItem.id === root.downloadModel.heroId
                    onCancelRequested: (id, displayName) => root.cancelRequested(id, displayName)
                    onPauseRequested: (id, displayName, atRisk) => root.pauseRequested(id, displayName, atRisk)
                }
            }

            SectionHeader {
                text: qsTr("Up next") + "  ·  " + (root.downloadModel ? root.downloadModel.queuedCount : 0)
                visible: root.downloadModel && root.downloadModel.queuedCount > 0
                Layout.topMargin: Theme.space.md
            }

            Repeater {
                model: root.downloadModel
                delegate: MiniRow {
                    id: upNextItem
                    Layout.fillWidth: true
                    downloadModel: root.downloadModel
                    visible: (status === "Queued" || status === "Paused")
                        && root.downloadModel && upNextItem.id !== root.downloadModel.heroId
                    onCancelRequested: (id, displayName) => root.cancelRequested(id, displayName)
                }
            }

            SectionHeader {
                text: qsTr("Failed") + "  ·  " + (root.downloadModel ? root.downloadModel.failedCount : 0)
                color: Theme.error
                visible: root.downloadModel && root.downloadModel.failedCount > 0
                Layout.topMargin: Theme.space.md
            }

            Repeater {
                model: root.downloadModel
                delegate: MiniRow {
                    Layout.fillWidth: true
                    downloadModel: root.downloadModel
                    visible: status === "Failed"
                    onCancelRequested: (id, displayName) => root.cancelRequested(id, displayName)
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: Theme.space.md
                visible: root.downloadModel && root.downloadModel.completedCount > 0

                SectionHeader {
                    Layout.fillWidth: true
                    text: qsTr("Completed") + "  ·  " + (root.downloadModel ? root.downloadModel.completedCount : 0)
                }

                M3Button {
                    small: true
                    variant: "tonal"
                    text: qsTr("Clear")
                    onClicked: root.downloadModel.clear_completed()
                }
            }

            Repeater {
                model: root.downloadModel
                delegate: MiniRow {
                    Layout.fillWidth: true
                    downloadModel: root.downloadModel
                    visible: status === "Completed" || status === "Cancelled"
                    onCancelRequested: (id, displayName) => root.cancelRequested(id, displayName)
                }
            }
        }
    }
}
