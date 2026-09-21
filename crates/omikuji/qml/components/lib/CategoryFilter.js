.pragma library
.import "RunnerGrouping.js" as RG

var RECENT_LIMIT = 10

function nameMatches(name, search) {
    var q = (search || "").trim().toLowerCase()
    return q === "" || (name || "").toLowerCase().indexOf(q) !== -1
}

function hiddenSplit(kind, hidden, showHidden) {
    if (showHidden) return false
    return kind === "hidden" ? !hidden : hidden
}

function cardVerdict(kind, hidden, favourite) {
    switch (kind) {
    case "all":       return true
    case "favourite": return favourite === true
    case "hidden":    return hidden === true
    }
    return null
}

function gameVerdict(kind, value, game, recentIds) {
    switch (kind) {
    case "recent": return recentIds[game.gameId] === true
    case "runner": return RG.runnerBucket(game.runnerType) === value
    case "tag":    return tagsOf(game).indexOf(value) !== -1
    default:       return true
    }
}

function tagsOf(game) {
    try { return JSON.parse(game.categories || "[]") } catch (e) { return [] }
}

function computeRecent(model) {
    var dated = []
    if (!model) return ({})
    for (var i = 0; i < model.count; i++) {
        var g = model.get_game(i)
        if (!g) continue
        var ts = Date.parse(g.lastPlayed || "") || 0
        if (ts > 0) dated.push({ id: g.gameId, ts: ts })
    }
    dated.sort(function (a, b) { return b.ts - a.ts })
    var out = {}
    for (var j = 0; j < Math.min(RECENT_LIMIT, dated.length); j++) out[dated[j].id] = true
    return out
}
