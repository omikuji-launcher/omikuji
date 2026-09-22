pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import omikuji 1.0

Item {
    id: root

    property var storeModel: null
    property real cardZoom: 1.0
    property string cardStyle: "normal"
    property int cardSpacing: 16
    property bool cardElevation: false
    property string searchText: ""
    property string cardFlow: "center"
    property var activeDownloads: ({})

    property string iconName: ""
    property string loginTitle: ""
    property string loginDescription: ""
    property string loginUrl: ""
    property string toolName: ""

    signal backClicked()
    signal gameImported()
    signal installRequested(int index)
    signal importRequested(int index)

    function _maybeRefresh() {
        if (!storeModel) return
        storeModel.refresh_tools()
        if (storeModel.isLoggedIn) {
            storeModel.refresh()
        }
    }
    Component.onCompleted: {
        _maybeRefresh()
        _syncRows()
    }
    onVisibleChanged: if (visible) _maybeRefresh()

    property bool hasRows: false

    function _syncRows() {
        root.hasRows = !!root.storeModel && root.storeModel.rowCount() > 0
    }

    onStoreModelChanged: _syncRows()

    Connections {
        target: root.storeModel
        function onModelReset() { root._syncRows() }
        function onRowsInserted() { root._syncRows() }
        function onRowsRemoved() { root._syncRows() }
    }

    // the grid stays mounted so cached cards paint during live refresh; overlays sit on top (z:90)
    readonly property bool isLoggedIn: storeModel && storeModel.isLoggedIn
    readonly property bool isRefreshing: storeModel && storeModel.isRefreshing === true
    readonly property bool gridReady: gridLoader.status === Loader.Ready

    Loader {
        id: gridLoader
        anchors.fill: parent
        asynchronous: true
        active: root.isLoggedIn && (root.hasRows || !root.isRefreshing)
        opacity: root.gridReady ? 1 : 0
        visible: opacity > 0
        sourceComponent: gridComponent

        Behavior on opacity {
            NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
        }
    }

    Component {
        id: gridComponent

        CardGrid {
            model: root.storeModel
            cardZoom: root.cardZoom
            cardSpacing: root.cardSpacing
            cardFlow: root.cardFlow

            headerComponent: Component {
                RowLayout {
                    anchors.fill: parent
                    spacing: 8

                    Text {
                        text: qsTr("Logged in as: %1").arg(root.storeModel ? root.storeModel.displayName : "")
                        color: Theme.textMuted
                        font.pixelSize: Theme.type.label.size
                    }

                    Item { Layout.fillWidth: true }

                    IconButton {
                        icon: "sync"
                        size: 32
                        onClicked: root.storeModel.refresh()
                    }

                    IconButton {
                        icon: "logout"
                        size: 32
                        onClicked: root.storeModel.logout()
                    }
                }
            }

            delegate: BaseCard {
                id: storeCard
                required property var modelData
                required property int index

                width: 180 * root.cardZoom
                height: styledHeight
                cardStyle: root.cardStyle
                elevation: root.cardElevation

                property bool isInstalled: modelData.isInstalled
                property bool hasLibraryEntry: modelData.hasLibraryEntry === true
                property bool isDownloading: root.activeDownloads[modelData.appName] !== undefined
                property string cardState: !isInstalled ? "uninstalled"
                    : (hasLibraryEntry ? "imported" : "needs-import")

                title: modelData.title
                imageSource: modelData.coverart || ""
                imageOpacity: isInstalled ? 1.0 : 0.6
                leftIconName: root.iconName
                leftIconSize: 20
                selected: isInstalled
                clickable: false
                cardVisible: root.searchText === ""
                    || (modelData.title || "").toLowerCase().includes(root.searchText.toLowerCase())

                function primaryAction() {
                    if (isDownloading) return
                    if (cardState === "needs-import") root.importRequested(index)
                    else root.installRequested(index)
                }

                actionComponent: Component {
                    StoreCardAction {
                        icon: {
                            if (storeCard.cardState === "uninstalled") return "add"
                            if (storeCard.cardState === "needs-import") return "download"
                            return "bookmark_check"
                        }
                        visible: !storeCard.isDownloading
                        onClicked: storeCard.primaryAction()
                    }
                }

                overlayComponent: isDownloading ? progressOverlay : null

                Component {
                    id: progressOverlay

                    CardProgressOverlay {
                        readonly property var download: root.activeDownloads[storeCard.modelData.appName]
                        bannerArea: storeCard.bannerArea
                        bottomInset: storeCard.overlayBottomInset
                        status: download ? download.status : ""
                        progress: download ? download.progress : 0
                    }
                }
            }
        }
    }

    DelayedFlag {
        id: loadingShown
        source: root.isLoggedIn && (!root.gridReady || root.isRefreshing && !root.hasRows)
    }

    Item {
        id: loadingOverlay
        anchors.fill: parent
        visible: loadingShown.value
        z: 90

        LoadingSpirit {
            anchors.centerIn: parent
            spiritWidth: Math.round(Math.min(loadingOverlay.width * 0.34, 280))
            text: qsTr("Loading library")
            running: loadingOverlay.visible
        }
    }

    Item {
        id: emptyOverlay
        anchors.fill: parent
        visible: root.isLoggedIn && root.gridReady && !root.isRefreshing && !root.hasRows
        z: 90

        EmptyState {
            anchors.fill: parent
            icon: root.iconName
            text: qsTr("No games in this store")
        }
    }

    StoreLoginOverlay {
        visible: root.storeModel && !root.storeModel.isLoggedIn
        iconName: root.iconName
        title: root.loginTitle
        description: root.loginDescription
        loginUrl: root.loginUrl
        toolName: root.toolName
        toolReady: root.storeModel && root.storeModel.toolReady
        toolInstalling: root.storeModel && root.storeModel.toolInstalling
        onLoginRequested: (code) => root.storeModel.login(code)
        onInstallToolRequested: root.storeModel.install_tools()
    }
}
