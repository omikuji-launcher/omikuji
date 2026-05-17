import QtQuick

QtObject {
    id: theme

    property SystemPalette active: SystemPalette { colorGroup: SystemPalette.Active }
    property SystemPalette inactive: SystemPalette { colorGroup: SystemPalette.Inactive }

    property bool followSystemColors: true
    property bool followSystemFont: true
    property string fontFamily: ""
    property var overrides: ({})

    function _resolve(token, fallback) {
        if (followSystemColors) return fallback
        var v = overrides[token]
        return v ? v : fallback
    }

    property color accent: _resolve("accent", active.highlight)
    property color accentText: _resolve("accentText", active.highlightedText)
    property color accentOn: accent.hslLightness > 0.5 ? "#000000" : "#ffffff"

    property color bg: _resolve("bg", active.window)
    property color bgAlt: Qt.darker(bg, 1.1)
    property color surface: _resolve("surface", active.base)
    property color surfaceHover: Qt.lighter(surface, 1.1)
    property color surfaceBorder: Qt.rgba(text.r, text.g, text.b, 0.08)

    property color text: _resolve("text", active.windowText)
    property color textMuted: Qt.rgba(text.r, text.g, text.b, 0.55)
    property color textSubtle: Qt.rgba(text.r, text.g, text.b, 0.35)
    property color textFaint: Qt.rgba(text.r, text.g, text.b, 0.2)

    property color navBg: bg
    property color navSeparator: Qt.rgba(text.r, text.g, text.b, 0.06)

    property color cardBg: Qt.lighter(surface, 1.08)
    property color cardBorder: "transparent"
    property color cardBorderHover: Qt.rgba(text.r, text.g, text.b, 0.12)

    property color barBg: Qt.rgba(bg.r, bg.g, bg.b, 0.92)
    property color barBorder: Qt.rgba(text.r, text.g, text.b, 0.08)

    property bool mutedIcons: false
    property color icon: Qt.rgba(text.r, text.g, text.b, mutedIcons ? 0.55 : 0.92)
    property color iconHover: Qt.rgba(text.r, text.g, text.b, mutedIcons ? 0.9 : 1.0)

    property color separator: Qt.rgba(text.r, text.g, text.b, 0.06)
    property color dot: Qt.rgba(text.r, text.g, text.b, 0.15)

    property color popup: Qt.hsla(bg.hslHue, bg.hslSaturation, bg.hslLightness, 1.0)

    property color tooltipBg: text
    property color tooltipText: bg

    property color error: _resolve("error", bg.hslLightness > 0.5 ? "#d32f2f" : "#ef5350")
    property color success: _resolve("success", bg.hslLightness > 0.5 ? "#388e3c" : "#66bb6a")
    property color warning: _resolve("warning", bg.hslLightness > 0.5 ? "#f57c00" : "#ffa726")
}
