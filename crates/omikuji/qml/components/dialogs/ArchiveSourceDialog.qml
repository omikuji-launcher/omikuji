import QtQuick
import QtQuick.Controls
import "../controls"


DialogCard {
    id: root

    property var archiveManager: null
    property string category: "runners"
    property string errorText: ""

    property string nameValue: ""
    property string descValue: ""
    property string kindValue: ""
    property string urlValue: ""

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
    title: category === "runners" ? qsTr("Add runner source") : qsTr("Add translation layer source")

    function show(cat) {
        category = cat
        nameValue = ""
        descValue = ""
        urlValue = ""
        errorText = ""
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
        const err = archiveManager.addSource(category, JSON.stringify({
            name: nameValue.trim(),
            kind: kindValue,
            api_url: normalizedUrl(),
            desc: descValue.trim()
        }))
        if (err && err.length > 0) errorText = err
        else close()
    }

    body: Column {
        width: parent.width
        spacing: theme.space.md

        M3TextField {
            label: qsTr("Name")
            placeholder: root.category === "runners" ? "Wine-GE" : "DXVK-gplasync"
            width: parent.width
            text: root.nameValue
            onTextEdited: (t) => root.nameValue = t
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
            onSelected: (v) => root.kindValue = v
            Component.onCompleted: root.kindValue = currentValue
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
            color: theme.textSubtle
            font.pixelSize: theme.type.micro.size
            font.family: "monospace"
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: qsTr("GitHub and Codeberg repo links are converted to their releases API automatically.")
            color: theme.textSubtle
            font.pixelSize: theme.type.caption.size
            wrapMode: Text.WordWrap
        }

        Text {
            visible: root.errorText !== ""
            width: parent.width
            text: root.errorText
            color: theme.error
            font.pixelSize: theme.type.caption.size
            wrapMode: Text.WordWrap
        }
    }

    actions: Row {
        spacing: theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "tonal"
            onClicked: root.close()
        }
        M3Button {
            text: qsTr("Add")
            variant: "filled"
            enabled: root.nameValue.trim() !== "" && root.urlValue.trim() !== ""
            onClicked: root.submit()
        }
    }
}
