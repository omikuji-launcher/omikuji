import QtQuick

// every knob is a live apply* call, no save/cancel flow
Item {
    id: root

    property var uiSettings: null
    property var componentsBridge: null
    property var archiveManager: null
    property var ofudaBridge: null
    property var defaults: null
    property var gameModel: null
    property var activeInstalls: ({})

    // bubbles to Main.qml which owns teh dialog, full-window dim needs a root-level sibling
    signal manageRequested(string category, string source, string kind)
    signal addSourceRequested(string category)

    signal categoryAddRequested()
    signal categoryEditRequested(int index, var entry)
    signal categoryDeleteRequested(int index, var entry)
    signal manageLogRulesRequested()

    signal defaultsApplyToExistingRequested()
    signal manageSetsRequested(string kind)

    signal prefixOpenRequested(var prefix)
    signal prefixCreateRequested()

    readonly property string modalTitle: qsTr("Settings")
    readonly property string modalSubtitle: ""
    readonly property string primaryLabel: ""
    readonly property string secondaryLabel: ""
    readonly property bool primaryEnabled: false
    readonly property bool secondaryEnabled: false

    function primaryAction() {}
    function secondaryAction() {}
    function closeAction() {}

    property var tabs: [
        { label: qsTr("Components"), kind: "components", icon: "layers" },
        { label: "Ofuda",            kind: "ofuda",      icon: "ofuda" },
        { label: qsTr("Defaults"),   kind: "defaults",   icon: "settings" },
        { label: qsTr("Interface"),  kind: "ui",         icon: "tune" },
        { label: qsTr("Theme"),      kind: "theme",      icon: "imagesmode" },
        { label: qsTr("About"),      kind: "about",      icon: "verified" }
    ]
    property int currentTabIndex: 0
    readonly property string currentKind:
        tabs[currentTabIndex] ? tabs[currentTabIndex].kind : "components"

    implicitHeight: contentCol.implicitHeight

    Column {
        id: contentCol
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 0

        Loader {
            width: parent.width
            active: root.currentKind === "components"
            visible: active
            source: "../settings/TabGlobalComponents.qml"
            onLoaded: {
                item.componentsBridge = root.componentsBridge
                item.archiveManager = root.archiveManager
                item.activeInstalls = Qt.binding(() => root.activeInstalls)
                item.manageRequested.connect((cat, name, kind) => {
                    root.manageRequested(cat, name, kind)
                })
                item.addSourceRequested.connect((cat) => {
                    root.addSourceRequested(cat)
                })
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "ofuda"
            visible: active
            source: "../settings/TabGlobalOfuda.qml"
            onLoaded: {
                item.ofudaBridge = root.ofudaBridge
                item.openRequested.connect((p) => root.prefixOpenRequested(p))
                item.createRequested.connect(() => root.prefixCreateRequested())
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "defaults"
            visible: active
            source: "../settings/TabGlobalDefaults.qml"
            onLoaded: {
                item.defaults = root.defaults
                item.gameModel = root.gameModel
                item.applyToExistingRequested.connect(() => root.defaultsApplyToExistingRequested())
                item.manageSetsRequested.connect((kind) => root.manageSetsRequested(kind))
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "ui"
            visible: active
            source: "../settings/TabGlobalUi.qml"
            onLoaded: {
                item.uiSettings = root.uiSettings
                item.categoryAddRequested.connect(() => root.categoryAddRequested())
                item.categoryEditRequested.connect((idx, entry) => root.categoryEditRequested(idx, entry))
                item.categoryDeleteRequested.connect((idx, entry) => root.categoryDeleteRequested(idx, entry))
                item.manageLogRulesRequested.connect(() => root.manageLogRulesRequested())
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "theme"
            visible: active
            source: "../settings/TabGlobalTheme.qml"
            onLoaded: {
                item.uiSettings = root.uiSettings
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "about"
            visible: active
            source: "../settings/TabGlobalAbout.qml"
            onLoaded: item.gameModel = root.gameModel
        }
    }
}
