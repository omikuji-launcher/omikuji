import QtQuick
import omikuji 1.0

// empty key = not a target
QtObject {
    id: marker

    required property string key
    required property Item target
    // below 0 = spotlight default
    property real padding: -1
    // kb/controller press when navActivate isn't the right one
    property var activate: () => {
        if (typeof marker.target.navActivate === "function") marker.target.navActivate()
    }

    property string _registered: ""

    function _sync() {
        if (_registered !== "") SpotlightTargets.unregister(_registered, marker)
        _registered = key
        if (key !== "") SpotlightTargets.register(key, marker)
    }

    onKeyChanged: _sync()
    Component.onCompleted: _sync()
    Component.onDestruction: if (_registered !== "") SpotlightTargets.unregister(_registered, marker)
}
