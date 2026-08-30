pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import Qt5Compat.GraphicalEffects
import "../controls"
import "../primitives"
import "../lib/Format.js" as Format

Item {
    id: root

    property var dlcs: []
    property var checkedIds: []
    property var lockedIds: []
    property bool readOnly: false
    property bool removable: false
    readonly property int visibleRows: 3
    readonly property int rowHeight: 52

    signal toggleRequested(string id)
    signal removeRequested(string id)

    implicitHeight: surface.height

    FieldSurface {
        id: surface
        width: parent.width
        height: Math.min(root.dlcs.length, root.visibleRows) * root.rowHeight + Theme.space.sm * 2

        ListView {
            id: list
            anchors.fill: parent
            anchors.topMargin: Theme.space.sm
            anchors.bottomMargin: Theme.space.sm
            clip: true
            model: root.dlcs
            boundsBehavior: Flickable.StopAtBounds

            ThinScrollBar.vertical: ThinScrollBar {}

            delegate: Item {
                id: dlcRow
                required property var modelData

                readonly property bool locked: root.lockedIds.indexOf(dlcRow.modelData.id) !== -1
                readonly property bool selected: dlcRow.locked
                    || root.checkedIds.indexOf(dlcRow.modelData.id) !== -1

                width: list.width
                height: root.rowHeight
                opacity: dlcRow.locked ? 0.55 : 1

                Rectangle {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space.sm
                    anchors.rightMargin: Theme.space.sm
                    radius: Theme.radius.sm
                    color: (rowArea.containsMouse && !dlcRow.locked) ? Theme.alpha(Theme.text, 0.06) : "transparent"
                    Behavior on color { ColorAnimation { duration: Theme.dur.fast } }
                }

                Rectangle {
                    id: artBox
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.space.md
                    anchors.verticalCenter: parent.verticalCenter
                    width: 56
                    height: 32
                    radius: Theme.radius.sm
                    color: art.visible ? "transparent" : Theme.alpha(Theme.accent, 0.15)

                    Image {
                        id: art
                        anchors.fill: parent
                        visible: dlcRow.modelData.image !== undefined
                            && dlcRow.modelData.image !== ""
                            && status === Image.Ready
                        source: dlcRow.modelData.image || ""
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                        sourceSize.width: 112
                        sourceSize.height: 64
                        layer.enabled: visible
                        layer.smooth: true
                        layer.effect: OpacityMask {
                            maskSource: Rectangle {
                                width: artBox.width
                                height: artBox.height
                                radius: artBox.radius
                            }
                        }
                    }

                    Text {
                        anchors.centerIn: parent
                        visible: !art.visible
                        text: (dlcRow.modelData.title || "?").charAt(0).toUpperCase()
                        color: Theme.accent
                        font.pixelSize: Theme.type.subtitle.size
                        font.weight: Font.DemiBold
                    }
                }

                Column {
                    anchors.left: artBox.right
                    anchors.leftMargin: Theme.space.md
                    anchors.right: root.removable ? removeBtn.left : check.left
                    anchors.rightMargin: Theme.space.md
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 1

                    Text {
                        width: parent.width
                        text: dlcRow.modelData.title || dlcRow.modelData.id
                        color: Theme.text
                        font.pixelSize: Theme.type.body.size
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        visible: text !== ""
                        text: dlcRow.modelData.downloadBytes > 0
                            ? Format.formatBytesShort(dlcRow.modelData.downloadBytes)
                            : ""
                        color: Theme.textSubtle
                        font.pixelSize: Theme.type.caption.size
                    }
                }

                IconButton {
                    id: removeBtn
                    icon: "close"
                    size: 24
                    danger: true
                    z: 2
                    visible: root.removable
                    anchors.right: parent.right
                    anchors.rightMargin: Theme.space.md
                    anchors.verticalCenter: parent.verticalCenter
                    opacity: rowArea.containsMouse || hovered ? 1 : 0
                    Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
                    onClicked: root.removeRequested(dlcRow.modelData.id)
                }

                M3Checkbox {
                    id: check
                    visible: !root.readOnly
                    anchors.right: parent.right
                    anchors.rightMargin: Theme.space.lg
                    anchors.verticalCenter: parent.verticalCenter
                    checked: dlcRow.selected
                }

                MouseArea {
                    id: rowArea
                    anchors.fill: parent
                    enabled: (!root.readOnly || root.removable) && !dlcRow.locked
                    hoverEnabled: enabled
                    cursorShape: (root.readOnly || dlcRow.locked) ? Qt.ArrowCursor : Qt.PointingHandCursor
                    onClicked: if (!root.readOnly) root.toggleRequested(dlcRow.modelData.id)
                }
            }
        }
    }
}
