import QtQuick
import "../dialogs"

Item {
    id: ctrl

    anchors.fill: parent
    z: 900

    property var gameModel: null
    property var downloadModel: null
    property var defaults: null
    property int runnersVersion: 0

    signal installEnqueued()

    function showInstall(manifestId) {
        dialog.manifestId = manifestId
        dialog.show()
    }

    GachaInstallDialog {
        id: dialog
        anchors.fill: parent
        gameModel: ctrl.gameModel
        downloadModel: ctrl.downloadModel
        runnersVersion: ctrl.runnersVersion
        defaults: ctrl.defaults
        onCancelled: hide()
        onInstallEnqueued: (id) => {
            hide()
            ctrl.installEnqueued()
        }
        onImported: (gameId) => hide()
    }

    Connections {
        target: ctrl.downloadModel
        function onDownload_completed(id, source, appId, displayName, installPath, prefixPath, runnerVersion) {
            if (!ctrl.gameModel) return
            if (source === "epic" || source === "gog") return
            let raw = ctrl.gameModel.gacha_manifest_for_app_id(appId)
            if (!raw || raw.length === 0) {
                console.warn("[gacha] no manifest for", source, "app_id:", appId)
                return
            }
            let m = null
            try { m = JSON.parse(raw) } catch (e) { m = null }
            if (m) {
                ctrl.gameModel.gacha_import_after_install(
                    m.manifest_id, m.edition_id, displayName,
                    installPath, runnerVersion, prefixPath
                )
            }
        }
    }
}
