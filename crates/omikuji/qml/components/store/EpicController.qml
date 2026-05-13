import QtQuick
import "../dialogs"

Item {
    id: ctrl

    anchors.fill: parent
    z: 900

    property var gameModel: null
    property var epicModel: null
    property var downloadModel: null
    property var defaults: null
    property int runnersVersion: 0

    property var activeDownloads: ({})

    signal installEnqueued()

    function showInstall(index) {
        dialog.gameIndex = index
        dialog.show()
    }

    EpicInstallDialog {
        id: dialog
        anchors.fill: parent
        gameModel: ctrl.gameModel
        epicModel: ctrl.epicModel
        runnersVersion: ctrl.runnersVersion
        defaults: ctrl.defaults
        onCancelled: hide()
        onInstallEnqueued: (id) => {
            hide()
            ctrl.installEnqueued()
        }
    }

    Connections {
        target: ctrl.downloadModel
        function onState_changed() {
            try { ctrl.activeDownloads = JSON.parse(ctrl.downloadModel.epic_state_json()) || ({}) }
            catch (e) { ctrl.activeDownloads = ({}) }
        }
        function onDownload_completed(id, source, appId, displayName, installPath, prefixPath, runnerVersion) {
            if (source !== "epic" || !ctrl.gameModel) return
            let newId = ctrl.gameModel.epic_import_after_install(appId, displayName, prefixPath, runnerVersion)
            if (newId && newId.length > 0 && ctrl.epicModel) ctrl.epicModel.refresh()
        }
    }
}
