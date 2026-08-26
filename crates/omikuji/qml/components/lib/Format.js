.pragma library

const KB = 1000
const MB = KB * 1000
const GB = MB * 1000

function formatBytes(b) {
    if (b >= GB) return (b / GB).toFixed(2) + " GB"
    if (b >= MB) return (b / MB).toFixed(1) + " MB"
    if (b >= KB) return (b / KB).toFixed(1) + " KB"
    return Math.round(b) + " B"
}

function formatSpeed(b) {
    return formatBytes(b) + "/s"
}

function formatBytesShort(bytes) {
    if (bytes <= 0) return ""
    if (bytes >= GB) return (bytes / GB).toFixed(1) + " GB"
    return (bytes / MB).toFixed(0) + " MB"
}

function formatEta(secs) {
    if (!isFinite(secs) || secs <= 0) return "?"
    if (secs >= 3600) return Math.floor(secs / 3600) + "h " + Math.floor((secs % 3600) / 60) + "m"
    if (secs >= 60) return Math.floor(secs / 60) + "m " + Math.floor(secs % 60) + "s"
    return Math.floor(secs) + "s"
}
