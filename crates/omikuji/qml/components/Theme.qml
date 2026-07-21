import QtQuick

QtObject {
    id: theme

    property SystemPalette active: SystemPalette { colorGroup: SystemPalette.Active }
    property SystemPalette inactive: SystemPalette { colorGroup: SystemPalette.Inactive }

    property bool followSystemColors: true
    readonly property bool systemColorsAvailable: Application.styleHints.colorScheme !== Qt.ColorScheme.Unknown
    property bool followSystemFont: true
    property string fontFamily: ""
    property var overrides: ({})

    function _resolve(token, system, fallback) {
        if (followSystemColors) return systemColorsAvailable ? system : fallback
        var v = overrides[token]
        return v ? v : fallback
    }

    property color accent: _resolve("accent", active.highlight, "#bdc2ff")
    property color accentText: _resolve("accentText", active.highlightedText, "#1d2678")
    property color accentOn: accent.hslLightness > 0.5 ? "#000000" : "#ffffff"

    property color bg: _resolve("bg", active.base, "#111111")
    property color bgAlt: Qt.darker(bg, 1.1)
    property color surface: _resolve("surface", active.base, "#111111")
    property color surfaceBorder: Qt.rgba(text.r, text.g, text.b, 0.08)

    property color text: _resolve("text", active.windowText, "#c5c6c6")
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
    property bool filledIcons: false
    property real uiScale: 1.0
    property color icon: Qt.rgba(text.r, text.g, text.b, mutedIcons ? 0.55 : 0.92)
    property color iconHover: Qt.rgba(text.r, text.g, text.b, mutedIcons ? 0.9 : 1.0)

    property color separator: Qt.rgba(text.r, text.g, text.b, 0.06)
    property color dot: Qt.rgba(text.r, text.g, text.b, 0.15)

    property color popup: Qt.hsla(bg.hslHue, bg.hslSaturation, bg.hslLightness, 1.0)

    property color tooltipBg: text
    property color tooltipText: bg

    property color error: _resolve("error", bg.hslLightness > 0.5 ? "#d32f2f" : "#ef5350", "#ef5350")
    property color success: _resolve("success", bg.hslLightness > 0.5 ? "#388e3c" : "#66bb6a", "#66bb6a")
    property color warning: _resolve("warning", bg.hslLightness > 0.5 ? "#f57c00" : "#ffa726", "#ffa726")

    function alpha(c, a) { return Qt.rgba(c.r, c.g, c.b, a) }
    function mix(a, b, t) {
        return Qt.rgba(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
    }
    function resolveColor(v) {
        const t = theme[v]
        return (t !== undefined && String(t).startsWith("#")) ? String(t) : v
    }

    property color outline: alpha(text, 0.12)
    property color outlineStrong: alpha(text, 0.24)
    property color stateHover: alpha(text, 0.06)
    property color statePressed: alpha(text, 0.11)

    property color fieldBg: alpha(text, 0.06)
    property color fieldBgFocus: alpha(text, 0.09)

    property bool fillFields: true

    property real radiusScale: 1.0

    property var fontSizes: ({})
    readonly property var fontDefaults: ({ micro: 12, caption: 12, label: 14, body: 14, subtitle: 16, title: 16, headline: 18, display: 22 })
    function _fontPx(role) {
        var v = fontSizes[role]
        return v > 0 ? v : fontDefaults[role]
    }

    property var radiusOverrides: ({})
    readonly property var radiusDefaults: ({ xs: 6, sm: 8, md: 12, lg: 16, xl: 22, xxl: 28, pill: 999 })
    function _radPx(role) {
        var v = radiusOverrides[role]
        if (v !== undefined) return v
        var d = radiusDefaults[role]
        return role === "pill" ? d : Math.round(d * radiusScale)
    }

    readonly property QtObject radius: QtObject {
        readonly property int xs: theme._radPx("xs")
        readonly property int sm: theme._radPx("sm")
        readonly property int md: theme._radPx("md")
        readonly property int lg: theme._radPx("lg")
        readonly property int xl: theme._radPx("xl")
        readonly property int xxl: theme._radPx("xxl")
        readonly property int pill: theme._radPx("pill")
    }

    readonly property QtObject space: QtObject {
        readonly property int xs: 4
        readonly property int sm: 8
        readonly property int md: 12
        readonly property int lg: 16
        readonly property int xl: 24
        readonly property int xxl: 32
    }

    readonly property QtObject dur: QtObject {
        readonly property int xfast: 90
        readonly property int fast: 140
        readonly property int med: 220
        readonly property int slow: 340
    }

    readonly property QtObject ease: QtObject {
        readonly property int standard: Easing.OutCubic
        readonly property int emphasized: Easing.OutBack
        readonly property real overshoot: 1.15
    }

    readonly property QtObject type: QtObject {
        readonly property var display: ({ size: theme._fontPx("display"), weight: Font.DemiBold })
        readonly property var headline: ({ size: theme._fontPx("headline"), weight: Font.DemiBold })
        readonly property var title: ({ size: theme._fontPx("title"), weight: Font.DemiBold })
        readonly property var subtitle: ({ size: theme._fontPx("subtitle"), weight: Font.Medium })
        readonly property var body: ({ size: theme._fontPx("body"), weight: Font.Normal })
        readonly property var label: ({ size: theme._fontPx("label"), weight: Font.Medium })
        readonly property var caption: ({ size: theme._fontPx("caption"), weight: Font.Normal })
        readonly property var micro: ({ size: theme._fontPx("micro"), weight: Font.Medium })
    }
}
