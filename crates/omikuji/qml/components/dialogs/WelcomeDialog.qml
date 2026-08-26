pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import omikuji 1.0
import "../controls"
import "../primitives"
import "../settings"


DialogCard {
    id: root

    sizeKey: "welcome"

    property var appSettings: null
    property var componentsBridge: null
    property var archiveManager: null

    property var runners: []
    property bool installUmu: true

    signal manageRequested(string category, string source, string kind)
    signal umuInstallRequested()

    property var umuStatus: ({ status: "missing", path: "" })
    readonly property bool umuFromSystem: umuStatus.status === "system"
    readonly property bool umuPresent: umuStatus.status === "system" || umuStatus.status === "completed"

    maxWidth: 560
    scrollable: true
    title: qsTr("Welcome to omikuji~")

    function refresh() {
        if (archiveManager) {
            try { runners = JSON.parse(archiveManager.listRunners()) || [] }
            catch (e) { runners = [] }
        }
        if (componentsBridge) {
            try {
                umuStatus = JSON.parse(componentsBridge.statusJson())["umu-run"]
                    || ({ status: "missing", path: "" })
            } catch (e) {
                umuStatus = ({ status: "missing", path: "" })
            }
        }
    }

    function show() {
        refresh()
        open()
    }

    function finish() {
        if (appSettings) appSettings.applyWelcomeSeen(true)
        if (installUmu && !umuPresent && componentsBridge) {
            componentsBridge.installComponent("umu-run")
            umuInstallRequested()
        }
        close()
    }

    onCloseRequested: root.finish()

    body: ColumnLayout {
        width: parent.width
        spacing: Theme.space.lg

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space.md

            SvgIcon {
                Layout.preferredWidth: 40
                Layout.preferredHeight: 40
                Layout.alignment: Qt.AlignVCenter
                name: "favorite_fill"
                size: 40
                color: Theme.accent
            }

            Text {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter
                text: qsTr("Everything here is optional, you can close this at any time and set it all up later from Settings.")
                color: Theme.textMuted
                font.pixelSize: Theme.type.body.size
                wrapMode: Text.WordWrap
            }
        }

        DialogSection {
            Layout.fillWidth: true
            label: qsTr("Runners")

            Text {
                width: parent.width
                text: qsTr("Games run through a Wine or Proton build. Pick a source to fetch one now, or skip and do it later.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }

            Repeater {
                model: root.runners

                delegate: ArchiveSourceRow {
                    id: runnerRow
                    required property var modelData
                    width: parent.width
                    sourceName: modelData.name
                    sourceKind: modelData.kind
                    onManageClicked: root.manageRequested("runners", runnerRow.sourceName, runnerRow.sourceKind)
                }
            }

            Text {
                width: parent.width
                visible: root.runners.length === 0
                text: qsTr("No runner sources configured yet.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }
        }

        DialogSection {
            Layout.fillWidth: true
            label: qsTr("Proton support")

            Text {
                width: parent.width
                visible: root.umuPresent
                text: root.umuFromSystem
                    ? qsTr("umu-run is already provided by your system, omikuji will use it as is.")
                    : qsTr("umu-run is already installed, nothing to do here.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }

            Text {
                width: parent.width
                visible: root.umuPresent && text !== ""
                text: root.umuStatus.path || ""
                color: Theme.accent
                font.family: "monospace"
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WrapAnywhere
            }

            LabeledSwitch {
                width: parent.width
                visible: !root.umuPresent
                label: qsTr("Install umu-run")
                description: qsTr("Proton runners launch through it")
                checked: root.installUmu
                onToggled: (val) => root.installUmu = val
            }
        }
    }

    actions: M3Button {
        text: qsTr("Done")
        variant: "filled"
        onClicked: root.finish()
    }
}
