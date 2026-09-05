pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"
import "../primitives"
import "../lib/PlayState.js" as PlayState


Item {
    id: root

    property var actions: null
    property int playState: root.actions ? root.actions.playState : PlayState.Play
    property var activity: root.actions ? root.actions.downloadActivity : null
    property bool runnerUpdating: root.actions ? root.actions.runnerUpdating : false
    property bool suppressAnim: false

    property int index: root.actions ? root.actions.selectedIndex : -1
    property string gameId: root.actions ? root.actions.selectedGameId : ""
    property bool iconOnly: false

    signal activityClicked()

    readonly property bool hasActivity: root.activity !== null && root.activity !== undefined

    function activityLabel() {
        if (!root.hasActivity) return ""
        let s = root.activity.status || ""
        let kindWord = root.activity.kind === "update" ? qsTr("Updating") : qsTr("Installing")
        if (s === "Paused") return qsTr("Paused")
        if (s === "Queued") return qsTr("%1 · Queued").arg(kindWord)
        if (s === "Extracting") return qsTr("Extracting")
        if (s === "Patching") return qsTr("Patching")
        let pct = Math.round(root.activity.progress || 0)
        return qsTr("%1 · %2%").arg(kindWord).arg(pct)
    }

    component ButtonSlot: Item {
        id: slot

        property int forState: -1

        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        width: root.iconOnly ? 40 : 100
        height: 40
        opacity: root.playState === slot.forState ? 1 : 0
        visible: opacity > 0.001

        Behavior on opacity {
            enabled: !root.suppressAnim
            NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
        }
    }

    width: root.iconOnly ? 40 : (root.playState === PlayState.Activity ? 150 : 100)
    height: 40

    Behavior on width {
        NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
    }

    ButtonSlot {
        forState: PlayState.Launching

        M3Button {
            anchors.fill: parent
            variant: "filled"
            enabled: false
            icon: root.iconOnly ? "schedule" : ""
            text: root.iconOnly ? "" : qsTr("Starting")
        }
    }

    ButtonSlot {
        forState: PlayState.Stop

        M3Button {
            anchors.fill: parent
            variant: "filled"
            danger: true
            icon: root.iconOnly ? "stop" : ""
            text: root.iconOnly ? "" : qsTr("Stop")
            onClicked: {
                if (root.actions) root.actions.stop(root.gameId)
            }
        }
    }

    ButtonSlot {
        forState: PlayState.Activity
        width: parent.width

        Squircle {
            anchors.fill: parent
            radius: Theme.radius.lg
            fillColor: Theme.alpha(Theme.text, 0.08)

            // no width Behavior, it raced the opacity fade and painted outside the rounded bounds (XDDDDDDDDDDDDDDZ))IS)D(ISDJ(SJD))
            clip: true
            Rectangle {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.margins: 1
                radius: 11
                width: {
                    if (!root.hasActivity) return 0
                    let pct = (root.activity.progress || 0) / 100
                    return Math.max(0, Math.min((parent.width - 2) * pct, parent.width - 2))
                }
                color: Theme.alpha(Theme.accent, 0.15)
            }

            Row {
                anchors.centerIn: parent
                spacing: 6

                SvgIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "schedule"
                    size: 14
                    color: Theme.accent
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !root.iconOnly
                    text: root.activityLabel()
                    color: Theme.text
                    font.pixelSize: Theme.type.micro.size
                    font.weight: Font.DemiBold
                }
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.activityClicked()
            }
        }
    }

    ButtonSlot {
        forState: PlayState.Play

        M3Button {
            anchors.fill: parent
            variant: "filled"
            enabled: !root.runnerUpdating
            icon: root.iconOnly ? "play_arrow" : ""
            text: root.iconOnly ? "" : qsTr("Play")
            onClicked: {
                if (root.actions) root.actions.play(root.index)
            }
        }
    }
}
