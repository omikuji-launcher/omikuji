.pragma library

function folderName(name) {
    // i firmly believe not a single fucking game world wide has only these characters but you may never know
    return (name || "").replace(/[\\/:*?"<>|]/g, "").trim() || "Game"
}

function join(base, name) {
    const root = (base || "").trim().replace(/\/+$/, "")
    return root === "" ? "" : root + "/" + folderName(name)
}

function freeBytes(gameModel, path) {
    const bytes = parseInt(gameModel.disk_free_space(path))
    return isNaN(bytes) ? -1 : bytes
}

function hasSpace(freeBytes, neededBytes, resuming) {
    if (freeBytes < 0 || neededBytes < 0) return false
    return resuming || freeBytes >= neededBytes
}
