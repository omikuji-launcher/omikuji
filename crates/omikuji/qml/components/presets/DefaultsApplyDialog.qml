pragma ComponentBehavior: Bound

import QtQuick
import omikuji 1.0
import QtQuick.Layouts

DialogCard {
    sizeKey: "defaults_apply"
    id: root

    property var defaults: null
    property var gameModel: null

    property var fields: []
    property var groups: []
    property var games: []
    property var checkedKeys: []
    property var checkedGameIds: []
    property bool replaceMaps: false

    readonly property bool mapChecked: fields.some(f => f.is_map && checkedKeys.indexOf(f.key) !== -1)

    maxWidth: 520
    fillHeight: true
    preferredHeight: 640
    title: qsTr("Apply defaults to existing games")

    function show() {
        if (!defaults || !gameModel) return
        try { root.fields = JSON.parse(defaults.fieldsJson()) }
        catch (e) { root.fields = [] }
        try {
            root.games = JSON.parse(gameModel.gameSummariesJson())
                .map(g => ({ id: g.id, title: g.name, image: g.coverart }))
        } catch (e) {
            root.games = []
        }
        root.groups = groupFields(root.fields)
        root.checkedKeys = []
        root.checkedGameIds = []
        root.replaceMaps = false
        open()
    }

    function hide() { close() }

    function groupFields(list) {
        let out = []
        for (const f of list) {
            let last = out[out.length - 1]
            if (!last || last.group !== f.group) {
                last = { group: f.group, fields: [] }
                out.push(last)
            }
            last.fields.push(f)
        }
        return out
    }

    function toggleKey(key) {
        root.checkedKeys = checkedKeys.indexOf(key) !== -1
            ? checkedKeys.filter(k => k !== key)
            : checkedKeys.concat([key])
    }

    function _apply() {
        if (checkedKeys.length > 0 && checkedGameIds.length > 0)
            gameModel.applyDefaultsToExistingGames(checkedKeys.join(","), checkedGameIds.join(","), replaceMaps)
        close()
    }

    onCloseRequested: root.close()

    component CheckRow: Item {
        id: checkRow
        property bool checked: false
        property string text: ""
        property string hint: ""
        signal clicked()

        width: parent ? parent.width : 0
        height: Math.max(28, textCol.implicitHeight + Theme.space.sm)

        Rectangle {
            anchors.fill: parent
            radius: Theme.radius.sm
            color: rowArea.containsMouse ? Theme.alpha(Theme.text, 0.06) : "transparent"
            Behavior on color { ColorAnimation { duration: 100 } }
        }

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 10
            anchors.right: parent.right
            anchors.rightMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            spacing: Theme.space.md

            M3Checkbox {
                anchors.verticalCenter: parent.verticalCenter
                checked: checkRow.checked
            }

            Column {
                id: textCol
                anchors.verticalCenter: parent.verticalCenter
                spacing: 1
                Text {
                    text: checkRow.text
                    color: Theme.text
                    font.pixelSize: Theme.type.label.size
                }
                Text {
                    visible: checkRow.hint !== ""
                    text: checkRow.hint
                    color: Theme.textSubtle
                    font.pixelSize: Theme.type.micro.size
                }
            }
        }

        MouseArea {
            id: rowArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: checkRow.clicked()
        }
    }

    body: ColumnLayout {
        width: parent.width
        height: parent.height
        spacing: Theme.space.lg

        Text {
            Layout.fillWidth: true
            text: qsTr("Each ticked setting overwrites every ticked game with the value it holds in the Defaults tab, even if you never changed it.")
            color: Theme.textMuted
            font.pixelSize: Theme.type.caption.size
            wrapMode: Text.Wrap
            lineHeight: 1.35
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredHeight: 1
            spacing: Theme.space.sm

            CheckAllHeader {
                Layout.fillWidth: true
                title: qsTr("Settings")
                checked: root.fields.length > 0 && root.checkedKeys.length === root.fields.length
                indeterminate: root.checkedKeys.length > 0
                onCheckAllClicked: root.checkedKeys = checked ? [] : root.fields.map(f => f.key)
            }

            FieldSurface {
                Layout.fillWidth: true
                Layout.fillHeight: true

                WheelSink {
                    anchors.fill: parent
                }

                Flickable {
                    id: fieldList
                    anchors.fill: parent
                    anchors.margins: Theme.space.md
                    contentHeight: groupList.height
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    interactive: contentHeight > height

                    ThinScrollBar.vertical: ThinScrollBar {}

                    Column {
                        id: groupList
                        width: parent.width
                        spacing: Theme.space.sm

                        Repeater {
                            model: root.groups

                            SettingsSection {
                                id: groupSection
                                required property var modelData
                                label: SettingLabels.groupTitle(modelData.group)
                                width: parent.width

                                Column {
                                    width: parent.width
                                    spacing: 0

                                    Repeater {
                                        model: groupSection.modelData.fields

                                        CheckRow {
                                            required property var modelData
                                            text: SettingLabels.label(modelData.key)
                                            checked: root.checkedKeys.indexOf(modelData.key) !== -1
                                            onClicked: root.toggleKey(modelData.key)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            CheckRow {
                Layout.fillWidth: true
                visible: root.mapChecked
                text: qsTr("Replace env / DLL tables")
                hint: root.replaceMaps
                    ? qsTr("wipes the game's keys, then writes the global ones")
                    : qsTr("merges global keys into the game (global wins on conflict)")
                checked: root.replaceMaps
                onClicked: root.replaceMaps = !root.replaceMaps
            }
        }

        ArtCheckList {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredHeight: 1
            visible: root.games.length > 0
            title: qsTr("Games")
            checkAllVisible: true
            fillHeight: true
            items: root.games
            checkedIds: root.checkedGameIds
            onSelectionRequested: (ids) => root.checkedGameIds = ids
        }
    }

    footerLeft: Text {
        height: 36
        verticalAlignment: Text.AlignVCenter
        text: qsTr("Affects %n game(s)", "", root.checkedGameIds.length)
        color: Theme.textSubtle
        font.pixelSize: Theme.type.caption.size
    }

    actions: Row {
        spacing: Theme.space.sm

        M3Button {
            text: qsTr("Cancel")
            variant: "text"
            onClicked: root.close()
        }

        M3Button {
            text: qsTr("Apply")
            variant: "filled"
            enabled: root.checkedKeys.length > 0 && root.checkedGameIds.length > 0
            onClicked: root._apply()
        }
    }
}
