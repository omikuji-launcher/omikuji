import QtQuick
import QtQuick.Layouts
import "../controls"
import "../primitives"

DialogCard {
    sizeKey: "log_rules"
    id: root

    property var uiSettings: null

    title: qsTr("Log highlight colors")
    maxWidth: 620

    onCloseRequested: close()

    function show() {
        rulesModel.clear()
        let rules = []
        try { rules = JSON.parse(uiSettings.logRulesJson()) } catch (e) {}
        for (const r of rules) rulesModel.append({ pattern: r.pattern || "", colorValue: r.color || "" })
        open()
    }

    function saveRules() {
        const out = []
        for (let i = 0; i < rulesModel.count; i++) {
            const r = rulesModel.get(i)
            if (r.pattern.length > 0) out.push({ pattern: r.pattern, color: r.colorValue })
        }
        uiSettings.applyLogRulesJson(JSON.stringify(out))
        close()
    }

    function swatchColor(value) {
        const resolved = theme.resolveColor(value)
        return /^#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/.test(resolved) ? resolved : "transparent"
    }

    ListModel { id: rulesModel }

    body: Column {
        width: parent.width
        spacing: theme.space.md

        Text {
            width: parent.width
            wrapMode: Text.Wrap
            text: qsTr("Lines matching a pattern (regex) get its color. Rules run before the built-in error/fixme/warning matching. You can use theme tokens such as 'accent', 'error', or hex values like '#7aa2f7'.")
            color: theme.textMuted
            font.pixelSize: theme.type.caption.size
        }

        Repeater {
            model: rulesModel

            RowLayout {
                width: parent.width
                spacing: theme.space.sm

                M3TextField {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 3
                    placeholder: qsTr("pattern")
                    text: pattern
                    onTextEdited: (t) => rulesModel.setProperty(index, "pattern", t)
                }

                M3TextField {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 2
                    placeholder: qsTr("color")
                    text: colorValue
                    onTextEdited: (t) => rulesModel.setProperty(index, "colorValue", t)
                }

                Rectangle {
                    Layout.alignment: Qt.AlignVCenter
                    width: 22
                    height: 22
                    radius: theme.radius.xs
                    color: root.swatchColor(colorValue)
                    border.width: 1
                    border.color: theme.outline
                }

                IconButton {
                    Layout.alignment: Qt.AlignVCenter
                    icon: "close"
                    size: 24
                    danger: true
                    onClicked: rulesModel.remove(index)
                }
            }
        }

        Text {
            visible: rulesModel.count === 0
            text: qsTr("No custom rules yet.")
            color: theme.textSubtle
            font.pixelSize: theme.type.body.size
        }
    }

    footerLeft: M3Button {
        small: true
        variant: "tonal"
        text: qsTr("Add rule")
        onClicked: rulesModel.append({ pattern: "", colorValue: "" })
    }

    actions: Row {
        spacing: theme.space.sm

        M3Button {
            variant: "text"
            text: qsTr("Cancel")
            onClicked: root.close()
        }

        M3Button {
            variant: "filled"
            text: qsTr("Save")
            onClicked: root.saveRules()
        }
    }
}
