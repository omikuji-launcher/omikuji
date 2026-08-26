import QtQuick
import omikuji 1.0

import "."
import "../controls"
import "../primitives"

Item {
    id: root

    property var appSettings: null

    readonly property int rowLabelWidth: 200

    implicitHeight: content.height

    signal categoryAddRequested()
    signal categoryEditRequested(int index, var entry)
    signal categoryDeleteRequested(int index, var entry)
    signal manageLogRulesRequested()

    // swallow our own eho so the ListModel doesnt tear down mid-toggle / mid-drag
    property bool _selfApplying: false

    property var _languageOptions: [{ label: qsTr("System default"), value: "system" }, { label: "English", value: "en" }]

    function _buildLanguageOptions() {
        let opts = [
            { label: qsTr("System default"), value: "system" },
            { label: "English", value: "en" }
        ]
        if (!appSettings) return opts
        let extra = []
        try { extra = JSON.parse(appSettings.availableLanguagesJson()) } catch (e) { extra = [] }
        for (let i = 0; i < extra.length; i++)
            opts.push({ label: extra[i].name, value: extra[i].code })
        return opts
    }

    function _refreshLanguageOptions() { _languageOptions = _buildLanguageOptions() }

    ListModel { id: categoriesModel }

    function _loadCategories() {
        if (!appSettings) return
        let arr = []
        try { arr = JSON.parse(appSettings.categoriesJson()) } catch (e) { arr = [] }
        categoriesModel.clear()
        for (let i = 0; i < arr.length; i++) {
            let c = arr[i]
            categoriesModel.append({
                enabled: c.enabled !== false,
                name: c.name || "",
                icon: c.icon || "",
                kind: c.kind || "tag",
                value: c.value || "",
                auto_name: c.auto_name === true
            })
        }
    }

    function _persistFromModel() {
        if (!appSettings) return
        let arr = []
        for (let i = 0; i < categoriesModel.count; i++) {
            let e = categoriesModel.get(i)
            arr.push({ enabled: e.enabled, name: e.name, icon: e.icon, kind: e.kind, value: e.value, auto_name: e.auto_name === true })
        }
        root._selfApplying = true
        appSettings.applyCategoriesJson(JSON.stringify(arr))
        root._selfApplying = false
    }

    function _setCategoryEnabled(index, value) {
        categoriesModel.setProperty(index, "enabled", value)
        _persistFromModel()
    }

    onAppSettingsChanged: { _loadCategories(); _refreshLanguageOptions() }
    Component.onCompleted: { _loadCategories(); _refreshLanguageOptions() }

    Connections {
        target: appSettings
        function onCategoriesChanged() {
            if (root._selfApplying) return
            root._loadCategories()
        }
    }

    Column {
        id: content
        width: parent.width
        spacing: Theme.space.xxl

        SettingsSection {
            label: qsTr("Language")
            width: parent.width

            SettingsRow {
                label: qsTr("Language")
                description: qsTr("Restart to apply")
                labelWidth: root.rowLabelWidth
                width: parent.width

                M3Dropdown {
                    width: 220
                    options: root._languageOptions
                    currentIndex: {
                        let cur = appSettings ? appSettings.language : "system"
                        let opts = root._languageOptions
                        for (let i = 0; i < opts.length; i++)
                            if (opts[i].value === cur) return i
                        return 0
                    }
                    onSelected: (value) => appSettings.applyLanguage(value)
                }
            }
        }

        SettingsSection {
            label: qsTr("Display")
            width: parent.width

            SettingsRow {
                label: qsTr("UI zoom")
                description: "Ctrl +, Ctrl -"
                labelWidth: root.rowLabelWidth
                width: parent.width
                contentRightMargin: 74

                M3SpinBox {
                    from: 70
                    to: 200
                    stepSize: 5
                    value: appSettings ? Math.round(appSettings.uiScale * 100) : 100
                    onMoved: (val) => appSettings.applyUiScale(val / 100)
                }
            }

            SettingsRow {
                label: qsTr("Card size")
                labelWidth: root.rowLabelWidth
                width: parent.width

                M3Slider {
                    width: 220
                    valueText: appSettings ? Math.round(appSettings.cardZoom * 100) + "%" : "100%"
                    from: 0.6
                    to: 1.5
                    stepSize: 0.05
                    value: appSettings ? appSettings.cardZoom : 1.0
                    onMoved: (val) => appSettings.applyCardZoom(val)
                    anchors.verticalCenter: parent.verticalCenter
                }
            }

            SettingsRow {
                label: qsTr("Card spacing")
                labelWidth: root.rowLabelWidth
                width: parent.width

                M3Slider {
                    width: 220
                    valueText: appSettings ? appSettings.cardSpacing + "px" : "16px"
                    from: 4
                    to: 40
                    stepSize: 2
                    value: appSettings ? appSettings.cardSpacing : 16
                    onMoved: (val) => appSettings.applyCardSpacing(Math.round(val))
                    anchors.verticalCenter: parent.verticalCenter
                }
            }

            SettingsRow {
                label: qsTr("Card shadow")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.cardElevation : true
                    onToggled: (val) => appSettings.applyCardElevation(val)
                }
            }

            SettingsRow {
                label: qsTr("Card flow")
                labelWidth: root.rowLabelWidth
                width: parent.width

                M3Dropdown {
                    width: 200
                    options: [
                        { label: qsTr("Left"),   value: "left" },
                        { label: qsTr("Center"), value: "center" },
                        { label: qsTr("Right"),  value: "right" }
                    ]
                    currentIndex: {
                        let v = appSettings ? appSettings.cardFlow : "center"
                        if (v === "left") return 0
                        if (v === "right") return 2
                        return 1
                    }
                    onSelected: (value) => appSettings.applyCardFlow(value)
                }
            }

            SettingsRow {
                label: qsTr("Library sort")
                labelWidth: root.rowLabelWidth
                width: parent.width

                M3Dropdown {
                    width: 200
                    options: [
                        { label: qsTr("Date added"), value: "default" },
                        { label: qsTr("Name A-Z"),   value: "a-z" },
                        { label: qsTr("Name Z-A"),   value: "z-a" },
                        { label: qsTr("Custom"),     value: "custom" }
                    ]
                    currentIndex: Math.max(0, options.findIndex(o => o.value === (appSettings ? appSettings.cardSort : "default")))
                    onSelected: (value) => appSettings.applyCardSort(value)
                }
            }

            SettingsRow {
                label: qsTr("Card style")
                description: qsTr("Normal crops to fill, Fit shows the whole image, Frameless makes the image the card")
                labelWidth: root.rowLabelWidth
                width: parent.width

                M3Dropdown {
                    width: 200
                    options: [
                        { label: qsTr("Normal"), value: "normal" },
                        { label: qsTr("Fit"), value: "fit" },
                        { label: qsTr("Frameless"), value: "frameless" }
                    ]
                    currentIndex: Math.max(0, options.findIndex(o => o.value === (appSettings ? appSettings.cardStyle : "normal")))
                    onSelected: (value) => appSettings.applyCardStyle(value)
                }
            }

            SettingsRow {
                label: qsTr("Show hidden games")
                description: qsTr("Keep games marked as hidden visible in the library")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.showHidden : false
                    onToggled: (val) => appSettings.applyShowHidden(val)
                }
            }

            SettingsRow {
                label: qsTr("Dim hidden games")
                description: qsTr("Fade hidden games so they stand out while shown")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.dimHidden : false
                    onToggled: (val) => appSettings.applyDimHidden(val)
                }
            }

            SettingsRow {
                label: qsTr("Muted icons")
                description: qsTr("Dim icons to ~55% instead of full contrast")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.mutedIcons : false
                    onToggled: (val) => appSettings.applyMutedIcons(val)
                }
            }

            SettingsRow {
                label: qsTr("Filled icons")
                description: qsTr("Use the filled Material Symbols variants")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.filledIcons : false
                    onToggled: (val) => appSettings.applyFilledIcons(val)
                }
            }

            SettingsRow {
                label: qsTr("Highlight logs")
                description: qsTr("Color error, fixme and warning lines in log output")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.highlightLogs : true
                    onToggled: (val) => appSettings.applyHighlightLogs(val)
                }
            }

            M3Button {
                small: true
                variant: "tonal"
                text: qsTr("Manage colors")
                visible: appSettings ? appSettings.highlightLogs : false
                onClicked: root.manageLogRulesRequested()
            }
        }

        SettingsSection {
            label: qsTr("Interaction")
            width: parent.width

            SettingsRow {
                label: qsTr("Double-click card to launch")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.doubleClickLaunches : false
                    onToggled: (val) => appSettings.applyDoubleClickLaunches(val)
                }
            }
        }

        SettingsSection {
            label: qsTr("Library categories")
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
                    required property var model
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
                                color: dragArea.held || dragArea.containsMouse ? Theme.iconHover : Theme.icon
                            }

                            SvgIcon {
                                name: wrapper.icon
                                size: 20
                                color: Theme.icon
                                anchors.verticalCenter: parent.verticalCenter
                            }

                            Column {
                                spacing: 2
                                anchors.verticalCenter: parent.verticalCenter

                                Text {
                                    text: CategoryLabels.label(wrapper)
                                    color: Theme.text
                                    font.pixelSize: Theme.type.subtitle.size
                                }
                                Text {
                                    text: {
                                        let k = wrapper.kind
                                        let v = wrapper.value || ""
                                        if (k === "runner")    return qsTr("runner: %1").arg(v)
                                        if (k === "tag")       return qsTr("tag: %1").arg(v)
                                        if (k === "favourite") return qsTr("favourites")
                                        if (k === "recent")    return qsTr("recent (top 10)")
                                        if (k === "all")       return qsTr("all games")
                                        return k
                                    }
                                    color: Theme.textSubtle
                                    font.pixelSize: Theme.type.caption.size
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
                                    enabled: wrapper.model.enabled, name: wrapper.name, icon: wrapper.icon,
                                    kind: wrapper.kind, value: wrapper.value, auto_name: wrapper.model.auto_name
                                })
                            }
                            IconButton {
                                icon: "close"
                                size: 32
                                danger: true
                                onClicked: root.categoryDeleteRequested(wrapper.index, {
                                    enabled: wrapper.model.enabled, name: wrapper.name, icon: wrapper.icon,
                                    kind: wrapper.kind, value: wrapper.value
                                })
                            }
                            Item {
                                width: 44
                                height: 32
                                M3Switch {
                                    anchors.centerIn: parent
                                    checked: wrapper.model.enabled
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
                    radius: Theme.radius.sm
                    color: addHover.containsMouse
                        ? Theme.alpha(Theme.text, 0.06)
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
                        color: Theme.accent
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: qsTr("Add category")
                        color: Theme.accent
                        font.pixelSize: Theme.type.body.size
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
            label: qsTr("Store tabs")
            width: parent.width

            SettingsRow {
                label: "Steam"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.showSteam : true
                    onToggled: (val) => appSettings.applyShowSteam(val)
                }
            }

            SettingsRow {
                label: "Epic Games"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.showEpic : true
                    onToggled: (val) => appSettings.applyShowEpic(val)
                }
            }

            SettingsRow {
                label: "GOG"
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.showGog : true
                    onToggled: (val) => appSettings.applyShowGog(val)
                }
            }

            SettingsRow {
                label: qsTr("Gachas")
                labelWidth: root.rowLabelWidth
                width: parent.width
                M3Switch {
                    checked: appSettings ? appSettings.showGachas : true
                    onToggled: (val) => appSettings.applyShowGachas(val)
                }
            }
        }
    }
}
