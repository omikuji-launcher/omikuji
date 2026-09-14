import QtQuick
import omikuji 1.0

// every knob is a live apply* call, no save/cancel flow
Item {
    id: root

    property var appSettings: null
    property var componentsBridge: null
    property var archiveManager: null
    property var ofudaBridge: null
    property var defaults: null
    property var gameModel: null
    property var activeInstalls: ({})

    // bubbles to Main.qml which owns teh dialog, full-window dim needs a root-level sibling
    signal manageRequested(string category, string source, string kind)
    signal addSourceRequested(string category)
    signal manageFoundRunnersRequested()

    signal categoryAddRequested()
    signal categoryEditRequested(int index, var entry)
    signal categoryDeleteRequested(int index, var entry)
    signal manageLogRulesRequested()

    signal defaultsApplyToExistingRequested()
    signal manageSetsRequested(string kind)
    signal manageFontSizesRequested()
    signal manageRadiiRequested()

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
        { label: qsTr("App"),        kind: "app",        icon: "dataset" },
        { label: qsTr("Interface"),  kind: "ui",         icon: "tune" },
        { label: qsTr("Defaults"),   kind: "defaults",   icon: "settings" },
        { label: qsTr("Presets"),    kind: "presets",    icon: "view_list" },
        { label: qsTr("Components"), kind: "components", icon: "layers" },
        { label: "Ofuda",            kind: "ofuda",      icon: "ofuda" },
        { label: qsTr("Theme"),      kind: "theme",      icon: "imagesmode", pinned: true },
        { label: qsTr("About"),      kind: "about",      icon: "verified",   pinned: true }
    ]
    property int currentTabIndex: 0
    readonly property string currentKind:
        tabs[currentTabIndex] ? tabs[currentTabIndex].kind : "app"

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
            sourceComponent: TabGlobalComponents {
                componentsBridge: root.componentsBridge
                archiveManager: root.archiveManager
                activeInstalls: root.activeInstalls
                onManageRequested: (category, source, kind) => root.manageRequested(category, source, kind)
                onAddSourceRequested: (category) => root.addSourceRequested(category)
                onManageFoundRunnersRequested: root.manageFoundRunnersRequested()
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "ofuda"
            visible: active
            sourceComponent: TabGlobalOfuda {
                ofudaBridge: root.ofudaBridge
                appSettings: root.appSettings
                onOpenRequested: (prefix) => root.prefixOpenRequested(prefix)
                onCreateRequested: root.prefixCreateRequested()
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "defaults"
            visible: active
            sourceComponent: TabGlobalDefaults {
                defaults: root.defaults
                gameModel: root.gameModel
                appSettings: root.appSettings
                onApplyToExistingRequested: root.defaultsApplyToExistingRequested()
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "presets"
            visible: active
            sourceComponent: TabGlobalPresets {
                onManageSetsRequested: (kind) => root.manageSetsRequested(kind)
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "ui"
            visible: active
            sourceComponent: TabGlobalUi {
                appSettings: root.appSettings
                onCategoryAddRequested: root.categoryAddRequested()
                onCategoryEditRequested: (index, entry) => root.categoryEditRequested(index, entry)
                onCategoryDeleteRequested: (index, entry) => root.categoryDeleteRequested(index, entry)
                onManageLogRulesRequested: root.manageLogRulesRequested()
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "app"
            visible: active
            sourceComponent: TabGlobalApp {
                appSettings: root.appSettings
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "theme"
            visible: active
            sourceComponent: TabGlobalTheme {
                appSettings: root.appSettings
                onManageFontSizesRequested: root.manageFontSizesRequested()
                onManageRadiiRequested: root.manageRadiiRequested()
            }
        }

        Loader {
            width: parent.width
            active: root.currentKind === "about"
            visible: active
            sourceComponent: TabGlobalAbout {
                gameModel: root.gameModel
            }
        }
    }
}
