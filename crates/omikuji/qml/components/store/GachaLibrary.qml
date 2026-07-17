import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../cards"
import "../primitives"


Item {
    id: root

    property var gameModel: null
    property real cardZoom: 1.0
    property string cardStyle: "normal"
    property int cardSpacing: 16
    property bool cardElevation: false
    property string searchText: ""
    property var activeDownloads: ({})
    property string cardFlow: "center"

    signal backClicked()
    signal installRequested(string manifestId)

    property var manifests: []
    property var posters: ({})

    function refreshManifests() {
        if (!gameModel) { manifests = []; return }
        let raw = gameModel.list_gachas()
        let arr = []
        try { arr = JSON.parse(raw) || [] } catch (e) { arr = [] }
        manifests = arr
    }

    function refreshPosters() {
        if (!gameModel) { posters = ({}); return }
        try {
            posters = JSON.parse(gameModel.gacha_posters()) || {}
        } catch (e) {
            posters = ({})
        }
    }

    Component.onCompleted: { refreshManifests(); refreshPosters() }
    onVisibleChanged: if (visible) { refreshManifests(); refreshPosters() }

    Connections {
        target: gameModel
        function onGachaManifestsReady(fetched) {
            if (fetched > 0) { refreshManifests(); refreshPosters() }
        }
    }

    // retry posters every 2s, background art downloads may still be landing
    Timer {
        interval: 2000
        running: root.visible
        repeat: true
        onTriggered: {
            let missing = false
            for (let i = 0; i < root.manifests.length; i++) {
                let id = root.manifests[i].id
                if (!root.posters[id]) { missing = true; break }
            }
            if (missing) refreshPosters()
            else running = false
        }
    }

    // hardcoded per-prefix tints until manifest schema gets a color field (i will probably forget to add this 100%)
    function cardTintFor(m) {
        let prefix = m.app_id_prefix || ""
        switch (prefix) {
            case "genshin":   return "#1a2540"
            case "star-rail": return "#1c1428"
            case "zzz":       return "#141e28"
            case "endfield":  return "#1a1f14"
            case "wuwa":      return "#0f2a28"
            case "pgr":       return "#1f0f1c"
            default:          return "#1a1a2e"
        }
    }

    CardGrid {
        id: cardGrid
        anchors.fill: parent
        model: root.manifests
        cardZoom: root.cardZoom
        cardSpacing: root.cardSpacing
        cardFlow: root.cardFlow

        headerComponent: Component {
            RowLayout {
                anchors.fill: parent

                SvgIcon {
                    name: "local_activity"
                    size: 20
                    color: theme.textMuted
                }

                Text {
                    text: qsTr("Gacha Games")
                    color: theme.textMuted
                    font.pixelSize: theme.type.label.size
                }

                Item { Layout.fillWidth: true }
            }
        }

        delegate: BaseCard {
            id: gachaCard
            required property var modelData
            required property int index

            width: 180 * root.cardZoom
            height: styledHeight
            cardStyle: root.cardStyle
            elevation: root.cardElevation

            property string appIdForDownloads:
                (modelData.app_id_prefix || "") + ":"
                + (modelData.editions && modelData.editions.length > 0
                   ? modelData.editions[0].id : "global")

            property bool isDownloading:
                root.activeDownloads[appIdForDownloads] !== undefined

            title: modelData.display_name || ""
            imageSource: root.posters[modelData.id] || ""
            placeholderTint: root.cardTintFor(modelData)
            letter: modelData.letter_fallback || ""
            letterFontSize: 64
            letterColor: Qt.rgba(1, 1, 1, 0.3)
            leftIconName: "local_activity"
            leftIconSize: 20
            clickable: false
            cardVisible: root.searchText === ""
                || (modelData.display_name || "").toLowerCase().includes(root.searchText.toLowerCase())

            actionComponent: Component {
                StoreCardAction {
                    icon: "add"
                    visible: !gachaCard.isDownloading
                    onClicked: root.installRequested(gachaCard.modelData.id)
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
                        visible: gachaCard.isDownloading

                        Text {
                            anchors.centerIn: parent
                            text: {
                                let dl = root.activeDownloads[gachaCard.appIdForDownloads]
                                if (!dl) return ""
                                if (dl.status === "Downloading") return dl.progress.toFixed(0) + "%"
                                return dl.status
                            }
                            color: theme.accentOn
                            font.pixelSize: theme.type.micro.size
                            font.weight: Font.Bold
                        }
                    }
                }
            }
        }
    }
}
