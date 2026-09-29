.pragma library

function folderName(name) {
    // i firmly believe not a single fucking game world wide has only these characters but you may never know
    return (name || "").replace(/[\\/:*?"<>|]/g, "").trim() || "Game"
}

function join(base, name) {
    const root = (base || "").trim().replace(/\/+$/, "")
    return root === "" ? "" : root + "/" + folderName(name)
}
