import QtQuick
import QtQuick.Layouts

import "."
import "../widgets"

Item {
    id: root

    property var uiSettings: null

    readonly property int rowLabelWidth: 200

    implicitHeight: content.height

    signal categoryAddRequested()
    signal categoryEditRequested(int index, var entry)
    signal categoryDeleteRequested(int index, var entry)

    // swallow our own eho so the ListModel doesnt tear down mid-toggle / mid-drag
    property bool _selfApplying: false

    ListModel { id: categoriesModel }

    function _loadCategories() {
        if (!uiSettings) return
        let arr = []
        try { arr = JSON.parse(uiSettings.categoriesJson()) } catch (e) { arr = [] }
        categoriesModel.clear()
        for (let i = 0; i < arr.length; i++) {
            let c = arr[i]
            categoriesModel.append({
                enabled: c.enabled !== false,
                name: c.name || "",
                icon: c.icon || "",
                kind: c.kind || "tag",
                value: c.value || ""
            })
        }
    }

    function _persistFromModel() {
        if (!uiSettings) return
        let arr = []
        for (let i = 0; i < categoriesModel.count; i++) {
            let e = categoriesModel.get(i)
            arr.push({ enabled: e.enabled, name: e.name, icon: e.icon, kind: e.kind, value: e.value })
        }
        root._selfApplying = true
        uiSettings.applyCategoriesJson(JSON.stringify(arr))
        root._selfApplying = false
    }

    function _setCategoryEnabled(index, value) {
        categoriesModel.setProperty(index, "enabled", value)
        _persistFromModel()
    }

    onUiSettingsChanged: _loadCategories()
    Component.onCompleted: _loadCategories()

    Connections {
        target: uiSettings
        function onCategoriesChanged() {
            if (root._selfApplying) return
            root._loadCategories()
        }
    }

    Column {
        id: content
        width: parent.width
        spacing: 20

        SettingsSection {
            label: "Display"
            width: parent.width

            SettingsRow {
                label: "UI zoom"
                description: "Ctrl +, Ctrl −"
                labelWidth: root.rowLabelWidth
                width: parent.width

                Row {
                    spacing: 12
                    anchors.verticalCenter: parent.verticalCenter

                    M3Slider {
                        width: 220
                        showValue: false
                        from: 0.7
                        to: 2.0
                        stepSize: 0.05
                        value: uiSettings ? uiSettings.uiScale : 1.0
                        onMoved: (val) => uiSettings.applyUiScale(val)
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: uiSettings ? Math.round(uiSettings.uiScale * 100) + "%" : "100%"
                        color: theme.text
                        font.pixelSize: 13
                        font.family: "monospace"
                        width: 50
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }

            SettingsRow {
                label: "Card size"
                labelWidth: root.rowLabelWidth
                width: parent.width

                Row {
                    spacing: 12
                    anchors.verticalCenter: parent.verticalCenter

                    M3Slider {
                        width: 220
                        showValue: false
                        from: 0.6
                        to: 1.5
                        stepSize: 0.05
                        value: uiSettings ? uiSettings.cardZoom : 1.0
                        onMoved: (val) => uiSettings.applyCardZoom(val)
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: uiSettings ? Math.round(uiSettings.cardZoom * 100) + "%" : "100%"
                        color: theme.text
                        font.pixelSize: 13
                        font.family: "monospace"
                        width: 50
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }

            SettingsRow {
                label: "Card spacing"
                labelWidth: root.rowLabelWidth
                width: parent.width

                Row {
                    spacing: 12
                    anchors.verticalCenter: parent.verticalCenter

                    M3Slider {
                        width: 220
                        showValue: false
                        from: 4
                        to: 40
                        stepSize: 2
                        value: uiSettings ? uiSettings.cardSpacing : 16
                        onMoved: (val) => uiSettings.applyCardSpacing(Math.round(val))
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: uiSettings ? uiSettings.cardSpacing + "px" : "16px"
                        color: theme.text
                        font.pixelSize: 13
                        font.family: "monospace"
                        width: 50
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }

            SettingsRow {
                label: "Card shadow"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.cardElevation : true
                    onToggled: (val) => uiSettings.applyCardElevation(val)
                }
            }

            SettingsRow {
                label: "Card flow"
                labelWidth: root.rowLabelWidth
                width: parent.width

                M3Dropdown {
                    width: 200
                    options: [
                        { label: "Left",   value: "left" },
                        { label: "Center", value: "center" },
                        { label: "Right",  value: "right" }
                    ]
                    currentIndex: {
                        let v = uiSettings ? uiSettings.cardFlow : "center"
                        if (v === "left") return 0
                        if (v === "right") return 2
                        return 1
                    }
                    onSelected: (value) => uiSettings.applyCardFlow(value)
                }
            }

            SettingsRow {
                label: "Muted icons"
                description: "Dim icons to ~55% instead of full contrast"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.mutedIcons : false
                    onToggled: (val) => uiSettings.applyMutedIcons(val)
                }
            }
        }

        SettingsSection {
            label: "Behavior"
            width: parent.width

            SettingsRow {
                label: "Hide while playing"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.minimizeOnLaunch : false
                    onToggled: (val) => uiSettings.applyMinimizeOnLaunch(val)
                }
            }

            SettingsRow {
                label: "Show tray icon"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.showTrayIcon : false
                    onToggled: (val) => uiSettings.applyShowTrayIcon(val)
                }
            }

            SettingsRow {
                label: "Discord Rich Presence"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.discordRpc : false
                    onToggled: (val) => uiSettings.applyDiscordRpc(val)
                }
            }

            SettingsRow {
                label: "Unload store tabs"
                description: "After 15s idle"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.unloadStorePages : true
                    onToggled: (val) => uiSettings.applyUnloadStorePages(val)
                }
            }

            SettingsRow {
                label: "Save game logs to disk"
                description: "Off: logs live in memory only until the game exits. On: also written to cache/logs/."
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.saveGameLogs : false
                    onToggled: (val) => uiSettings.applySaveGameLogs(val)
                }
            }

            SettingsRow {
                label: "Check EG games updates on run"
                description: "Might slowdown start times for Epic games"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.autoCheckEpicUpdatesOnLaunch : false
                    onToggled: (val) => uiSettings.applyAutoCheckEpicUpdatesOnLaunch(val)
                }
            }

            SettingsRow {
                label: "Check GOG games updates on run"
                description: "Might slowdown start times for GOG games"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.autoCheckGogUpdatesOnLaunch : false
                    onToggled: (val) => uiSettings.applyAutoCheckGogUpdatesOnLaunch(val)
                }
            }

            SettingsRow {
                label: "Check for updates on app launch"
                description: "Queues updates in the downloads page on startup"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.autoCheckUpdatesOnBoot : false
                    onToggled: (val) => uiSettings.applyAutoCheckUpdatesOnBoot(val)
                }
            }
        }

        SettingsSection {
            label: "Library categories"
            width: parent.width

            ListView {
                id: categoriesList
                width: parent.width
                height: contentHeight
                model: categoriesModel
                interactive: false
                spacing: 0

                moveDisplaced: Transition {
                    NumberAnimation { properties: "y"; duration: 180; easing.type: Easing.OutCubic }
                }

                delegate: Item {
                    id: wrapper
                    required property int index
                    required property bool enabled
                    required property string name
                    required property string icon
                    required property string kind
                    required property string value

                    width: ListView.view.width
                    height: 52

                    Behavior on y {
                        NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
                    }

                    Item {
                        id: content
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        height: 52

                        Drag.active: dragArea.held
                        Drag.source: wrapper
                        Drag.hotSpot.x: width / 2
                        Drag.hotSpot.y: height / 2

                        scale: dragArea.held ? 1.02 : 1.0
                        opacity: dragArea.held ? 0.92 : 1.0
                        z: dragArea.held ? 2 : 0
                        Behavior on scale { NumberAnimation { duration: 120 } }
                        Behavior on opacity { NumberAnimation { duration: 120 } }

                        states: State {
                            when: dragArea.held
                            ParentChange { target: content; parent: categoriesList }
                            AnchorChanges {
                                target: content
                                anchors.left: undefined
                                anchors.right: undefined
                                anchors.verticalCenter: undefined
                            }
                        }

                        Row {
                            anchors.left: parent.left
                            anchors.leftMargin: 12
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 14

                            SvgIcon {
                                anchors.verticalCenter: parent.verticalCenter
                                name: "drag_indicator"
                                size: 20
                                color: dragArea.held || dragArea.containsMouse ? theme.iconHover : theme.icon
                            }

                            SvgIcon {
                                name: wrapper.icon
                                size: 20
                                color: theme.icon
                                anchors.verticalCenter: parent.verticalCenter
                            }

                            Column {
                                spacing: 2
                                anchors.verticalCenter: parent.verticalCenter

                                Text {
                                    text: wrapper.name
                                    color: theme.text
                                    font.pixelSize: 15
                                }
                                Text {
                                    text: {
                                        let k = wrapper.kind
                                        let v = wrapper.value || ""
                                        if (k === "runner")    return "runner: " + v
                                        if (k === "tag")       return "tag: " + v
                                        if (k === "favourite") return "favourites"
                                        if (k === "recent")    return "recent (top 10)"
                                        if (k === "all")       return "all games"
                                        return k
                                    }
                                    color: theme.textSubtle
                                    font.pixelSize: 12
                                }
                            }
                        }

                        Row {
                            anchors.right: parent.right
                            anchors.rightMargin: 98
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 8

                            IconButton {
                                icon: "tune"
                                size: 32
                                onClicked: root.categoryEditRequested(wrapper.index, {
                                    enabled: wrapper.enabled, name: wrapper.name, icon: wrapper.icon,
                                    kind: wrapper.kind, value: wrapper.value
                                })
                            }
                            IconButton {
                                icon: "close"
                                size: 32
                                onClicked: root.categoryDeleteRequested(wrapper.index, {
                                    enabled: wrapper.enabled, name: wrapper.name, icon: wrapper.icon,
                                    kind: wrapper.kind, value: wrapper.value
                                })
                            }
                            Item {
                                width: 44
                                height: 32
                                M3Switch {
                                    anchors.centerIn: parent
                                    checked: wrapper.enabled
                                    onToggled: (v) => root._setCategoryEnabled(wrapper.index, v)
                                }
                            }
                        }
                    }

                    DropArea {
                        anchors.fill: parent
                        anchors.margins: 4
                        onEntered: (drag) => {
                            let from = drag.source.index
                            let to = wrapper.index
                            if (from !== to) categoriesModel.move(from, to, 1)
                        }
                    }

                    MouseArea {
                        id: dragArea
                        property bool held: false

                        anchors.left: parent.left
                        anchors.top: parent.top
                        anchors.bottom: parent.bottom
                        width: 44

                        hoverEnabled: true
                        cursorShape: held ? Qt.ClosedHandCursor : Qt.OpenHandCursor
                        pressAndHoldInterval: 150

                        drag.target: held ? content : undefined
                        drag.axis: Drag.YAxis

                        onPressAndHold: held = true
                        onReleased: {
                            if (held) root._persistFromModel()
                            held = false
                        }
                    }
                }
            }

            Item {
                width: parent.width
                height: 40

                Rectangle {
                    anchors.fill: parent
                    radius: 8
                    color: addHover.containsMouse
                        ? Qt.rgba(theme.text.r, theme.text.g, theme.text.b, 0.06)
                        : "transparent"
                    Behavior on color { ColorAnimation { duration: 100 } }
                }

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: 6
                    spacing: 8

                    SvgIcon {
                        name: "add"
                        size: 18
                        color: theme.accent
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: "Add category"
                        color: theme.accent
                        font.pixelSize: 14
                        font.weight: Font.Medium
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }

                MouseArea {
                    id: addHover
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.categoryAddRequested()
                }
            }
        }

        SettingsSection {
            label: "Store tabs"
            width: parent.width

            SettingsRow {
                label: "Steam"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.showSteam : true
                    onToggled: (val) => uiSettings.applyShowSteam(val)
                }
            }

            SettingsRow {
                label: "Epic Games"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.showEpic : true
                    onToggled: (val) => uiSettings.applyShowEpic(val)
                }
            }

            SettingsRow {
                label: "GOG"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.showGog : true
                    onToggled: (val) => uiSettings.applyShowGog(val)
                }
            }

            SettingsRow {
                label: "Gachas"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: uiSettings ? uiSettings.showGachas : true
                    onToggled: (val) => uiSettings.applyShowGachas(val)
                }
            }
        }
    }
}
