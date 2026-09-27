pragma Singleton
import QtQuick

// a key can be registered in several places at once (add-game and game settings share tabs) so find() returns the one on screen
QtObject {
    property var _markers: ({})

    function register(key, marker) {
        _markers[key] = (_markers[key] || []).concat([marker])
    }

    function unregister(key, marker) {
        const list = (_markers[key] || []).filter(m => m && m !== marker)
        if (list.length > 0) _markers[key] = list
        else delete _markers[key]
    }

    function find(key) {
        return (_markers[key] || []).find(m => {
            const it = m ? m.target : null
            return it && it.visible && it.width > 0 && it.height > 0
        }) || null
    }
}
