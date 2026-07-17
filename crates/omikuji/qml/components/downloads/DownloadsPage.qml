import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "."
import "../primitives"

Item {
    id: root

    property var downloadModel: null
    property var componentsBridge: null

    // paused when off-screen so the wave bar doesnt run the scene graph hot at 60fps behind a hidden panel (thanks for having me to do this manually. fabolous.)
    property bool pageVisible: true

    // bubbled to main so the confirm dialog dims the whole window not just this pane
    signal cancelRequested(string id, string displayName)

    // patched row-by-row so we dont reparse the full json on every progress tick
    property var componentStatuses: ({})
    readonly property var componentOrder: ["umu-run", "hpatchz", "legendary", "gogdl", "jadeite", "egl-dummy"]
    readonly property bool componentsVisible: {
        if (!componentsBridge) return false
        if (componentsBridge.inProgress) return true
        if (componentsBridge.pendingCount > 0) return true
        for (let k in componentStatuses) {
            if (componentStatuses[k] && componentStatuses[k].status === "failed") return true
        }
        return false
    }

    function syncComponentStatuses() {
        if (!componentsBridge) return
        try {
            componentStatuses = JSON.parse(componentsBridge.statusJson())
        } catch (e) {
            console.warn("[downloads] bad components statusJson:", e)
        }
    }

    Component.onCompleted: syncComponentStatuses()

    Connections {
        target: componentsBridge
        function onComponentStarted(name) { root.syncComponentStatuses() }
        function onComponentProgress(name, phase, percent) {
            let s = root.componentStatuses[name] || {}
            s.status = phase
            s.percent = percent
            let next = Object.assign({}, root.componentStatuses)
            next[name] = s
            root.componentStatuses = next
        }
        function onComponentCompleted(name, version) { root.syncComponentStatuses() }
        function onComponentFailed(name, error) { root.syncComponentStatuses() }
    }

    component SectionHeader: CapsLabel {
        color: theme.textMuted
        size: 11
    }

    Item {
        anchors.fill: parent
        anchors.margins: 24
        visible: (!downloadModel || downloadModel.count === 0) && !componentsVisible

        ColumnLayout {
            anchors.centerIn: parent
            spacing: 12

            SvgIcon {
                Layout.alignment: Qt.AlignHCenter
                name: "download"
                size: 48
                color: theme.textFaint
            }
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: qsTr("No active downloads")
                color: theme.textMuted
                font.pixelSize: theme.type.title.size
                font.weight: Font.Medium
            }
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: qsTr("Install a game from one of the connected stores to see it here.")
                color: theme.textFaint
                font.pixelSize: theme.type.label.size
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
        visible: (downloadModel && downloadModel.count > 0) || root.componentsVisible

        ColumnLayout {
            id: listCol
            width: parent.width
            spacing: 10

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 8
                visible: root.componentsVisible

                SectionHeader { text: qsTr("Runtime components") }

                Repeater {
                    model: root.componentOrder
                    delegate: ComponentRow {
                        required property string modelData
                        Layout.fillWidth: true
                        name: modelData
                        entry: root.componentStatuses[modelData] || ({})
                        onRetryRequested: {
                            if (root.componentsBridge) root.componentsBridge.installAll()
                        }
                    }
                }
            }

            SectionHeader {
                text: downloadModel && downloadModel.runningCount > 0 ? qsTr("Now downloading") : qsTr("Paused")
                visible: downloadModel && downloadModel.heroId !== ""
                Layout.topMargin: root.componentsVisible ? theme.space.md : 0
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
                }
            }

            SectionHeader {
                text: qsTr("Up next") + "  ·  " + (downloadModel ? downloadModel.queuedCount : 0)
                visible: downloadModel && downloadModel.queuedCount > 0
                Layout.topMargin: theme.space.md
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
                text: qsTr("Failed") + "  ·  " + (downloadModel ? downloadModel.failedCount : 0)
                color: theme.error
                visible: downloadModel && downloadModel.failedCount > 0
                Layout.topMargin: theme.space.md
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

            SectionHeader {
                text: qsTr("Completed") + "  ·  " + (downloadModel ? downloadModel.completedCount : 0)
                visible: downloadModel && downloadModel.completedCount > 0
                Layout.topMargin: theme.space.md
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
