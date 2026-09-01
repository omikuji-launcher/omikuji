pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import omikuji 1.0
import "../controls"
import "../primitives"


DialogCard {
    id: root

    sizeKey: "welcome"

    property var appSettings: null
    property var componentsBridge: null
    property var archiveManager: null

    property var runners: []
    property bool installUmu: true

    readonly property string recommendedRunner: {
        for (let i = 0; i < runners.length; i++) {
            if (runners[i].name.toLowerCase().indexOf("cachyos") >= 0) return runners[i].name
        }
        return runners.length > 0 ? runners[0].name : ""
    }

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
                text: qsTr("Games run through a Wine or Proton build, you'll need at least one.")
                color: Theme.textSubtle
                font.pixelSize: Theme.type.caption.size
                wrapMode: Text.WordWrap
            }

            Item {
                width: parent.width
                height: 40
                visible: root.recommendedRunner !== ""

                Text {
                    anchors.left: parent.left
                    anchors.right: openRunners.left
                    anchors.rightMargin: Theme.space.md
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Install a runner")
                    color: Theme.text
                    font.pixelSize: Theme.type.body.size
                    elide: Text.ElideRight
                }

                Item {
                    id: openRunners
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    width: 36
                    height: 36

                    Squircle {
                        anchors.fill: parent
                        radius: Theme.radius.md
                        fillColor: openHover.containsPress
                            ? Theme.alpha(Theme.accent, 0.28)
                            : openHover.containsMouse
                                ? Theme.alpha(Theme.accent, 0.20)
                                : Theme.alpha(Theme.accent, 0.13)

                        Behavior on fillColor { ColorAnimation { duration: Theme.dur.fast } }
                    }

                    SvgIcon {
                        anchors.centerIn: parent
                        name: "open_in_new"
                        size: 18
                        color: Theme.accent
                    }

                    MouseArea {
                        id: openHover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            for (let i = 0; i < root.runners.length; i++) {
                                if (root.runners[i].name === root.recommendedRunner) {
                                    root.manageRequested("runners", root.runners[i].name, root.runners[i].kind)
                                    return
                                }
                            }
                        }
                    }
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
            visible: !root.umuPresent
            label: qsTr("Proton support")

            SwitchField {
                width: parent.width
                label: qsTr("Install umu-run")
                description: qsTr("Proton runners launch through it")
                checked: root.installUmu
                onToggled: (val) => root.installUmu = val
            }
        }
    }

    footerLeft: M3Button {
        text: qsTr("Docs")
        variant: "tonal"
        onClicked: Qt.openUrlExternally("https://omikuji-launcher.github.io/omikuji/")
    }

    actions: M3Button {
        text: qsTr("Get started")
        variant: "filled"
        onClicked: root.finish()
    }
}
