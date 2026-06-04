import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects

import "../widgets"

Item {
    id: dialog

    // index -1 means append
    signal saved(var entry, int index)
    signal closed()

    property int _editingIndex: -1

    property string formName: ""
    property string formIcon: "star"
    property string formKind: "tag"
    property string formValue: ""

    readonly property var kindOptions: [
        { label: "All games",  value: "all" },
        { label: "Favourites", value: "favourite" },
        { label: "Recent",     value: "recent" },
        { label: "Runner",     value: "runner" },
        { label: "Tag",        value: "tag" }
    ]

    readonly property var runnerOptions: [
        { label: "Wine / Proton", value: "wine" },
        { label: "Native",        value: "native" },
        { label: "Steam",         value: "steam" },
        { label: "Flatpak",       value: "flatpak" }
    ]

    readonly property bool _valueNeeded: formKind === "runner" || formKind === "tag"

    visible: false
    z: 2100

    function showAdd() {
        _editingIndex = -1
        formName = ""
        formIcon = "star"
        formKind = "tag"
        formValue = ""
        _syncKindIndex()
        _syncRunnerIndex()
        visible = true
    }

    function showEdit(index, entry) {
        _editingIndex = index
        formName = entry.name || ""
        formIcon = entry.icon || "star"
        formKind = entry.kind || "tag"
        formValue = entry.value || ""
        _syncKindIndex()
        _syncRunnerIndex()
        visible = true
    }

    function hide() {
        visible = false
        closed()
    }

    function _syncKindIndex() {
        for (let i = 0; i < kindOptions.length; i++) {
            if (kindOptions[i].value === formKind) {
                kindDropdown.currentIndex = i
                return
            }
        }
        kindDropdown.currentIndex = 0
    }

    function _syncRunnerIndex() {
        for (let i = 0; i < runnerOptions.length; i++) {
            if (runnerOptions[i].value === formValue) {
                runnerDropdown.currentIndex = i
                return
            }
        }
        runnerDropdown.currentIndex = 0
    }

    function _buildEntry() {
        let v = ""
        if (formKind === "runner") v = runnerOptions[runnerDropdown.currentIndex].value
        else if (formKind === "tag") v = formValue.trim()
        return {
            enabled: true,
            name: formName.trim(),
            icon: formIcon,
            kind: formKind,
            value: v
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, 0.55)
        MouseArea {
            anchors.fill: parent
            hoverEnabled: true
            acceptedButtons: Qt.AllButtons
            onClicked: (mouse) => { if (mouse.button === Qt.LeftButton) dialog.hide() }
            onWheel: (wheel) => wheel.accepted = true
            cursorShape: Qt.ArrowCursor
        }
    }

    Rectangle {
        id: card
        anchors.centerIn: parent
        width: Math.min(parent.width - 80, 480)
        height: Math.min(parent.height - 60, contentCol.implicitHeight + 44)
        radius: 22
        color: theme.surface
        border.width: 1
        border.color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.08)

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            onClicked: {}
            onWheel: (wheel) => wheel.accepted = true
        }

        layer.enabled: true
        layer.effect: DropShadow {
            radius: 24
            samples: 32
            color: Qt.rgba(0, 0, 0, 0.4)
            horizontalOffset: 0
            verticalOffset: 6
        }

        Flickable {
            id: cardScroll
            anchors.fill: parent
            anchors.margins: 22
            contentWidth: width
            contentHeight: contentCol.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            interactive: contentHeight > height
            ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

        ColumnLayout {
            id: contentCol
            width: cardScroll.width
            spacing: 16

            Text {
                text: dialog._editingIndex === -1 ? "Add category" : "Edit category"
                color: theme.text
                font.pixelSize: 17
                font.weight: Font.DemiBold
            }

            M3TextField {
                id: nameField
                Layout.fillWidth: true
                label: "Name"
                text: dialog.formName
                onTextEdited: dialog.formName = nameField.text
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4

                Text {
                    text: "Icon"
                    color: theme.textMuted
                    font.pixelSize: 14
                    font.weight: Font.Medium
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 12

                    Rectangle {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48
                        radius: 10
                        color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                        border.width: 1
                        border.color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.12)

                        SvgIcon {
                            anchors.centerIn: parent
                            name: dialog.formIcon
                            size: 22
                            color: theme.icon
                        }
                    }

                    Item {
                        Layout.preferredWidth: 100
                        Layout.preferredHeight: 34

                        Rectangle {
                            anchors.fill: parent
                            radius: 17
                            color: pickHover.containsPress
                                ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.12)
                                : pickHover.containsMouse
                                    ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                                    : "transparent"
                            border.width: 1
                            border.color: Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.15)
                            Behavior on color { ColorAnimation { duration: 100 } }
                        }
                        Text {
                            anchors.centerIn: parent
                            text: "Change"
                            color: theme.text
                            font.pixelSize: 13
                            font.weight: Font.Medium
                        }
                        MouseArea {
                            id: pickHover
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: iconPicker.show(dialog.formIcon)
                        }
                    }

                    Item { Layout.fillWidth: true }
                }
            }

            M3Dropdown {
                id: kindDropdown
                Layout.fillWidth: true
                label: "Kind"
                options: dialog.kindOptions
                onSelected: (value) => dialog.formKind = value
            }

            M3TextField {
                id: tagValueField
                Layout.fillWidth: true
                visible: dialog.formKind === "tag"
                label: "Tag value"
                placeholder: "e.g. anime, speedrun"
                text: dialog.formValue
                onTextEdited: dialog.formValue = tagValueField.text
            }

            M3Dropdown {
                id: runnerDropdown
                Layout.fillWidth: true
                visible: dialog.formKind === "runner"
                label: "Runner"
                options: dialog.runnerOptions
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: 4
                spacing: 10

                Item { Layout.fillWidth: true }

                Item {
                    implicitWidth: 90
                    implicitHeight: 36

                    Rectangle {
                        anchors.fill: parent
                        radius: 18
                        color: cancelHover.containsPress
                            ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.12)
                            : cancelHover.containsMouse
                                ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                                : "transparent"
                        Behavior on color { ColorAnimation { duration: 100 } }
                    }
                    Text {
                        anchors.centerIn: parent
                        text: "Cancel"
                        color: theme.text
                        font.pixelSize: 13
                        font.weight: Font.Medium
                    }
                    MouseArea {
                        id: cancelHover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: dialog.hide()
                    }
                }

                Item {
                    implicitWidth: 100
                    implicitHeight: 36
                    opacity: dialog.formName.trim().length > 0 ? 1.0 : 0.5
                    enabled: dialog.formName.trim().length > 0

                    Rectangle {
                        anchors.fill: parent
                        radius: 18
                        color: theme.accent
                        opacity: saveHover.containsPress ? 0.8
                            : saveHover.containsMouse ? 0.95 : 0.9
                        scale: saveHover.containsPress ? 0.97 : 1.0
                        Behavior on opacity { NumberAnimation { duration: 100 } }
                        Behavior on scale { NumberAnimation { duration: 100 } }
                    }
                    Text {
                        anchors.centerIn: parent
                        text: "Save"
                        color: theme.accentOn
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                    }
                    MouseArea {
                        id: saveHover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            dialog.saved(dialog._buildEntry(), dialog._editingIndex)
                            dialog.hide()
                        }
                    }
                }
            }
        }
        }
    }

    IconPickerPopup {
        id: iconPicker
        anchors.fill: parent
        onPicked: (name) => dialog.formIcon = name
    }
}
