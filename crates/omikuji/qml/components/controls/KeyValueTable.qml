pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts

Item {
    id: root

    property string json: "{}"
    property string keyPlaceholder: "KEY"
    property string valuePlaceholder: "value"
    property string addLabel: "Add variable"
    property var gameModel: null
    property bool expandHint: true
    property alias actions: actionRow.data

    signal changed(string json)

    implicitHeight: col.implicitHeight

    onJsonChanged: _syncFromJson()
    Component.onCompleted: _syncFromJson()

    ListModel { id: listModel }

    // skip if rows already match, parent echoes would clobber a half-typed new key
    function _syncFromJson() {
        let obj = {}
        try { obj = JSON.parse(root.json || "{}") } catch (e) {}
        if (_modelEquals(obj)) return
        listModel.clear()
        for (let k of Object.keys(obj)) {
            listModel.append({ k: String(k), v: String(obj[k]) })
        }
    }

    function _rowsObject() {
        let obj = {}
        for (let i = 0; i < listModel.count; ++i) {
            let row = listModel.get(i)
            let k = (row.k || "").trim()
            if (k === "" || k in obj) continue
            obj[k] = String(row.v || "")
        }
        return obj
    }

    function _modelEquals(obj) {
        let rows = _rowsObject()
        let keys = Object.keys(rows)
        if (keys.length !== Object.keys(obj).length) return false
        return keys.every(k => k in obj && String(obj[k]) === rows[k])
    }

    function _emit() {
        root.changed(JSON.stringify(_rowsObject()))
    }

    function _addRow() {
        listModel.append({ k: "", v: "" })
    }

    function _removeRow(i) {
        listModel.remove(i, 1)
        _emit()
    }

    ColumnLayout {
        id: col
        width: parent.width
        spacing: 8

        Repeater {
            model: listModel
            delegate: RowLayout {
                id: rowItem
                required property int index
                required property string k
                required property string v
                Layout.fillWidth: true
                spacing: 8

                M3TextField {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 1
                    Layout.alignment: Qt.AlignTop
                    placeholder: root.keyPlaceholder
                    text: rowItem.k
                    onTextEdited: (t) => {
                        listModel.setProperty(rowItem.index, "k", t)
                        root._emit()
                    }
                }

                Text {
                    text: "="
                    color: Theme.textSubtle
                    font.pixelSize: Theme.type.body.size
                    Layout.alignment: Qt.AlignTop
                    Layout.topMargin: 14
                }

                M3TextField {
                    Layout.fillWidth: true
                    Layout.preferredWidth: 2
                    Layout.alignment: Qt.AlignTop
                    placeholder: root.valuePlaceholder
                    text: rowItem.v
                    gameModel: root.gameModel
                    expandHint: root.expandHint
                    onTextEdited: (t) => {
                        listModel.setProperty(rowItem.index, "v", t)
                        root._emit()
                    }
                }

                IconButton {
                    icon: "close"
                    size: 32
                    danger: true
                    Layout.alignment: Qt.AlignTop
                    Layout.topMargin: 6
                    onClicked: root._removeRow(rowItem.index)
                }
            }
        }

        Row {
            spacing: Theme.space.sm
            Layout.topMargin: Theme.space.xs

            M3Button {
                text: root.addLabel
                variant: "tonal"
                icon: "add"
                onClicked: root._addRow()
            }

            Row {
                id: actionRow
                spacing: Theme.space.sm
            }
        }
    }
}
