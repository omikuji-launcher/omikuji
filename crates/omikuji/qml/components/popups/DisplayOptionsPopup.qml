import QtQuick
import omikuji 1.0
import QtQuick.Controls
import "../controls"


Popup {
    id: root

    property real zoomValue: 1.0
    property real zoomFrom: Math.min(0.6, root.zoomValue)
    property real zoomTo: Math.max(1.5, root.zoomValue)
    property real zoomStep: 0.05

    property int spacingValue: 16
    property int spacingFrom: Math.min(4, root.spacingValue)
    property int spacingTo: Math.max(40, root.spacingValue)
    property int spacingStep: 2

    property string sortValue: "default"
    property bool showSort: false
    property bool showHiddenValue: false
    property bool showHiddenOption: false
    property string cardStyleValue: "normal"
    property bool cardPlayButtonValue: false
    property bool showCardPlayButton: false

    signal zoomMoved(real value)
    signal spacingMoved(int value)
    signal sortSelected(string value)
    signal showHiddenToggled(bool value)
    signal cardStyleSelected(string value)
    signal cardPlayButtonToggled(bool value)

    padding: 16
    margins: 0
    width: 260
    modal: false
    focus: true
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutsideParent

    background: PopupSurface {}

    PopupZoom { target: root }

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

            M3Slider {
                width: parent.width
                label: qsTr("Card size")
                valueText: Math.round(root.zoomValue * 100) + "%"
                from: root.zoomFrom
                to: root.zoomTo
                stepSize: root.zoomStep
                value: root.zoomValue
                onMoved: (val) => root.zoomMoved(val)
            }

            M3Slider {
                width: parent.width
                label: qsTr("Card spacing")
                valueText: Math.round(root.spacingValue) + "px"
                from: root.spacingFrom
                to: root.spacingTo
                stepSize: root.spacingStep
                value: root.spacingValue
                onMoved: (val) => root.spacingMoved(Math.round(val))
            }

            Column {
                width: parent.width
                spacing: 8
                visible: root.showSort

                Text {
                    text: qsTr("Sort by")
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.label.size
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

            Column {
                width: parent.width
                spacing: 8

                Text {
                    text: qsTr("Card style")
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.label.size
                    font.weight: Font.Medium
                }

                M3Dropdown {
                    width: parent.width
                    options: CardStyles.options()
                    currentIndex: Math.max(0, options.findIndex(o => o.value === root.cardStyleValue))
                    onSelected: (value) => root.cardStyleSelected(value)
                }
            }

            LabeledSwitch {
                width: parent.width
                visible: root.showHiddenOption
                label: qsTr("Show hidden games")
                checked: root.showHiddenValue
                onToggled: (val) => root.showHiddenToggled(val)
            }

            LabeledSwitch {
                width: parent.width
                visible: root.showCardPlayButton
                label: qsTr("Play button on cards")
                checked: root.cardPlayButtonValue
                onToggled: (val) => root.cardPlayButtonToggled(val)
            }
        }
    }
}
