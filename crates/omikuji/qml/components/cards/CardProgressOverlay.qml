pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import omikuji 1.0

Item {
    id: root

    property Item bannerArea: null
    property real bottomInset: 0
    property string status: ""
    property real progress: 0
    property bool animate: true

    readonly property bool isUninterruptible: status === "Extracting" || status === "Patching"
    readonly property bool isPaused: status === "Paused"

    readonly property real bannerInset: bannerArea ? bannerArea.x : 0

    Rectangle {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.leftMargin: root.bannerInset + Theme.space.xs
        anchors.rightMargin: root.bannerInset + Theme.space.xs
        anchors.bottomMargin: root.bottomInset + Theme.space.xs
        height: content.implicitHeight + Theme.space.sm * 2
        radius: Theme.radius.md
        color: Theme.alpha(Theme.bg, 0.86)

        ColumnLayout {
            id: content
            anchors.fill: parent
            anchors.margins: Theme.space.sm
            spacing: Theme.space.xs

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space.xs

                Text {
                    Layout.fillWidth: true
                    text: root.status
                    color: root.isUninterruptible ? Theme.warning : Theme.textMuted
                    font.pixelSize: Theme.type.micro.size
                    font.weight: Font.Medium
                    elide: Text.ElideRight
                }

                Text {
                    text: Math.round(root.progress) + "%"
                    visible: root.progress > 0
                    color: Theme.text
                    font.pixelSize: Theme.type.micro.size
                    font.weight: Font.DemiBold
                }
            }

            LinearProgress {
                Layout.fillWidth: true
                Layout.preferredHeight: 12
                value: Math.max(0, Math.min(1, root.progress / 100.0))
                paused: root.isPaused
                animate: root.animate
                trackWidth: 3
                handleWidth: 3
                handleHeight: 12
                handleMargins: 2
            }
        }
    }
}
