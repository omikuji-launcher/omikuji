pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"


DialogCard {
    id: root

    property var archiveManager: null
    property string category: "runners"

    property string nameValue: ""
    property string descValue: ""
    property string kindValue: ""
    property string urlValue: ""
    property string priorityValue: ""

    property bool editing: false

    readonly property var kindOptions: category === "runners"
        ? [
            { label: "Proton", value: "proton" },
            { label: "Wine", value: "wine" }
        ]
        : [
            { label: "DXVK", value: "dxvk" },
            { label: "VKD3D", value: "vkd3d" },
            { label: "DXVK-NVAPI", value: "dxvk_nvapi" },
            { label: qsTr("Other"), value: "other" }
        ]

    maxWidth: 480
    title: {
        if (editing) return qsTr("Edit %1").arg(nameValue)
        return category === "runners" ? qsTr("Add runner source") : qsTr("Add translation layer source")
    }

    function show(cat) {
        category = cat
        editing = false
        nameValue = ""
        descValue = ""
        urlValue = ""
        priorityValue = ""
        kindValue = kindOptions[0].value
        root.errorText = ""
        open()
    }

    function showEdit(cat, source) {
        category = cat
        editing = true
        nameValue = source.name || ""
        descValue = source.desc || ""
        kindValue = source.kind || ""
        urlValue = source.api_url || ""
        priorityValue = (source.asset_priority || []).join(" ")
        root.errorText = ""
        open()
    }

    onCloseRequested: close()

    function normalizedUrl() {
        const u = urlValue.trim()
        let m = u.match(/^https?:\/\/github\.com\/([^\/]+)\/([^\/]+?)(?:\.git)?(?:\/(?:releases|tags)(?:\/.*)?)?\/?$/)
        if (m) return "https://api.github.com/repos/" + m[1] + "/" + m[2] + "/releases"
        m = u.match(/^https?:\/\/codeberg\.org\/([^\/]+)\/([^\/]+?)(?:\.git)?(?:\/(?:releases|tags)(?:\/.*)?)?\/?$/)
        if (m) return "https://codeberg.org/api/v1/repos/" + m[1] + "/" + m[2] + "/releases"
        return u
    }

    function submit() {
        const payload = JSON.stringify({
            name: nameValue.trim(),
            kind: kindValue,
            api_url: normalizedUrl(),
            desc: descValue.trim(),
            asset_priority: priorityValue.trim().split(/\s+/).filter(p => p.length > 0)
        })
        const err = editing
            ? archiveManager.updateSource(category, nameValue.trim(), payload)
            : archiveManager.addSource(category, payload)
        if (err && err.length > 0) root.errorText = err
        else close()
    }

    body: Column {
        width: parent.width
        spacing: Theme.space.md

        M3TextField {
            label: qsTr("Name")
            placeholder: root.category === "runners" ? "Wine-GE" : "DXVK-gplasync"
            width: parent.width
            text: root.nameValue
            readOnly: root.editing
            onTextEdited: (t) => root.nameValue = t
        }

        Text {
            width: parent.width
            visible: root.editing
            text: qsTr("The name identifies installed versions on disk, so it can't be changed here.")
            color: Theme.textSubtle
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        M3TextField {
            label: qsTr("Description")
            placeholder: qsTr("optional")
            width: parent.width
            text: root.descValue
            onTextEdited: (t) => root.descValue = t
        }

        M3Dropdown {
            label: qsTr("Kind")
            width: parent.width
            options: root.kindOptions
            currentIndex: Math.max(0, root.kindOptions.findIndex(o => o.value === root.kindValue))
            onSelected: (v) => root.kindValue = v
        }

        M3TextField {
            label: qsTr("Releases URL")
            placeholder: "https://github.com/owner/repo"
            width: parent.width
            text: root.urlValue
            onTextEdited: (t) => root.urlValue = t
        }

        Text {
            visible: root.urlValue.trim() !== "" && root.normalizedUrl() !== root.urlValue.trim()
            width: parent.width
            text: root.normalizedUrl()
            color: Theme.textSubtle
            font.pixelSize: Theme.type.micro.size
            font.family: "monospace"
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: qsTr("GitHub and Codeberg repo links are converted to their releases API automatically.")
            color: Theme.textSubtle
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        M3TextField {
            visible: root.category === "runners"
            label: qsTr("Latest build priority")
            placeholder: "_v3 _x86_64_v3"
            width: parent.width
            text: root.priorityValue
            onTextEdited: (t) => root.priorityValue = t
        }

        Text {
            width: parent.width
            visible: root.category === "runners"
            text: qsTr("Space separated. When a release has several builds, the first one that matches is used, so entries further right have lower priority. If none match, the normal pick is used.")
            color: Theme.textSubtle
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.WordWrap
        }
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "tonal"
            onClicked: root.close()
        }
        M3Button {
            text: root.editing ? qsTr("Save") : qsTr("Add")
            variant: "filled"
            enabled: root.nameValue.trim() !== "" && root.urlValue.trim() !== ""
            onClicked: root.submit()
        }
    }
}
