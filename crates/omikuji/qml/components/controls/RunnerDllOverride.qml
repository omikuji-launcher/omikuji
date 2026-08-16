pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0

Column {
    id: root

    property var archiveManager: null
    property string runnerDir: ""
    property var options: ({})

    signal errorRaised(string message)

    readonly property var _kinds: [
        { kind: "dxvk", label: "DXVK" },
        { kind: "vkd3d", label: "VKD3D" },
        { kind: "dxvk_nvapi", label: "DXVK-NVAPI" }
    ]

    property var _supported: ({})
    property var _active: ({})

    readonly property bool anySupported: {
        for (var k in _supported) if (_supported[k]) return true
        return false
    }

    spacing: Theme.space.xs

    onRunnerDirChanged: refresh()

    function refresh() {
        if (!archiveManager || runnerDir === "") {
            _supported = ({})
            _active = ({})
            return
        }
        try {
            var s = JSON.parse(archiveManager.runnerDllStatus(runnerDir))
            _supported = s.supported || ({})
            _active = s.active || ({})
        } catch (e) {
            _supported = ({})
            _active = ({})
        }
    }

    Repeater {
        model: root._kinds

        delegate: Item {
            id: kindRow
            required property var modelData
            readonly property string kind: modelData.kind
            readonly property bool supported: root._supported[kind] === true
            readonly property string activeTag: root._active[kind] || ""
            readonly property var tags: root.options[kind] || []
            readonly property var opts: [{ label: qsTr("Default"), value: "" }].concat(
                tags.map(t => ({ label: t, value: t })))
            readonly property int activeIndex: {
                for (var i = 0; i < opts.length; i++)
                    if (opts[i].value === activeTag) return i
                return 0
            }

            visible: supported
            width: parent ? parent.width : 0
            height: supported ? 48 : 0

            onActiveIndexChanged: dd.currentIndex = activeIndex

            Text {
                id: kindLabel
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                width: 110
                text: kindRow.modelData.label
                color: Theme.text
                font.pixelSize: Theme.type.body.size
                font.weight: Font.Medium
            }

            M3Dropdown {
                id: dd
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                width: 200
                fieldHeight: 40
                options: kindRow.opts
                currentIndex: kindRow.activeIndex
                onSelected: (v) => {
                    var err = root.archiveManager.setRunnerDllOverride(root.runnerDir, kindRow.kind, v)
                    if (err && err.length > 0) root.errorRaised(err)
                    root.refresh()
                    dd.currentIndex = kindRow.activeIndex
                }
            }
        }
    }

    Text {
        visible: !root.anySupported
        width: parent.width
        text: qsTr("This runner has no overridable Proton DLLs.")
        color: Theme.textSubtle
        font.pixelSize: Theme.type.caption.size
        wrapMode: Text.WordWrap
    }
}
