pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

LogHighlighter {
    id: root

    property var settings: null

    active: settings ? settings.highlightLogs : true
    errorColor: Theme.error
    warnColor: Theme.warning
    fixmeColor: Theme.accent
    traceColor: Theme.textSubtle

    readonly property string themePulse: [Theme.accent, Theme.error, Theme.warning, Theme.success, Theme.text, Theme.textMuted, Theme.textSubtle].join("")

    function pushRules() {
        if (!settings) return
        let rules = []
        try { rules = JSON.parse(settings.logRulesJson()) } catch (e) {}
        const resolved = rules.map(r => ({ pattern: r.pattern, color: Theme.resolveColor(r.color) }))
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
