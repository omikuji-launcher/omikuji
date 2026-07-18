.pragma library

function parseVersion(name) {
    var matches = name.match(/\d+/g) || []
    var out = []
    for (var i = 0; i < matches.length; i++) out.push(parseInt(matches[i], 10))
    return out
}

function compareDesc(a, b) {
    var va = parseVersion(a.value)
    var vb = parseVersion(b.value)
    var n = Math.max(va.length, vb.length)
    for (var i = 0; i < n; i++) {
        var ai = va[i] || 0
        var bi = vb[i] || 0
        if (ai !== bi) return bi - ai
    }
    return a.value < b.value ? 1 : (a.value > b.value ? -1 : 0)
}

function displayLabel(value, name) {
    if (value === "system") return "System Wine"
    if (value.indexOf("system:") === 0) return value.substring(7) + " (System)"
    if (value.indexOf("steam:") === 0) return (name || value.substring(6)) + " (Steam)"
    return name || value
}

function isProton(name) {
    return String(name).toLowerCase().indexOf("proton") !== -1
}

function groupRunners(rawList, opts) {
    opts = opts || {}
    var proton = []
    var wine = []
    for (var i = 0; i < rawList.length; i++) {
        var raw = rawList[i]
        var value = Array.isArray(raw) ? raw[0] : raw
        var name = Array.isArray(raw) ? raw[1] : ""
        var kind = Array.isArray(raw) && raw.length > 2 ? raw[2] : ""
        var entry = { label: displayLabel(value, name), value: value }
        if (kind === "proton" || (kind === "" && isProton(value))) proton.push(entry)
        else wine.push(entry)
    }
    proton.sort(compareDesc)
    wine.sort(compareDesc)

    var out = []
    if (opts.includeSystemDefault) {
        out.push({ label: opts.defaultLabel || "System default", value: "" })
    }
    if (proton.length > 0) {
        out.push({ header: true, label: "Proton" })
        for (var j = 0; j < proton.length; j++) out.push(proton[j])
    }
    if (wine.length > 0) {
        out.push({ header: true, label: "Wine" })
        for (var k = 0; k < wine.length; k++) out.push(wine[k])
    }
    return out
}

function indexOfValue(options, value) {
    for (var i = 0; i < options.length; i++) {
        if (options[i].header) continue
        if (options[i].value === value) return i
    }
    return -1
}

function firstNonHeader(options) {
    for (var i = 0; i < options.length; i++) {
        if (!options[i].header) return i
    }
    return -1
}

// shhhh nobody will notice you living here
function runnerBucket(runnerType) {
    var t = String(runnerType || "")
    if (t === "steam" || t === "flatpak" || t === "native") return t
    return "wine"
}

function pickPreferred(options, substrings) {
    if (!substrings || substrings.length === 0) return firstNonHeader(options)
    for (var s = 0; s < substrings.length; s++) {
        var needle = String(substrings[s] || "").toLowerCase()
        if (needle === "") continue
        for (var i = 0; i < options.length; i++) {
            if (options[i].header) continue
            if (options[i].value.toLowerCase().indexOf(needle) !== -1) return i
        }
    }
    return firstNonHeader(options)
}

function preferredIndex(options, defaultValue, fallbackSubstrings) {
    var idx = indexOfValue(options, defaultValue || "")
    if (idx >= 0) return idx
    return pickPreferred(options, fallbackSubstrings)
}
