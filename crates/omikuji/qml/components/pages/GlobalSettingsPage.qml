import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "../widgets"
import "../settings"

// every knob is a live apply* call, no save/cancel flow
Item {
    id: root

    property var uiSettings: null
    property var componentsBridge: null
    property var archiveManager: null
    property var defaults: null
    property var gameModel: null
    property var activeInstalls: ({})

    // bubbles to Main.qml which owns teh dialog, full-window dim needs a root-level sibling
    signal manageRequested(string category, string source, string kind)

    signal categoryAddRequested()
    signal categoryEditRequested(int index, var entry)
    signal categoryDeleteRequested(int index, var entry)

    signal defaultsApplyToExistingRequested()

    property var tabs: [
        { label: "Components", kind: "components" },
        { label: "Defaults",   kind: "defaults"   },
        { label: "Interface",  kind: "ui"         },
        { label: "Theme",      kind: "theme"      },
        { label: "About",      kind: "about"      }
    ]
    property int currentTabIndex: 0
    readonly property string currentKind:
        tabs[currentTabIndex] ? tabs[currentTabIndex].kind : "components"

    Item {
        id: contentHost
        property bool isDropdownHost: true
        anchors.fill: parent

        Flickable {
            id: contentFlick
            anchors.fill: parent
            contentHeight: contentCol.height + 40
            clip: true
            boundsBehavior: Flickable.StopAtBounds

            Column {
                id: contentCol
                anchors.top: parent.top
                anchors.topMargin: 20
                anchors.left: parent.left
                anchors.leftMargin: 48
                anchors.right: parent.right
                anchors.rightMargin: 48
                spacing: 0

                Loader {
                    id: componentsTabLoader
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
                }
            }
        }
    }
}
