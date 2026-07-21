import QtQuick
import QtQuick.Layouts
import "../controls"
import "../popups"
import "../primitives"

DialogCard {
    id: root

    signal saved(var entry, int index)
    signal closed()

    property int _editingIndex: -1

    property string formName: ""
    property string formIcon: "star"
    property string formKind: "tag"
    property string formValue: ""

    readonly property var kindOptions: [
        { label: qsTr("All games"),  value: "all" },
        { label: qsTr("Favourites"), value: "favourite" },
        { label: qsTr("Recent"),     value: "recent" },
        { label: qsTr("Runner"),     value: "runner" },
        { label: qsTr("Tag"),        value: "tag" }
    ]

    readonly property var runnerOptions: [
        { label: "Wine / Proton", value: "wine" },
        { label: "Native",        value: "native" },
        { label: "Steam",         value: "steam" },
        { label: "Flatpak",       value: "flatpak" }
    ]

    readonly property bool _valueNeeded: formKind === "runner" || formKind === "tag"

    readonly property int _kindIndex: {
        for (let i = 0; i < kindOptions.length; i++)
            if (kindOptions[i].value === formKind) return i
        return 0
    }
    readonly property int _runnerIndex: {
        for (let i = 0; i < runnerOptions.length; i++)
            if (runnerOptions[i].value === formValue) return i
        return 0
    }

    maxWidth: 480
    title: _editingIndex === -1 ? qsTr("Add category") : qsTr("Edit category")
    escEnabled: !iconPicker.visible

    function showAdd() {
        _editingIndex = -1
        formName = ""
        formIcon = "star"
        formKind = "tag"
        formValue = ""
        open()
    }

    function showEdit(index, entry) {
        _editingIndex = index
        formName = entry.name || ""
        formIcon = entry.icon || "star"
        formKind = entry.kind || "tag"
        formValue = entry.value || ""
        open()
    }

    function hide() { root.closed(); close() }

    function _buildEntry() {
        let v = ""
        if (formKind === "runner") v = runnerOptions[root._runnerIndex].value
        else if (formKind === "tag") v = formValue.trim()
        return {
            enabled: true,
            name: formName.trim(),
            icon: formIcon,
            kind: formKind,
            value: v
        }
    }

    onCloseRequested: { root.closed(); root.close() }

    body: ColumnLayout {
        width: parent.width
        spacing: theme.space.md

        M3TextField {
            id: nameField
            Layout.fillWidth: true
            label: qsTr("Name")
            text: root.formName
            onTextEdited: (t) => root.formName = t
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4

            Text {
                text: qsTr("Icon")
                color: theme.textMuted
                font.pixelSize: theme.type.body.size
                font.weight: Font.Medium
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: theme.space.md

                Rectangle {
                    Layout.preferredWidth: 48
                    Layout.preferredHeight: 48
                    radius: 10
                    color: theme.alpha(theme.text, 0.06)
                    border.width: 1
                    border.color: theme.alpha(theme.text, 0.12)

                    SvgIcon {
                        anchors.centerIn: parent
                        name: root.formIcon
                        size: 22
                        color: theme.icon
                    }
                }

                M3Button {
                    text: qsTr("Change")
                    variant: "tonal"
                    onClicked: iconPicker.show(root.formIcon)
                }

                Item { Layout.fillWidth: true }
            }
        }

        M3Dropdown {
            id: kindDropdown
            Layout.fillWidth: true
            label: qsTr("Kind")
            options: root.kindOptions
            currentIndex: root._kindIndex
            onSelected: (value) => root.formKind = value
        }

        M3TextField {
            id: tagValueField
            Layout.fillWidth: true
            visible: root.formKind === "tag"
            label: qsTr("Tag value")
            placeholder: qsTr("e.g. anime, speedrun")
            text: root.formValue
            onTextEdited: (t) => root.formValue = t
        }

        M3Dropdown {
            id: runnerDropdown
            Layout.fillWidth: true
            visible: root.formKind === "runner"
            label: qsTr("Runner")
            options: root.runnerOptions
            currentIndex: root._runnerIndex
            onSelected: (value) => root.formValue = value
        }
    }

    actions: Row {
        spacing: theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "text"
            onClicked: { root.closed(); root.close() }
        }

        M3Button {
            text: qsTr("Save")
            variant: "filled"
            enabled: root.formName.trim().length > 0
            onClicked: {
                root.saved(root._buildEntry(), root._editingIndex)
                root.closed()
                root.close()
            }
        }
    }

    IconPickerPopup {
        id: iconPicker
        anchors.fill: parent
        onPicked: (name) => root.formIcon = name
    }
}
