import QtQuick
import QtQuick.Layouts
import "../primitives"

Item {
    id: root

    property string json: "{}"
    property string keyPlaceholder: "KEY"
    property string valuePlaceholder: "value"
    property string addLabel: "Add variable"
    property var gameModel: null
    property bool expandHint: true

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
        let keys = Object.keys(obj).sort()
        for (let k of keys) {
            listModel.append({ k: String(k), v: String(obj[k]) })
        }
    }

    function _modelEquals(obj) {
        let filtered = []
        for (let i = 0; i < listModel.count; ++i) {
            let row = listModel.get(i)
            let k = (row.k || "").trim()
            if (k === "") continue
            filtered.push([k, String(row.v || "")])
        }
        let objKeys = Object.keys(obj)
        if (filtered.length !== objKeys.length) return false
        for (let pair of filtered) {
            if (!(pair[0] in obj) || String(obj[pair[0]]) !== pair[1]) return false
        }
        return true
    }

    function _emit() {
        let obj = {}
        for (let i = 0; i < listModel.count; ++i) {
            let row = listModel.get(i)
            let k = (row.k || "").trim()
            if (k === "") continue
            obj[k] = String(row.v || "")
        }
        root.changed(JSON.stringify(obj))
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
                    color: theme.textSubtle
                    font.pixelSize: theme.type.body.size
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

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 40
            radius: theme.radius.sm
            color: addArea.containsMouse ? theme.stateHover : theme.alpha(theme.text, 0)
            border.width: 1
            border.color: theme.surfaceBorder

            Behavior on color {
                ColorAnimation { duration: 100 }
            }

            Row {
                anchors.centerIn: parent
                spacing: 6

                SvgIcon {
                    name: "add"
                    size: 16
                    color: addArea.containsMouse ? theme.text : theme.textMuted
                    anchors.verticalCenter: parent.verticalCenter

                    Behavior on color {
                        ColorAnimation { duration: 100 }
                    }
                }

                Text {
                    text: root.addLabel
                    color: addArea.containsMouse ? theme.text : theme.textMuted
                    font.pixelSize: theme.type.label.size
                    anchors.verticalCenter: parent.verticalCenter

                    Behavior on color {
                        ColorAnimation { duration: 100 }
                    }
                }
            }

            MouseArea {
                id: addArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root._addRow()
            }
        }
    }
}
