.pragma library

var TIERS = {
    daikichi: { kanji: "大吉" },
    kichi:    { kanji: "吉"   },
    chukichi: { kanji: "中吉" },
    shokichi: { kanji: "小吉" },
    suekichi: { kanji: "末吉" },
    kyo:      { kanji: "凶"   }
}

var BOUNCED_HOURS = 0.5
var WELL_WORN_HOURS = 100
var DAY_MS = 86400000

function daysSince(iso) {
    var ts = Date.parse(iso || "")
    return ts ? (Date.now() - ts) / DAY_MS : -1
}

function readingFor(game) {
    var hours = Number(game && game.playtime) || 0
    var days = daysSince(game && game.lastPlayed)

    if (hours <= 0 && days < 0) return { tier: "daikichi", reason: "untrodden" }
    if (hours < BOUNCED_HOURS && days >= 0) return { tier: "kyo", reason: "turnedBack" }
    if (days < 0) return { tier: "kichi", reason: "noRecord" }
    if (days >= 365) return { tier: "chukichi", reason: "aYearGone" }
    if (hours >= WELL_WORN_HOURS) return { tier: "suekichi", reason: "wellWorn" }
    if (days >= 90) return { tier: "kichi", reason: "seasonsPassed" }
    if (days >= 14) return { tier: "shokichi", reason: "awhileBack" }
    return { tier: "suekichi", reason: "justHere" }
}

function pick(pool) {
    if (!pool || pool.length === 0) return -1
    return pool[Math.floor(Math.random() * pool.length)]
}
