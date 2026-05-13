import QtQuick
import "../dialogs"

Item {
    id: ctrl

    anchors.fill: parent
    z: 900

    property var gameModel: null
    property var gogModel: null
    property var downloadModel: null
    property var defaults: null
    property int runnersVersion: 0

    property var activeDownloads: ({})

    signal installEnqueued()

    function showInstall(index) {
        dialog.gameIndex = index
        dialog.show()
    }

    GogInstallDialog {
        id: dialog
        anchors.fill: parent
        gameModel: ctrl.gameModel
        gogModel: ctrl.gogModel
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
            try { ctrl.activeDownloads = JSON.parse(ctrl.downloadModel.gog_state_json()) || ({}) }
            catch (e) { ctrl.activeDownloads = ({}) }
        }
        function onDownload_completed(id, source, appId, displayName, installPath, prefixPath, runnerVersion) {
            if (source !== "gog" || !ctrl.gameModel) return
            let newId = ctrl.gameModel.gog_import_after_install(appId, displayName, prefixPath, runnerVersion)
            if (newId && newId.length > 0 && ctrl.gogModel) ctrl.gogModel.refresh()
        }
    }
}
