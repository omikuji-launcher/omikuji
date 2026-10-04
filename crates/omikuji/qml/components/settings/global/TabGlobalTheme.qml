pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Item {
    id: root

    property var appSettings: null

    signal manageFontSizesRequested()
    signal manageRadiiRequested()
    signal colorPickRequested(color initial, var onPicked)

    readonly property int rowLabelWidth: 200

    readonly property var tokens: [
        { key: "bg",         label: qsTr("Window background") },
        { key: "surface",    label: qsTr("Content surface") },
        { key: "cardBg",     label: qsTr("Card background") },
        { key: "accent",     label: qsTr("Accent") },
        { key: "accentText", label: qsTr("Accent text") },
        { key: "secondary",  label: qsTr("Secondary") },
        { key: "text",       label: qsTr("Text") },
        { key: "error",      label: qsTr("Error") },
        { key: "success",    label: qsTr("Success") },
        { key: "warning",    label: qsTr("Warning") }
    ]

    property var overrides: ({})
    property var fonts: []
    readonly property var fontOptions: [{ label: qsTr("Default"), value: "" }]
        .concat(fonts.map(f => ({ label: f, value: f })))

    implicitHeight: content.height

    function _refresh() {
        if (!appSettings) return
        try { overrides = JSON.parse(appSettings.overridesJson()) } catch (e) { overrides = ({}) }
        try { fonts = JSON.parse(appSettings.availableFontsJson()) } catch (e) { fonts = [] }
    }

    function _hasOverride(token) {
        return overrides[token] !== undefined && overrides[token] !== ""
    }

    function _effective(token) {
        if (!appSettings || appSettings.followSystemColors) return Theme[token]
        return _hasOverride(token) ? overrides[token] : Theme[token]
    }

    function _fontIndex(family) {
        return fonts.indexOf(family) + 1
    }

    onAppSettingsChanged: _refresh()
    Component.onCompleted: _refresh()

    Connections {
        target: root.appSettings
        function onThemeChanged() { root._refresh() }
    }

    Column {
        id: content
        width: parent.width
        spacing: Theme.space.xxl

        SettingsSection {
            label: qsTr("Colors")
            width: parent.width

            SettingsRow {
                label: qsTr("Follow system")
                description: qsTr("Use the desktop palette. Disable to apply per-token overrides below.")
                labelWidth: root.rowLabelWidth
                M3Switch {
                    checked: root.appSettings ? root.appSettings.followSystemColors : true
                    onToggled: (value) => root.appSettings.applyFollowSystemColors(value)
                }
            }

            Repeater {
                model: root.tokens
                delegate: SettingsRow {
                    id: tokenRow
                    required property var modelData
                    width: content.width
                    label: modelData.label
                    labelWidth: root.rowLabelWidth
                    opacity: (root.appSettings && !root.appSettings.followSystemColors) ? 1.0 : 0.4

                    Row {
                        spacing: 10

                        IconButton {
                            anchors.verticalCenter: parent.verticalCenter
                            icon: "close"
                            size: 24
                            danger: true
                            visible: root._hasOverride(tokenRow.modelData.key) && root.appSettings && !root.appSettings.followSystemColors
                            onClicked: root.appSettings.setColorOverride(tokenRow.modelData.key, "")
                        }

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: root._hasOverride(tokenRow.modelData.key) ? root.overrides[tokenRow.modelData.key] : qsTr("system")
                            color: root._hasOverride(tokenRow.modelData.key) ? Theme.text : Theme.textSubtle
                            font.pixelSize: Theme.type.label.size
                        }

                        ColorSwatch {
                            anchors.verticalCenter: parent.verticalCenter
                            fillColor: root._effective(tokenRow.modelData.key)

                            PressArea {
                                anchors.fill: parent
                                ringRadius: parent.radius
                                enabled: root.appSettings && !root.appSettings.followSystemColors
                                onActivated: {
                                    const key = tokenRow.modelData.key
                                    root.colorPickRequested(root._effective(key), (c) => root.appSettings.setColorOverride(key, c))
                                }
                            }
                        }
                    }
                }
            }

            SettingsRow {
                label: qsTr("Dimmed text opacity")
                labelWidth: root.rowLabelWidth
                width: parent.width
                contentRightMargin: 74

                M3SpinBox {
                    from: 20
                    to: 100
                    stepSize: 5
                    suffix: "%"
                    value: root.appSettings ? Math.round(root.appSettings.dimmedTextOpacity * 100) : 55
                    onMoved: (val) => root.appSettings.applyDimmedTextOpacity(val / 100)
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
                    checked: root.appSettings ? root.appSettings.followSystemFont : true
                    onToggled: (value) => root.appSettings.applyFollowSystemFont(value)
                }
            }

            SettingsRow {
                label: qsTr("Font family")
                description: qsTr("Applied app-wide. Requires restart.")
                labelWidth: root.rowLabelWidth
                opacity: (root.appSettings && !root.appSettings.followSystemFont) ? 1.0 : 0.4

                M3Dropdown {
                    width: 260
                    options: root.fontOptions
                    currentIndex: root._fontIndex(root.appSettings ? root.appSettings.fontFamily : "")
                    onSelected: (value) => root.appSettings.applyFontFamily(value)
                }
            }

            SettingsRow {
                label: qsTr("Mono font")
                description: qsTr("Paths, versions and inline values.")
                labelWidth: root.rowLabelWidth

                M3Dropdown {
                    width: 260
                    options: root.fontOptions
                    currentIndex: root._fontIndex(root.appSettings ? root.appSettings.fontFamilyMono : "")
                    onSelected: (value) => root.appSettings.applyFontFamilyMono(value)
                }
            }

            SettingsRow {
                label: qsTr("Logs font")
                description: qsTr("Log views and preformatted blocks.")
                labelWidth: root.rowLabelWidth

                M3Dropdown {
                    width: 260
                    options: root.fontOptions
                    currentIndex: root._fontIndex(root.appSettings ? root.appSettings.fontFamilyLogs : "")
                    onSelected: (value) => root.appSettings.applyFontFamilyLogs(value)
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
