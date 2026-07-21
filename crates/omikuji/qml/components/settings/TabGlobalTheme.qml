import QtQuick
import QtQuick.Dialogs

import "."
import "../controls"

Item {
    id: root

    property var uiSettings: null

    signal manageFontSizesRequested()
    signal manageRadiiRequested()

    readonly property int rowLabelWidth: 200

    readonly property var tokens: [
        { key: "bg",         label: qsTr("Window background") },
        { key: "surface",    label: qsTr("Content surface") },
        { key: "accent",     label: qsTr("Accent") },
        { key: "accentText", label: qsTr("Accent text") },
        { key: "text",       label: qsTr("Text") },
        { key: "error",      label: qsTr("Error") },
        { key: "success",    label: qsTr("Success") },
        { key: "warning",    label: qsTr("Warning") }
    ]

    property var overrides: ({})
    property var fonts: []

    implicitHeight: content.height

    function _refresh() {
        if (!uiSettings) return
        try { overrides = JSON.parse(uiSettings.overridesJson()) } catch (e) { overrides = ({}) }
        try { fonts = JSON.parse(uiSettings.availableFontsJson()) } catch (e) { fonts = [] }
    }

    function _hasOverride(token) {
        return overrides[token] !== undefined && overrides[token] !== ""
    }

    function _effective(token) {
        if (!uiSettings || uiSettings.followSystemColors) return theme[token]
        return _hasOverride(token) ? overrides[token] : theme[token]
    }

    onUiSettingsChanged: _refresh()
    Component.onCompleted: _refresh()

    Connections {
        target: uiSettings
        function onThemeChanged() { root._refresh() }
    }

    ColorDialog {
        id: pickerDialog
        property string targetToken: ""
        onAccepted: {
            if (uiSettings && targetToken !== "") {
                uiSettings.setColorOverride(targetToken, selectedColor.toString())
            }
        }
    }

    Column {
        id: content
        width: parent.width
        spacing: theme.space.xxl

        SettingsSection {
            label: qsTr("Colors")
            width: parent.width

            SettingsRow {
                label: qsTr("Follow system")
                description: qsTr("Use the desktop palette. Disable to apply per-token overrides below.")
                labelWidth: root.rowLabelWidth
                M3Switch {
                    checked: uiSettings ? uiSettings.followSystemColors : true
                    onToggled: (value) => uiSettings.applyFollowSystemColors(value)
                }
            }

            Repeater {
                model: root.tokens
                delegate: SettingsRow {
                    required property var modelData
                    width: content.width
                    label: modelData.label
                    labelWidth: root.rowLabelWidth
                    opacity: (uiSettings && !uiSettings.followSystemColors) ? 1.0 : 0.4

                    Row {
                        spacing: 10

                        IconButton {
                            anchors.verticalCenter: parent.verticalCenter
                            icon: "close"
                            size: 24
                            danger: true
                            visible: root._hasOverride(modelData.key) && uiSettings && !uiSettings.followSystemColors
                            onClicked: uiSettings.setColorOverride(modelData.key, "")
                        }

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: root._hasOverride(modelData.key) ? root.overrides[modelData.key] : qsTr("system")
                            color: root._hasOverride(modelData.key) ? theme.text : theme.textSubtle
                            font.pixelSize: theme.type.label.size
                        }

                        Rectangle {
                            anchors.verticalCenter: parent.verticalCenter
                            width: 28
                            height: 28
                            radius: 11
                            color: root._effective(modelData.key)
                            border.width: 1
                            border.color: theme.surfaceBorder

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                enabled: uiSettings && !uiSettings.followSystemColors
                                onClicked: {
                                    pickerDialog.targetToken = modelData.key
                                    pickerDialog.selectedColor = root._effective(modelData.key)
                                    pickerDialog.open()
                                }
                            }
                        }
                    }
                }
            }
        }

        SettingsSection {
            label: qsTr("Font")
            width: parent.width

            SettingsRow {
                label: qsTr("Follow system")
                description: qsTr("Use the desktop default font. Disable to pick a family below.")
                labelWidth: root.rowLabelWidth
                M3Switch {
                    checked: uiSettings ? uiSettings.followSystemFont : true
                    onToggled: (value) => uiSettings.applyFollowSystemFont(value)
                }
            }

            SettingsRow {
                label: qsTr("Font family")
                description: qsTr("Applied app-wide. Requires restart.")
                labelWidth: root.rowLabelWidth
                opacity: (uiSettings && !uiSettings.followSystemFont) ? 1.0 : 0.4

                M3Dropdown {
                    width: 260
                    options: {
                        let arr = [{ label: qsTr("Default"), value: "" }]
                        for (let i = 0; i < root.fonts.length; i++) {
                            arr.push({ label: root.fonts[i], value: root.fonts[i] })
                        }
                        return arr
                    }
                    currentIndex: {
                        if (!uiSettings) return 0
                        let v = uiSettings.fontFamily
                        if (!v) return 0
                        for (let i = 0; i < root.fonts.length; i++) {
                            if (root.fonts[i] === v) return i + 1
                        }
                        return 0
                    }
                    onSelected: (value) => uiSettings.applyFontFamily(value)
                }
            }

            SettingsRow {
                label: qsTr("Font sizes")
                description: qsTr("Per-role text sizes used across the app.")
                labelWidth: root.rowLabelWidth

                M3Button {
                    text: qsTr("Manage")
                    variant: "tonal"
                    onClicked: root.manageFontSizesRequested()
                }
            }
        }

        SettingsSection {
            label: qsTr("Shape")
            width: parent.width

            SettingsRow {
                label: qsTr("Corner radius")
                description: qsTr("Per-token corner rounding used across the app.")
                labelWidth: root.rowLabelWidth

                M3Button {
                    text: qsTr("Manage")
                    variant: "tonal"
                    onClicked: root.manageRadiiRequested()
                }
            }
        }
    }
}
