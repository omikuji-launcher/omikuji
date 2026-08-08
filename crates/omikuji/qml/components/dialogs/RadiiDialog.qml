pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import "../controls"
import "../primitives"

DialogCard {
    id: root

    property var uiSettings: null

    title: qsTr("Corner radius")
    maxWidth: 420

    onCloseRequested: close()

    readonly property var roles: [
        { key: "xs",   name: qsTr("Extra small") },
        { key: "sm",   name: qsTr("Small") },
        { key: "md",   name: qsTr("Medium") },
        { key: "lg",   name: qsTr("Large") },
        { key: "xl",   name: qsTr("Extra large") },
        { key: "xxl",  name: qsTr("Huge") },
        { key: "pill", name: qsTr("Pill") }
    ]

    function defaultFor(key) {
        return key === "pill"
            ? Theme.radiusDefaults[key]
            : Math.round(Theme.radiusDefaults[key] * Theme.radiusScale)
    }

    function applyRadius(key, px) {
        let m = {}
        try { m = JSON.parse(uiSettings.radiusOverridesJson()) } catch (e) {}
        if (px === defaultFor(key)) delete m[key]
        else m[key] = px
        uiSettings.applyRadiusOverridesJson(JSON.stringify(m))
    }

    body: Column {
        width: parent.width
        spacing: Theme.space.sm

        Repeater {
            model: root.roles

            Item {
                id: row
                required property var modelData
                readonly property int roleRadius: Theme.radius[modelData.key]
                width: parent.width
                height: 36

                onRoleRadiusChanged: spin.value = roleRadius

                Squircle {
                    id: swatch
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: 48
                    height: 30
                    radius: row.roleRadius
                    fillColor: Theme.alpha(Theme.accent, 0.25)
                }

                Text {
                    anchors.left: swatch.right
                    anchors.leftMargin: Theme.space.md
                    anchors.verticalCenter: parent.verticalCenter
                    text: row.modelData.name
                    color: Theme.text
                    font.pixelSize: Theme.type.body.size
                }

                M3SpinBox {
                    id: spin
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    from: 0
                    to: 999
                    Component.onCompleted: value = row.roleRadius
                    onMoved: (v) => root.applyRadius(row.modelData.key, v)
                }
            }
        }
    }

    footerLeft: M3Button {
        text: qsTr("Reset all")
        variant: "tonal"
        onClicked: root.uiSettings.applyRadiusOverridesJson("{}")
    }

    actions: M3Button {
        text: qsTr("Done")
        variant: "filled"
        onClicked: root.close()
    }
}
