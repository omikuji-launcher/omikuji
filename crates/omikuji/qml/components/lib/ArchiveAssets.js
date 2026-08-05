.pragma library

var EXT_RE = /\.(tar\.(gz|xz|zst)|zip)$/

function stems(assets) {
    return assets.map(a => String(a.name).replace(EXT_RE, ""))
}

function labels(assets) {
    var s = stems(assets)
    if (s.length < 2) return s

    var prefix = s[0]
    for (var i = 1; i < s.length; i++) {
        while (prefix.length > 0 && s[i].indexOf(prefix) !== 0)
            prefix = prefix.substring(0, prefix.length - 1)
    }

    var out = s.map(v => {
        var r = v.substring(prefix.length).replace(/^[-_.]+/, "")
        return r === "" ? "x86_64" : r
    })

    var counts = {}
    out.forEach(l => counts[l] = (counts[l] || 0) + 1)
    return out.map((l, i) => {
        if (counts[l] < 2) return l
        var m = String(assets[i].name).match(EXT_RE)
        return m ? l + " · " + m[0].substring(1) : l
    })
}
