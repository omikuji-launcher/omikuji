pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../lib/Format.js" as Format

Item {
    id: root

    property var items: []
    property var checkedIds: []
    property var lockedIds: []
    property bool readOnly: false
    property bool removable: false
    property string title: ""
    property bool checkAllVisible: false
    property int visibleRows: 3
    property bool fillHeight: false
    readonly property int rowHeight: 52

    readonly property var selectableIds: items
        .map(i => i.id)
        .filter(id => lockedIds.indexOf(id) === -1)
    readonly property int checkedSelectableCount: selectableIds
        .filter(id => checkedIds.indexOf(id) !== -1)
        .length
    readonly property bool allChecked: selectableIds.length > 0
        && checkedSelectableCount === selectableIds.length

    signal selectionRequested(var ids)
    signal removeRequested(string id)

    implicitHeight: content.height

    function toggle(id) {
        root.selectionRequested(checkedIds.indexOf(id) !== -1
            ? checkedIds.filter(v => v !== id)
            : checkedIds.concat([id]))
    }

    Column {
        id: content
        width: parent.width
        spacing: Theme.space.sm

        CheckAllHeader {
            id: header
            width: parent.width
            visible: root.title !== "" || root.checkAllVisible
            title: root.title
            checkAllVisible: root.checkAllVisible
            checked: root.allChecked
            indeterminate: root.checkedSelectableCount > 0
            onCheckAllClicked: root.selectionRequested(root.lockedIds.concat(root.allChecked ? [] : root.selectableIds))
        }

        FieldSurface {
            id: surface
            width: parent.width
            height: root.fillHeight
                ? root.height - (header.visible ? header.height + content.spacing : 0)
                : Math.min(root.items.length, root.visibleRows) * root.rowHeight + Theme.space.sm * 2

            WheelSink {
                anchors.fill: parent
            }

            ListView {
                id: list
                keyNavigationEnabled: false
                currentIndex: -1
                anchors.fill: parent
                anchors.topMargin: Theme.space.sm
                anchors.bottomMargin: Theme.space.sm
                clip: true
                model: root.items
                boundsBehavior: Flickable.StopAtBounds

                ThinScrollBar.vertical: ThinScrollBar {}

                delegate: Item {
                    id: itemRow
                    required property var modelData

                    readonly property bool locked: root.lockedIds.indexOf(itemRow.modelData.id) !== -1
                    readonly property bool selected: itemRow.locked
                        || root.checkedIds.indexOf(itemRow.modelData.id) !== -1

                    width: list.width
                    height: root.rowHeight
                    opacity: itemRow.locked ? 0.55 : 1

                    Rectangle {
                        anchors.fill: parent
                        anchors.leftMargin: Theme.space.sm
                        anchors.rightMargin: Theme.space.sm
                        radius: Theme.radius.sm
                        color: (rowArea.containsMouse && !itemRow.locked) ? Theme.alpha(Theme.text, 0.06) : "transparent"
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
                            visible: itemRow.modelData.image !== undefined
                                && itemRow.modelData.image !== ""
                                && status === Image.Ready
                            source: itemRow.modelData.image || ""
                            fillMode: Image.PreserveAspectCrop
                            asynchronous: true
                            sourceSize.width: 112
                            sourceSize.height: 64
                            layer.enabled: visible
                            layer.smooth: true
                            layer.effect: RoundedRectMask {
                                radius: artBox.radius
                            }
                        }

                        Text {
                            anchors.centerIn: parent
                            visible: !art.visible
                            text: (itemRow.modelData.title || "?").charAt(0).toUpperCase()
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
                            text: itemRow.modelData.title || itemRow.modelData.id
                            color: Theme.text
                            font.pixelSize: Theme.type.body.size
                            elide: Text.ElideRight
                        }

                        Text {
                            width: parent.width
                            visible: text !== ""
                            text: itemRow.modelData.downloadBytes > 0
                                ? Format.formatBytesShort(itemRow.modelData.downloadBytes)
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
                        opacity: rowArea.containsMouse || hovered || InputMode.keyFocus(rowArea) || InputMode.keyFocus(removeBtn) ? 1 : 0
                        Behavior on opacity { NumberAnimation { duration: Theme.dur.fast } }
                        onClicked: root.removeRequested(itemRow.modelData.id)
                    }

                    M3Checkbox {
                        id: check
                        visible: !root.readOnly
                        anchors.right: parent.right
                        anchors.rightMargin: Theme.space.lg
                        anchors.verticalCenter: parent.verticalCenter
                        checked: itemRow.selected
                    }

                    PressArea {
                        id: rowArea
                        anchors.fill: parent
                        enabled: (!root.readOnly || root.removable) && !itemRow.locked
                        hoverEnabled: enabled
                        cursorShape: (root.readOnly || itemRow.locked) ? Qt.ArrowCursor : Qt.PointingHandCursor
                        onActivated: if (!root.readOnly) root.toggle(itemRow.modelData.id)
                    }
                }
            }
        }
    }
}
