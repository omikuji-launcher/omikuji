import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import omikuji 1.0
import "../cards"
import "../controls"
import "../primitives"

Item {
    id: root

    property var epicModel: null
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
        if (!epicModel) return
        epicModel.refresh_tools()
        if (epicModel.isLoggedIn) {
            epicModel.refresh()
        }
    }
    Component.onCompleted: _maybeRefresh()
    onVisibleChanged: if (visible) _maybeRefresh()

    readonly property bool isLoggedIn: epicModel && epicModel.isLoggedIn
    readonly property bool isRefreshing: epicModel && epicModel.isRefreshing === true

    CardGrid {
        id: cardGrid
        anchors.fill: parent
        visible: root.isLoggedIn
        enabled: visible

        model: epicModel
        cardZoom: root.cardZoom
        cardSpacing: root.cardSpacing
        cardFlow: root.cardFlow

        headerComponent: Component {
            RowLayout {
                anchors.fill: parent
                spacing: 8

                Text {
                    text: qsTr("Logged in as: %1").arg(epicModel ? epicModel.displayName : "")
                    color: theme.textMuted
                    font.pixelSize: 13
                }

                Item { Layout.fillWidth: true }

                IconButton {
                    icon: "sync"
                    size: 32
                    onClicked: epicModel.refresh()
                }

                IconButton {
                    icon: "logout"
                    size: 32
                    onClicked: epicModel.logout()
                }
            }
        }

        delegate: BaseCard {
            id: epicCard
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
            leftIconName: "shield_moon"
            leftIconSize: 20
            selected: isInstalled
            clickable: false
            cardVisible: root.searchText === ""
                || (modelData.title || "").toLowerCase().includes(root.searchText.toLowerCase())

            actionComponent: Component {
                StoreCardAction {
                    icon: {
                        if (epicCard.cardState === "uninstalled") return "add"
                        if (epicCard.cardState === "needs-import") return "download"
                        return "check_circle"
                    }
                    visible: !epicCard.isDownloading
                    primary: epicCard.cardState !== "imported"
                    onClicked: {
                        if (epicCard.cardState === "uninstalled") {
                            root.installRequested(epicCard.index)
                        } else if (epicCard.cardState === "needs-import") {
                            root.importRequested(epicCard.index)
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
                        color: theme.alpha(theme.accent, 0.9)
                        visible: epicCard.isDownloading

                        Text {
                            anchors.centerIn: parent
                            text: {
                                let dl = root.activeDownloads[epicCard.modelData.appName]
                                if (!dl) return ""
                                if (dl.status === "Downloading") return dl.progress.toFixed(0) + "%"
                                return dl.status
                            }
                            color: theme.accentOn
                            font.pixelSize: 11
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

        Column {
            anchors.centerIn: parent
            spacing: 10

            SvgIcon {
                anchors.horizontalCenter: parent.horizontalCenter
                name: "shield_moon"
                size: 48
                color: theme.textFaint
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: qsTr("No games in this store")
                color: theme.textMuted
                font.pixelSize: 16
                font.weight: Font.Medium
            }
        }
    }

    StoreLoginOverlay {
        visible: epicModel && !epicModel.isLoggedIn
        iconName: "shield_moon"
        title: qsTr("Login to Epic Games")
        description: qsTr("To sync your Epic library, you need to provide an authorization code from Epic's website.")
        loginUrl: "https://legendary.gl/epiclogin"
        toolName: "Legendary"
        toolReady: epicModel && epicModel.toolReady
        toolInstalling: epicModel && epicModel.toolInstalling
        onLoginRequested: (code) => epicModel.login(code)
        onInstallToolRequested: epicModel.install_tools()
    }
}
