import QtQuick
import QtQuick.Controls
import "../controls"
import "../primitives"


DialogCard {
    sizeKey: "archive_manage"
    id: root

    property var archiveManager: null
    property var activeInstalls: ({})

    property string category: ""
    property string sourceName: ""
    property string sourceKind: ""

    property var versions: []
    property var installedTags: ({})

    property string errorMessage: ""
    property bool fetching: false

    signal closed()
    signal versionDeleted(string category, string sourceName, string tag)
    signal removeSourceRequested(string category, string sourceName)

    maxWidth: 720
    scrollable: false
    fillHeight: true
    title: ""

    function show(cat, name, kind) {
        category = cat
        sourceName = name
        sourceKind = kind
        versions = []
        installedTags = ({})
        errorMessage = ""
        refreshInstalled()
        open()
        fetchVersionsNow()
    }

    function hide() {
        root.closed()
        close()
    }

    function refreshInstalled() {
        if (!archiveManager || sourceName === "") return
        try {
            let raw = archiveManager.listInstalled(category, sourceName)
            let list = JSON.parse(raw) || []
            let map = ({})
            for (let i = 0; i < list.length; i++) map[list[i]] = true
            installedTags = map
        } catch (e) {
            console.warn("installedTags parse failed:", e)
            installedTags = ({})
        }
    }

    function fetchVersionsNow() {
        if (!archiveManager || sourceName === "") return
        fetching = true
        errorMessage = ""
        archiveManager.fetchVersions(category, sourceName)
    }

    onCloseRequested: { root.closed(); root.close() }

    footerLeft: M3Button {
        text: qsTr("Remove source")
        variant: "tonal"
        danger: true
        onClicked: root.removeSourceRequested(root.category, root.sourceName)
    }

    actions: Row {
        M3Button {
            text: qsTr("Close")
            variant: "tonal"
            onClicked: { root.closed(); root.close() }
        }
    }

    Connections {
        target: archiveManager
        enabled: root.shown && archiveManager !== null

        function onVersionsReady(cat, name, json) {
            if (cat !== root.category || name !== root.sourceName) return
            root.fetching = false
            try {
                root.versions = JSON.parse(json) || []
            } catch (e) {
                root.versions = []
                root.errorMessage = qsTr("Couldn't parse versions response.")
            }
        }
        function onVersionsFailed(cat, name, err) {
            if (cat !== root.category || name !== root.sourceName) return
            root.fetching = false
            root.errorMessage = err
        }
        function onInstallCompleted(cat, name, tag, installDir) {
            if (cat !== root.category || name !== root.sourceName) return
            root.refreshInstalled()
        }
        function onInstallFailed(cat, name, tag, err) {
            if (cat !== root.category || name !== root.sourceName) return
            root.errorMessage = err
        }
    }

    body: Item {
        width: parent.width
        height: parent.height

        Item {
            id: bodyHeader
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 64

            Column {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2

                Row {
                    spacing: 10
                    Text {
                        text: root.sourceName
                        color: theme.text
                        font.pixelSize: theme.type.headline.size
                        font.weight: Font.DemiBold
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Rectangle {
                        height: 18
                        width: kindLabel.width + 14
                        radius: 9
                        color: theme.alpha(theme.accent, 0.15)
                        anchors.verticalCenter: parent.verticalCenter
                        Text {
                            id: kindLabel
                            anchors.centerIn: parent
                            text: root.sourceKind
                            color: theme.accent
                            font.pixelSize: theme.type.micro.size
                            font.weight: Font.Medium
                            font.capitalization: Font.AllUppercase
                            font.letterSpacing: 0.6
                        }
                    }
                }

                Text {
                    text: root.fetching ? qsTr("Fetching versions…")
                        : root.versions.length > 0 ? qsTr("%1 versions available").arg(root.versions.length)
                        : root.errorMessage !== "" ? root.errorMessage
                        : qsTr("No versions loaded yet")
                    color: root.errorMessage !== "" ? theme.error : theme.textSubtle
                    font.pixelSize: theme.type.caption.size
                }
            }
        }

        Rectangle {
            id: bodyDivider
            anchors.top: bodyHeader.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            height: 1
            color: theme.separator
        }

        ListView {
            id: list
            anchors.top: bodyDivider.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            clip: true
            model: root.versions
            spacing: 0

            ScrollBar.vertical: ThinScrollBar {}

            Text {
                anchors.centerIn: parent
                visible: list.count === 0
                text: root.fetching ? qsTr("Loading…")
                    : root.errorMessage !== "" ? qsTr("Couldn't load versions.")
                    : qsTr("No versions available.")
                color: theme.textSubtle
                font.pixelSize: theme.type.label.size
            }

            delegate: Item {
                required property int index
                required property var modelData

                readonly property string tag: modelData.tag || ""
                readonly property string publishedAt: modelData.published_at || ""
                readonly property int assetSize: modelData.asset_size || 0
                readonly property bool installed: root.installedTags[tag] === true
                readonly property bool busy:
                    root.activeInstalls[root.category + "/" + root.sourceName + "/" + tag] !== undefined

                width: ListView.view.width
                height: 54

                Rectangle {
                    anchors.fill: parent
                    anchors.leftMargin: theme.space.sm
                    anchors.rightMargin: theme.space.sm
                    anchors.topMargin: 3
                    anchors.bottomMargin: 3
                    radius: theme.radius.sm
                    color: rowMouse.containsMouse
                        ? theme.alpha(theme.text, 0.05)
                        : "transparent"
                    Behavior on color { ColorAnimation { duration: theme.dur.fast } }
                }

                MouseArea {
                    id: rowMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    acceptedButtons: Qt.NoButton
                }

                Row {
                    anchors.left: parent.left
                    anchors.leftMargin: 24
                    anchors.right: actionSlot.left
                    anchors.rightMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 14

                    Text {
                        text: tag
                        color: theme.text
                        font.pixelSize: theme.type.label.size
                        font.weight: Font.Medium
                        font.family: "monospace"
                        width: 220
                        elide: Text.ElideRight
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Text {
                        text: publishedAt.length >= 10 ? publishedAt.substring(0, 10) : publishedAt
                        color: theme.textSubtle
                        font.pixelSize: theme.type.caption.size
                        font.family: "monospace"
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Text {
                        text: assetSize > 0
                            ? (assetSize / (1024 * 1024)).toFixed(1) + " MB"
                            : ""
                        color: theme.textSubtle
                        font.pixelSize: theme.type.caption.size
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }

                Item {
                    id: actionSlot
                    anchors.right: parent.right
                    anchors.rightMargin: 20
                    anchors.verticalCenter: parent.verticalCenter
                    width: 96
                    height: 30

                    M3Button {
                        anchors.centerIn: parent
                        visible: !installed && !busy
                        text: qsTr("Install")
                        variant: "filled"
                        onClicked: archiveManager.installVersion(
                            root.category,
                            root.sourceName,
                            JSON.stringify(modelData)
                        )
                    }

                    Row {
                        anchors.centerIn: parent
                        visible: installed && !busy
                        spacing: 8

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: qsTr("Installed")
                            color: theme.success
                            font.pixelSize: theme.type.micro.size
                            font.weight: Font.Medium
                        }

                        IconButton {
                            anchors.verticalCenter: parent.verticalCenter
                            icon: "close"
                            size: 28
                            rounded: true
                            danger: true
                            onClicked: {
                                archiveManager.deleteVersion(root.category, root.sourceName, tag)
                                root.refreshInstalled()
                                root.versionDeleted(root.category, root.sourceName, tag)
                            }
                        }
                    }

                    Text {
                        anchors.centerIn: parent
                        visible: busy
                        text: qsTr("Working…")
                        color: theme.textMuted
                        font.pixelSize: theme.type.caption.size
                    }
                }

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: theme.separator
                    visible: index < (list.count - 1)
                }
            }
        }
    }
}
