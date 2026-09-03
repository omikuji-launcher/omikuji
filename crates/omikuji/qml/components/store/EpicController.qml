import QtQuick

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

    function syncDownloads() {
        if (!ctrl.downloadModel) {
            ctrl.activeDownloads = ({})
            return
        }
        try { ctrl.activeDownloads = JSON.parse(ctrl.downloadModel.source_state_json("epic")) || ({}) }
        catch (e) { ctrl.activeDownloads = ({}) }
    }

    Component.onCompleted: syncDownloads()
    onDownloadModelChanged: syncDownloads()

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
        function onState_changed() { ctrl.syncDownloads() }
        function onDownload_completed(id, source, appId, displayName, installPath, prefixPath, runnerVersion, dlcs) {
            if (source !== "epic" || !ctrl.gameModel) return
            let newId = ctrl.gameModel.epic_import_after_install(appId, displayName, prefixPath, runnerVersion, dlcs)
            if (newId && newId.length > 0 && ctrl.epicModel) ctrl.epicModel.refresh()
        }
    }
}
