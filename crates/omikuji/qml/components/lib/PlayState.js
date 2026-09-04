.pragma library

const Play = 0
const Stop = 1
const Activity = 2
const Launching = 3

function stateFor(launching, running, activity) {
    if (launching) return Launching
    if (running) return Stop
    if (activity) return Activity
    return Play
}
