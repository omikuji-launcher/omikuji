pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../lib/Nav.js" as Nav

// step: { keys, title, body, note?, until?, hint?, demo?, centered?, art? }. with until() the holes take clicks and the step ends when it turns true. without it there's a Next button
Item {
    id: root

    anchors.fill: parent

    property var appSettings: null

    property string tourId: ""
    property var steps: []
    property int index: -1

    readonly property bool running: index >= 0
    readonly property var step: running ? steps[index] : null
    readonly property bool isDoStep: step !== null && typeof step.until === "function"
    readonly property string demo: step && step.demo ? step.demo : ""
    readonly property bool _ownsKeys: _open && !lost.value && (spotlight.shown || (step !== null && !isDoStep))
    readonly property bool canBack: index > 0 && typeof steps[index - 1].until !== "function"
        && (!steps[index - 1].demo || steps[index - 1].demo === demo)

    property bool _present: false
    // flipped before the step clears or the closing card chases its anchor to the bottom
    property bool _open: false
    property int _revealedIndex: -1
    property Item _lastTarget: null

    function has(key) {
        return SpotlightTargets.find(key) !== null
    }

    function _done() {
        try { return JSON.parse(appSettings.toursDoneJson()) || [] }
        catch (e) { return [] }
    }

    function isDone(id) {
        return _done().indexOf(id) !== -1
    }

    function start(tour) {
        tourId = tour.tourId
        steps = tour.steps
        _revealedIndex = -1
        index = 0
        _open = true
        OverlayStack.push(root)
    }

    function next() {
        if (!running) return
        if (index + 1 >= steps.length) finish()
        else index++
    }

    function back() {
        if (canBack) index--
    }

    function finish() {
        if (!isDone(tourId)) appSettings.applyToursDoneJson(JSON.stringify(_done().concat([tourId])))
        _open = false
        index = -1
        OverlayStack.pop(root)
    }

    function _bounds(rects) {
        if (rects.length === 0) return null
        const x0 = Math.min(...rects.map(r => r.x)), y0 = Math.min(...rects.map(r => r.y))
        const x1 = Math.max(...rects.map(r => r.x + r.width)), y1 = Math.max(...rects.map(r => r.y + r.height))
        return Qt.rect(x0, y0, x1 - x0, y1 - y0)
    }

    // steps[index] cause inside onIndexChanged step's binding still holds the previous step
    function _checkPresent() {
        const current = index >= 0 ? steps[index] : null
        _present = current !== null && current.keys.every(has)
        if (_present && current.keys.length > 0) _lastTarget = SpotlightTargets.find(current.keys[0]).target
        if (!_present || _revealedIndex === index) return
        _revealedIndex = index
        for (const key of current.keys) _reveal(SpotlightTargets.find(key).target)
    }

    function _reveal(item) {
        for (const flick of Nav.scrollParents(item, null)) {
            const y = Nav.revealY(flick, item, { margin: Theme.space.sm, alignTall: true })
            if (y === null) continue
            scrollGlide.stop()
            scrollGlide.target = flick
            scrollGlide.to = y
            scrollGlide.start()
        }
    }

    NumberAnimation {
        id: scrollGlide
        property: "contentY"
        duration: Theme.dur.slow
        easing.type: Easing.InOutCubic
    }

    function _tick() {
        _checkPresent()
        if (isDoStep && step.until()) next()
    }

    onIndexChanged: _checkPresent()

    Component.onDestruction: OverlayStack.pop(root)

    Timer {
        interval: 100
        repeat: true
        running: root.running
        onTriggered: root._tick()
    }

    Binding {
        target: OverlayStack
        property: "grabbed"
        value: root
        when: root._ownsKeys
    }

    Shortcut {
        sequence: "Escape"
        enabled: root.running && OverlayStack.top === root
        onActivated: root.finish()
    }

    DelayedFlag {
        id: lost
        source: root.running && !root._present
        delay: 600
    }

    readonly property var _cardContent: {
        if (!step) return null
        const stepText = qsTr("%1 of %2").arg(index + 1).arg(steps.length)
        if (lost.value) return {
            stepText: stepText,
            title: step.title,
            body: qsTr("Head back to where you were to pick the tour up again."),
            note: "", hint: "", canBack: false, canNext: false, isLast: false, art: null
        }
        if (!_present) return null
        return {
            stepText: stepText,
            title: step.title,
            body: step.body,
            note: step.note || "",
            hint: isDoStep ? (step.hint || (step.keys.length > 0 ? qsTr("Click the highlighted spot") : "")) : "",
            canBack: canBack,
            canNext: !isDoStep,
            isLast: index === steps.length - 1,
            art: step.art || null
        }
    }

    Spotlight {
        id: spotlight
        keys: root.step ? root.step.keys : []
        shown: root._open && !lost.value && root.step !== null && (keys.length > 0 || root.step.centered === true)
        passThrough: root.isDoStep
        highlighted: InputMode.keyFocus(targetProxy)
    }

    Item {
        id: targetProxy

        readonly property rect _rect: spotlight.targetRects.length > 0 ? spotlight.targetRects[0] : Qt.rect(0, 0, 0, 0)
        readonly property bool navigable: root.isDoStep && root._present && root.step.keys.length > 0

        function navActivate() {
            const marker = SpotlightTargets.find(root.step.keys[0])
            if (marker) marker.activate()
        }

        x: _rect.x
        y: _rect.y
        width: _rect.width
        height: _rect.height
    }

    FocusTrap {
        id: trap
        host: root
        scope: root
        active: root._ownsKeys
        sections: [targetProxy, card]
        // put back focus on the settings page
        returnTo: root.demo === "" ? root._lastTarget : null
        onActiveChanged: if (active) Qt.callLater(root._focusPrimary)
    }

    Keys.onPressed: (event) => trap.handleKey(event)

    function _focusPrimary() {
        if (!trap.active) return
        const item = targetProxy.navigable ? targetProxy : card.nextButton.visible ? card.nextButton : null
        if (item) item.forceActiveFocus(Qt.TabFocusReason)
    }

    TourCard {
        id: card
        shown: root._open
        centered: root.step !== null && root.step.centered === true
        anchorRect: spotlight.shown ? root._bounds(spotlight.targetRects) : null
        content: root._cardContent
        onBackClicked: root.back()
        onNextClicked: root.next()
        onCloseClicked: root.finish()
        onContentShown: Qt.callLater(root._focusPrimary)
        // cant take focus if still doesnt exist
        onVisibleChanged: if (visible) Qt.callLater(root._focusPrimary)
    }
}
