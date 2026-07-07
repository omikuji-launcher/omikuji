import QtQuick
import QtQuick.Controls
import "../controls"


Popup {
    id: root

    property real zoomValue: 1.0
    property real zoomFrom: 0.6
    property real zoomTo: 1.5
    property real zoomStep: 0.05

    property int spacingValue: 16
    property int spacingFrom: 4
    property int spacingTo: 40
    property int spacingStep: 2

    property string sortValue: "default"
    property bool showSort: false

    signal zoomMoved(real value)
    signal spacingMoved(int value)
    signal sortSelected(string value)

    padding: 16
    margins: 0
    width: 260
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent

    background: PopupSurface {}

    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: 120; easing.type: Easing.OutCubic }
        NumberAnimation { property: "scale"; from: 0.96; to: 1; duration: 120; easing.type: Easing.OutCubic }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1; to: 0; duration: 80 }
    }

    contentItem: Item {
        property bool isDropdownHost: true
        implicitWidth: optionsCol.implicitWidth
        implicitHeight: optionsCol.implicitHeight

        Column {
            id: optionsCol
            width: parent.width
            spacing: 14

            Column {
                width: parent.width
                spacing: 8

                Row {
                    width: parent.width
                    spacing: 8

                    Text {
                        text: qsTr("Card size")
                        color: theme.textMuted
                        font.pixelSize: 12
                        font.weight: Font.Medium
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Item { height: 1; width: parent.width - zoomVal.width - x - 16 }
                    Text {
                        id: zoomVal
                        text: Math.round(root.zoomValue * 100) + "%"
                        color: theme.text
                        font.pixelSize: 12
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }

                M3Slider {
                    width: parent.width
                    label: ""
                    showValue: false
                    from: root.zoomFrom
                    to: root.zoomTo
                    stepSize: root.zoomStep
                    value: root.zoomValue
                    onMoved: root.zoomMoved(value)
                }
            }

            Column {
                width: parent.width
                spacing: 8

                Row {
                    width: parent.width
                    spacing: 8

                    Text {
                        text: qsTr("Card spacing")
                        color: theme.textMuted
                        font.pixelSize: 12
                        font.weight: Font.Medium
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Item { height: 1; width: parent.width - spacingVal.width - x - 16 }
                    Text {
                        id: spacingVal
                        text: Math.round(root.spacingValue) + "px"
                        color: theme.text
                        font.pixelSize: 12
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }

                M3Slider {
                    width: parent.width
                    label: ""
                    showValue: false
                    from: root.spacingFrom
                    to: root.spacingTo
                    stepSize: root.spacingStep
                    value: root.spacingValue
                    onMoved: root.spacingMoved(Math.round(value))
                }
            }

            Column {
                width: parent.width
                spacing: 8
                visible: root.showSort

                Text {
                    text: qsTr("Sort by")
                    color: theme.textMuted
                    font.pixelSize: 12
                    font.weight: Font.Medium
                }

                M3Dropdown {
                    width: parent.width
                    options: [
                        { label: qsTr("Date added"), value: "default" },
                        { label: qsTr("Name A-Z"), value: "a-z" },
                        { label: qsTr("Name Z-A"), value: "z-a" },
                        { label: qsTr("Custom"), value: "custom" }
                    ]
                    currentIndex: Math.max(0, options.findIndex(o => o.value === root.sortValue))
                    onSelected: (value) => root.sortSelected(value)
                }
            }
        }
    }
}
