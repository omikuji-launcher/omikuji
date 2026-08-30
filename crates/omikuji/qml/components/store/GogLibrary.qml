pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import omikuji 1.0
import "../cards"
import "../controls"
import "../primitives"

Item {
    id: root

    property var gogModel: null
    property real cardZoom: 1.0
    property string cardStyle: "normal"
    property int cardSpacing: 16
    property bool cardElevation: false
    property string searchText: ""
    property string cardFlow: "center"
    property var activeDownloads: ({})

    signal backClicked()
    signal gameImported()
    signal installRequested(int index)
    signal importRequested(int index)

    function _maybeRefresh() {
        if (!gogModel) return
        gogModel.refresh_tools()
        if (gogModel.isLoggedIn) {
            gogModel.refresh()
        }
    }
    Component.onCompleted: _maybeRefresh()
    onVisibleChanged: if (visible) _maybeRefresh()

    // cardGrid stays mounted so cached cards paint during live refresh; overlays sit on top (z:90)
    readonly property bool isLoggedIn: gogModel && gogModel.isLoggedIn
    readonly property bool isRefreshing: gogModel && gogModel.isRefreshing === true

    CardGrid {
        id: cardGrid
        anchors.fill: parent
        visible: root.isLoggedIn
        enabled: visible

        model: root.gogModel
        cardZoom: root.cardZoom
        cardSpacing: root.cardSpacing
        cardFlow: root.cardFlow

        headerComponent: Component {
            RowLayout {
                anchors.fill: parent
                spacing: 8

                Text {
                    text: qsTr("Logged in as: %1").arg(root.gogModel ? root.gogModel.displayName : "")
                    color: Theme.textMuted
                    font.pixelSize: Theme.type.label.size
                }

                Item { Layout.fillWidth: true }

                IconButton {
                    icon: "sync"
                    size: 32
                    onClicked: root.gogModel.refresh()
                }

                IconButton {
                    icon: "logout"
                    size: 32
                    onClicked: root.gogModel.logout()
                }
            }
        }

        delegate: BaseCard {
            id: gogCard
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
            leftIconName: "gog"
            leftIconSize: 20
            selected: isInstalled
            clickable: false
            cardVisible: root.searchText === ""
                || (modelData.title || "").toLowerCase().includes(root.searchText.toLowerCase())

            actionComponent: Component {
                StoreCardAction {
                    icon: {
                        if (gogCard.cardState === "uninstalled") return "add"
                        if (gogCard.cardState === "needs-import") return "download"
                        return "check_circle"
                    }
                    visible: !gogCard.isDownloading
                    onClicked: {
                        if (gogCard.cardState === "needs-import") {
                            root.importRequested(gogCard.index)
                        } else {
                            root.installRequested(gogCard.index)
                        }
                    }
                }
            }

            overlayComponent: Component {
                Item {
                    Rectangle {
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: 4
                        height: 24
                        radius: 10
                        color: Theme.alpha(Theme.accent, 0.9)
                        visible: gogCard.isDownloading

                        Text {
                            anchors.centerIn: parent
                            text: {
                                let dl = root.activeDownloads[gogCard.modelData.appName]
                                if (!dl) return ""
                                if (dl.status === "Downloading") return dl.progress.toFixed(0) + "%"
                                return dl.status
                            }
                            color: Theme.accentOn
                            font.pixelSize: Theme.type.micro.size
                            font.weight: Font.Bold
                        }
                    }
                }
            }
        }
    }

    Item {
        id: loadingOverlay
        anchors.fill: parent
        visible: root.isLoggedIn && root.isRefreshing && cardGrid.count === 0
        z: 90

        LoadingDots {
            anchors.centerIn: parent
            text: qsTr("Loading library")
            running: loadingOverlay.visible
        }
    }

    Item {
        id: emptyOverlay
        anchors.fill: parent
        visible: root.isLoggedIn && !root.isRefreshing && cardGrid.count === 0
        z: 90

        EmptyState {
            anchors.fill: parent
            icon: "gog"
            text: qsTr("No games in this store")
        }
    }

    StoreLoginOverlay {
        visible: root.gogModel && !root.gogModel.isLoggedIn
        iconName: "gog"
        title: qsTr("Login to GOG")
        description: qsTr("To sync your GOG library, sign in on gog.com and paste the authorization code from the redirect URL.")
        loginUrl: root.gogModel ? root.gogModel.get_login_url() : ""
        toolName: "gogdl"
        toolReady: root.gogModel && root.gogModel.toolReady
        toolInstalling: root.gogModel && root.gogModel.toolInstalling
        onLoginRequested: (code) => root.gogModel.login(code)
        onInstallToolRequested: root.gogModel.install_tools()
    }
}
