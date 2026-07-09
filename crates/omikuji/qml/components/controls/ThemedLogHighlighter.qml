import QtQuick
import omikuji 1.0

LogHighlighter {
    id: root

    property var settings: null

    active: settings ? settings.highlightLogs : true
    errorColor: theme.error
    warnColor: theme.warning
    fixmeColor: theme.accent
    traceColor: theme.textSubtle

    readonly property string themePulse: [theme.accent, theme.error, theme.warning, theme.success, theme.text, theme.textMuted, theme.textSubtle].join("")

    function pushRules() {
        if (!settings) return
        let rules = []
        try { rules = JSON.parse(settings.logRulesJson()) } catch (e) {}
        const resolved = rules.map(r => ({ pattern: r.pattern, color: theme.resolveColor(r.color) }))
        setRules(JSON.stringify(resolved))
    }

    onActiveChanged: refresh()
    onErrorColorChanged: refresh()
    onWarnColorChanged: refresh()
    onFixmeColorChanged: refresh()
    onTraceColorChanged: refresh()
    onThemePulseChanged: pushRules()
    onSettingsChanged: pushRules()
    Component.onCompleted: pushRules()

    readonly property var _rulesWatch: Connections {
        target: root.settings
        function onLogRulesChanged() { root.pushRules() }
    }
}
