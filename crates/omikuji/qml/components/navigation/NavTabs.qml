import QtQuick
import "../primitives"


Item {
    id: root
    width: 180
    clip: true
    property int minWidth: 56
    property int maxWidth: 320
    property int collapseThreshold: 36
    readonly property bool iconOnly: width < 110

    property int currentIndex: 0
    property string currentStore: ""
    property string currentBottom: ""
    property string headerLabel: ""

    property int downloadCount: 0

    property var uiSettings: null

    property bool showSteam: true
    property bool showEpic: true
    property bool showGog: true
    property bool showGachas: true

    signal tabSelected(int index)
    signal storeSelected(string storeName)
    signal downloadsClicked()
    signal settingsClicked()
    signal widthRequested(int value)

    property var tabs: []

    readonly property var storeDefs: [
        { name: "Steam", label: "Steam", icon: "steam", shown: root.showSteam },
        { name: "Epic", label: "Epic Games", icon: "shield_moon", shown: root.showEpic },
        { name: "GOG", label: "GOG", icon: "gog", shown: root.showGog },
        { name: "HoYo", label: qsTr("Gachas"), icon: "local_activity", shown: root.showGachas }
    ]

    function _storeOffset(name) {
        let y = 0
        for (let i = 0; i < storeDefs.length; i++) {
            if (storeDefs[i].name === name) return y
            if (storeDefs[i].shown) y += 42
        }
        return y
    }

    function _loadCategories() {
        if (!uiSettings) return
        let raw = uiSettings.categoriesJson()
        let parsed = []
        try { parsed = JSON.parse(raw) } catch (e) { parsed = [] }
        let next = []
        for (let i = 0; i < parsed.length; i++) {
            let c = parsed[i]
            if (c.enabled === false) continue
            next.push({ label: c.name, icon: c.icon, kind: c.kind, value: c.value || "" })
        }
        root.tabs = next
    }

    onUiSettingsChanged: _loadCategories()
    Component.onCompleted: _loadCategories()

    Connections {
        target: uiSettings
        function onCategoriesChanged() { root._loadCategories() }
    }

    component NavItem: Item {
        id: navItem
        property string icon: ""
        property string label: ""
        property bool selected: false
        property bool badge: false
        signal activated()

        width: root.width
        height: 40

        Rectangle {
            anchors.centerIn: parent
            width: parent.width - 20
            height: 36
            radius: 18
            color: navItemMouse.containsMouse && !navItem.selected
                ? theme.alpha(theme.text, 0.06)
                : "transparent"
            visible: !navItem.selected

            Behavior on color {
                ColorAnimation { duration: 100 }
            }
        }

        Row {
            anchors.left: parent.left
            anchors.leftMargin: root.iconOnly ? (root.width - 18) / 2 : 20
            anchors.verticalCenter: parent.verticalCenter
            spacing: 10
            z: 1

            Behavior on anchors.leftMargin {
                NumberAnimation { duration: 120; easing.type: Easing.OutCubic }
            }

            Item {
                width: 18
                height: 18
                anchors.verticalCenter: parent.verticalCenter

                SvgIcon {
                    anchors.fill: parent
                    name: navItem.icon
                    size: 18
                    color: navItem.selected ? theme.accent : theme.icon

                    Behavior on color {
                        ColorAnimation { duration: 100 }
                    }
                }

                Rectangle {
                    width: 8
                    height: 8
                    radius: 4
                    color: theme.accent
                    border.width: 2
                    border.color: theme.navBg
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.rightMargin: -2
                    anchors.topMargin: -2
                    visible: navItem.badge
                    scale: visible ? 1.0 : 0.0

                    Behavior on scale {
                        NumberAnimation { duration: 150; easing.type: Easing.OutBack; easing.overshoot: 1.6 }
                    }
                }
            }

            Text {
                text: navItem.label
                color: theme.text
                font.pixelSize: 13
                font.weight: navItem.selected ? Font.DemiBold : Font.Normal
                anchors.verticalCenter: parent.verticalCenter
                opacity: root.iconOnly ? 0 : 1
                visible: opacity > 0

                Behavior on opacity {
                    NumberAnimation { duration: 120 }
                }
            }
        }

        MouseArea {
            id: navItemMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: navItem.activated()
        }
    }

    Rectangle {
        anchors.fill: parent
        color: theme.navBg
    }

    Text {
        id: appTitle
        anchors.top: parent.top
        anchors.topMargin: root.iconOnly ? 0 : 20

        Behavior on anchors.topMargin {
            NumberAnimation { duration: 120; easing.type: Easing.OutCubic }
        }
        anchors.left: parent.left
        anchors.leftMargin: 20
        anchors.right: parent.right
        anchors.rightMargin: 20
        text: root.headerLabel
        color: theme.text
        font.pixelSize: 20
        font.weight: Font.DemiBold
        elide: Text.ElideRight
        opacity: root.iconOnly ? 0 : 1
        height: root.iconOnly ? 0 : implicitHeight

        Behavior on opacity {
            NumberAnimation { duration: 120 }
        }
        Behavior on height {
            NumberAnimation { duration: 120; easing.type: Easing.OutCubic }
        }
    }

    Rectangle {
        id: slidingPill
        x: 10
        width: root.width - 20
        height: 36
        radius: 18
        color: theme.alpha(theme.accent, 0.15)
        z: 0

        property real baseY: {
            if (root.currentBottom === "downloads") return downloadsBtn.y
            if (root.currentBottom === "settings")  return settingsBtn.y
            if (root.currentStore !== "")
                return navScroll.y + storesList.y + root._storeOffset(root.currentStore)
            return navScroll.y + tabList.y + root.currentIndex * 42
        }

        Behavior on baseY {
            NumberAnimation { duration: 250; easing.type: Easing.OutBack; easing.overshoot: 1.4 }
        }

        y: root.currentBottom !== ""
            ? baseY + 2
            : baseY - navScroll.contentY + 2

        // hide if tracked item is scrolled out of the Flickable viewport
        visible: {
            if (root.currentBottom !== "") return true
            let screenY = baseY - navScroll.contentY
            return screenY + height > navScroll.y && screenY < navScroll.y + navScroll.height
        }
    }

    Flickable {
        id: navScroll
        anchors.top: appTitle.bottom
        anchors.topMargin: 12
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: downloadsBtn.top
        anchors.bottomMargin: 8
        contentHeight: navContent.height
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        Item {
            id: navContent
            width: navScroll.width
            height: storesList.y + storesList.height + 8

            Text {
                id: libraryHeader
                anchors.top: parent.top
                anchors.topMargin: visible ? 12 : 0
                anchors.left: parent.left
                anchors.leftMargin: 20
                text: qsTr("Library")
                color: theme.textMuted
                font.pixelSize: 12
                font.weight: Font.Medium
                visible: root.tabs.length > 0 && !root.iconOnly
                height: visible ? implicitHeight : 0
            }

            Column {
                id: tabList
                anchors.top: libraryHeader.bottom
                anchors.topMargin: 8
                anchors.left: parent.left
                anchors.right: parent.right
                spacing: 2
                z: 1

                Repeater {
                    model: root.tabs

                    NavItem {
                        required property var modelData
                        required property int index

                        icon: modelData.icon
                        label: modelData.label
                        selected: index === root.currentIndex && root.currentStore === "" && root.currentBottom === ""
                        onActivated: {
                            root.currentIndex = index
                            root.currentStore = ""
                            root.tabSelected(index)
                        }
                    }
                }
            }

            Text {
                id: storesHeader
                anchors.top: tabList.bottom
                anchors.topMargin: visible ? 24 : 0
                anchors.left: parent.left
                anchors.leftMargin: 20
                text: qsTr("Stores")
                color: theme.textMuted
                font.pixelSize: 12
                font.weight: Font.Medium
                visible: (root.showSteam || root.showEpic || root.showGog || root.showGachas) && !root.iconOnly
                height: visible ? implicitHeight : 0
            }

            Column {
                id: storesList
                anchors.top: storesHeader.bottom
                anchors.topMargin: root.iconOnly ? 24 : 8
                anchors.left: parent.left
                anchors.right: parent.right
                spacing: 2
                z: 1

                Repeater {
                    model: root.storeDefs

                    NavItem {
                        required property var modelData

                        visible: modelData.shown
                        height: visible ? 40 : 0
                        icon: modelData.icon
                        label: modelData.label
                        selected: root.currentStore === modelData.name && root.currentBottom === ""
                        onActivated: {
                            root.currentStore = modelData.name
                            root.storeSelected(modelData.name)
                        }
                    }
                }
            }
        }
    }

    NavItem {
        id: downloadsBtn
        anchors.bottom: settingsBtn.top
        anchors.bottomMargin: 4
        icon: "download"
        label: root.downloadCount > 0 ? qsTr("Downloads (%1)").arg(root.downloadCount) : qsTr("Downloads")
        selected: root.currentBottom === "downloads"
        badge: root.downloadCount > 0
        onActivated: root.downloadsClicked()
    }

    NavItem {
        id: settingsBtn
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 16
        icon: "settings"
        label: qsTr("Settings")
        selected: root.currentBottom === "settings"
        onActivated: root.settingsClicked()
    }

    MouseArea {
        id: resizer
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: 6
        cursorShape: Qt.SizeHorCursor
        hoverEnabled: true
        z: 99

        property int startWidth: 0
        property real startGlobalX: 0

        onPressed: (mouse) => {
            startWidth = root.width
            startGlobalX = mapToGlobal(mouse.x, 0).x
        }
        onPositionChanged: (mouse) => {
            if (!pressed) return
            const globalX = mapToGlobal(mouse.x, 0).x
            const raw = startWidth + (globalX - startGlobalX)
            if (raw < root.collapseThreshold) {
                root.widthRequested(0)
            } else {
                root.widthRequested(Math.max(root.minWidth, Math.min(root.maxWidth, raw)))
            }
        }

        Rectangle {
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: 2
            color: theme.accent
            opacity: resizer.pressed ? 0.7 : (resizer.containsMouse ? 0.35 : 0)
            Behavior on opacity { NumberAnimation { duration: 120 } }
        }
    }

}
