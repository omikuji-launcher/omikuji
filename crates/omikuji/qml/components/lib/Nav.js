.pragma library

const MISALIGNED_PENALTY = 1000

function isActivate(event) {
    return !event.isAutoRepeat
        && (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space)
}

function tabCycleStep(event) {
    if (!(event.modifiers & Qt.ControlModifier)) return 0
    if (event.key === Qt.Key_Backtab) return -1
    if (event.key === Qt.Key_Tab) return (event.modifiers & Qt.ShiftModifier) ? -1 : 1
    return 0
}

function contains(scope, item) {
    for (let cur = item; cur; cur = cur.parent) {
        if (cur === scope) return true
    }
    return false
}

function navigableOwner(item, stopAt) {
    for (let cur = item; cur && cur !== stopAt; cur = cur.parent) {
        if (cur.navigable === true) return cur
    }
    return null
}

function _collectInto(item, out) {
    const kids = item.children
    for (let i = 0; i < kids.length; i++) {
        const c = kids[i]
        if (!c.visible || !c.enabled || c.opacity === 0) continue
        if (c.navigable === true) out.push(c)
        else _collectInto(c, out)
    }
}

function collect(item) {
    if (!item || !item.visible || !item.enabled) return []
    if (item.navigable === true) return [item]
    const out = []
    _collectInto(item, out)
    return out
}

function isReachable(scope, item) {
    return !!item && collect(scope).indexOf(item) !== -1
}

function _focus(item) {
    item.forceActiveFocus(Qt.TabFocusReason)
    if (item.navFocusTarget) item.navFocusTarget.forceActiveFocus(Qt.TabFocusReason)
    return item
}

function focusFirst(section) {
    const items = collect(section)
    return items.length > 0 ? _focus(items[0]) : null
}

function _isFlickable(item) {
    return typeof item.contentY === "number" && item.contentItem !== undefined && typeof item.flick === "function"
}

function _scrollParents(item, stopAt) {
    const out = []
    for (let cur = item ? item.parent : null; cur && cur !== stopAt; cur = cur.parent) {
        if (_isFlickable(cur)) out.push(cur)
    }
    return out
}

function _itemRect(item) {
    return item.mapToItem(null, 0, 0, item.width, item.height)
}

function sourceRect(item) {
    const inner = typeof item.navRectItem === "function" ? item.navRectItem() : null
    return _itemRect(inner || item)
}

function _clip(a, b) {
    const x = Math.max(a.x, b.x)
    const y = Math.max(a.y, b.y)
    const right = Math.min(a.x + a.width, b.x + b.width)
    const bottom = Math.min(a.y + a.height, b.y + b.height)
    return right > x && bottom > y ? { x: x, y: y, width: right - x, height: bottom - y } : null
}

function _shownRect(item, container) {
    return _scrollParents(item, container).reduce((r, f) => r && _clip(r, _itemRect(f)), _itemRect(item))
}

function _gap(a0, a1, b0, b1) {
    return Math.max(0, b0 - a1, a0 - b1)
}

function _score(a, b, dx, dy) {
    const beyond = dx > 0 ? b.x >= a.x + a.width / 2
        : dx < 0 ? b.x + b.width <= a.x + a.width / 2
        : dy > 0 ? b.y >= a.y + a.height / 2
        : b.y + b.height <= a.y + a.height / 2
    if (!beyond) return Infinity
    const primary = dx > 0 ? b.x - (a.x + a.width)
        : dx < 0 ? a.x - (b.x + b.width)
        : dy > 0 ? b.y - (a.y + a.height)
        : a.y - (b.y + b.height)
    const cross = dx !== 0
        ? _gap(a.y, a.y + a.height, b.y, b.y + b.height)
        : _gap(a.x, a.x + a.width, b.x, b.x + b.width)
    const drift = dx !== 0
        ? Math.abs((b.y + b.height / 2) - (a.y + a.height / 2))
        : Math.abs((b.x + b.width / 2) - (a.x + a.width / 2))
    return Math.max(0, primary) + (cross > 0 ? MISALIGNED_PENALTY + cross * 4 : 0) + drift * 0.01
}

function _targetsOf(item) {
    return typeof item.navTargets === "function" ? item.navTargets() : [item]
}

function _best(from, candidates, container, dx, dy) {
    let best = null
    let bestScore = Infinity
    for (const c of candidates) {
        for (const target of _targetsOf(c)) {
            const rect = _shownRect(target, container)
            const s = rect ? _score(from, rect, dx, dy) : Infinity
            if (s < bestScore) {
                bestScore = s
                best = { item: c, target: target }
            }
        }
    }
    return best
}

function _enter(items, focusItem) {
    const inside = items.find(it => contains(focusItem, it))
    return inside ? _focus(inside) : (items.length > 0 ? _focus(items[0]) : null)
}

function spatialMove(scope, focusItem, dx, dy, origin) {
    const items = collect(scope)
    const cur = items.findIndex(it => contains(it, focusItem))
    if (cur === -1 && !origin) return _enter(items, focusItem)
    const from = cur === -1 ? origin : sourceRect(items[cur])
    const others = items.filter((_, i) => i !== cur)
    const containers = (cur === -1 ? [] : _scrollParents(items[cur], scope)).concat([scope])
    for (const container of containers) {
        const best = _best(from, others.filter(it => contains(container, it)), container, dx, dy)
        if (!best) continue
        if (best.target !== best.item) best.item.navEnter(best.target)
        return _focus(best.item)
    }
    return null
}

function _depth(item) {
    let d = 0
    for (let cur = item; cur; cur = cur.parent) d++
    return d
}

function _innermostSection(sections, focusItem) {
    let best = -1
    for (let i = 0; i < sections.length; i++) {
        if (!sections[i] || !contains(sections[i], focusItem)) continue
        if (best === -1 || _depth(sections[i]) > _depth(sections[best])) best = i
    }
    return best
}

function sectionStep(sections, focusItem, forward) {
    const n = sections.length
    const cur = focusItem ? _innermostSection(sections, focusItem) : -1
    for (let s = 1; s <= n; s++) {
        const idx = cur === -1
            ? (forward ? s - 1 : n - s)
            : ((cur + (forward ? s : -s)) % n + n) % n
        if (idx === cur) continue
        const entered = focusFirst(sections[idx])
        if (entered) return entered
    }
    return null
}

function _activate(scope, focusItem) {
    const owner = navigableOwner(focusItem, scope)
    if (!owner || typeof owner.navActivate !== "function") return false
    owner.navActivate()
    return true
}

function hostKey(event, scope, sections, focusItem, origin) {
    if (event.modifiers & Qt.ControlModifier) return false
    if (isActivate(event)) return _activate(scope, focusItem)
    switch (event.key) {
    case Qt.Key_Tab: sectionStep(sections, focusItem, true); return true
    case Qt.Key_Backtab: sectionStep(sections, focusItem, false); return true
    case Qt.Key_Left: spatialMove(scope, focusItem, -1, 0, origin); return true
    case Qt.Key_Right: spatialMove(scope, focusItem, 1, 0, origin); return true
    case Qt.Key_Up: spatialMove(scope, focusItem, 0, -1, origin); return true
    case Qt.Key_Down: spatialMove(scope, focusItem, 0, 1, origin); return true
    }
    return false
}

function ensureVisible(flick, item, margin) {
    if (!flick || !item || !contains(flick.contentItem, item)) return
    const pad = margin === undefined ? 8 : margin
    const r = item.mapToItem(flick.contentItem, 0, 0, item.width, item.height)
    if (r.height > flick.height) return
    const minY = flick.originY
    const maxY = Math.max(minY, minY + flick.contentHeight - flick.height)
    if (r.y < flick.contentY)
        flick.contentY = Math.max(minY, r.y - pad)
    else if (r.y + r.height > flick.contentY + flick.height)
        flick.contentY = Math.min(maxY, r.y + r.height - flick.height + pad)
}

function scrollParent(item) {
    for (let cur = item ? item.parent : null; cur; cur = cur.parent) {
        if (_isFlickable(cur)) return cur
    }
    return null
}

function revealAncestors(item, stopAt) {
    const owner = navigableOwner(item, stopAt)
    for (let cur = owner ? owner.parent : null; cur && cur !== stopAt; cur = cur.parent) {
        if (_isFlickable(cur)) ensureVisible(cur, owner)
    }
}

function focusLost(scope, focusItem, lastInside) {
    if (!focusItem) return false
    if (!contains(scope, focusItem)) return true
    return !!lastInside && !isReachable(scope, lastInside)
}

function guardReturn(scope, lastInside) {
    const fallback = lastInside && typeof lastInside.navFallback === "function" ? lastInside.navFallback() : null
    const target = [lastInside, fallback].find(it => isReachable(scope, it))
    if (target) _focus(target)
    else scope.forceActiveFocus()
}
