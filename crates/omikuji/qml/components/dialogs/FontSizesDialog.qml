pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"

DialogCard {
    id: root

    property var appSettings: null

    title: qsTr("Font sizes")
    maxWidth: 420

    onCloseRequested: close()

    readonly property var roles: [
        { key: "display",  name: qsTr("Display") },
        { key: "headline", name: qsTr("Headline") },
        { key: "title",    name: qsTr("Title") },
        { key: "subtitle", name: qsTr("Subtitle") },
        { key: "body",     name: qsTr("Body") },
        { key: "label",    name: qsTr("Label") },
        { key: "caption",  name: qsTr("Caption") },
        { key: "micro",    name: qsTr("Micro") }
    ]

    function applySize(key, px) {
        let m = {}
        try { m = JSON.parse(appSettings.fontSizesJson()) } catch (e) {}
        if (px === Theme.fontDefaults[key]) delete m[key]
        else m[key] = px
        appSettings.applyFontSizesJson(JSON.stringify(m))
    }

    body: Column {
        width: parent.width
        spacing: Theme.space.sm

        Repeater {
            model: root.roles

            Item {
                id: row
                required property var modelData
                readonly property var roleType: Theme.type[modelData.key]
                width: parent.width
                height: Math.max(36, roleLabel.implicitHeight)

                onRoleTypeChanged: spin.value = roleType.size

                Text {
                    id: roleLabel
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    text: row.modelData.name
                    color: Theme.text
                    font.pixelSize: row.roleType.size
                    font.weight: row.roleType.weight
                }

                M3SpinBox {
                    id: spin
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    from: 8
                    to: 40
                    Component.onCompleted: value = row.roleType.size
                    onMoved: (v) => root.applySize(row.modelData.key, v)
                }
            }
        }
    }

    footerLeft: M3Button {
        text: qsTr("Reset all")
        variant: "tonal"
        onClicked: root.appSettings.applyFontSizesJson("{}")
    }

    actions: M3Button {
        text: qsTr("Done")
        variant: "filled"
        onClicked: root.close()
    }
}
