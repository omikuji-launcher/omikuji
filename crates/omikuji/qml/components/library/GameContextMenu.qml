import QtQuick
import "../dialogs"
import "../popups"

Item {
    id: ctrl
    anchors.fill: parent
    z: 2000

    property var gameModel: null

    // emitted for actions that need cross-cutting state changes in Main
    signal playRequested(int index)
    signal logsRequested(string gameId, string gameName)
    signal configureRequested(int index)
    signal categoriesRequested(int index)
    signal removeRequested(int index)

    function show(index, x, y) {
        ctrl._pendingIndex = index
        ctrl._pendingX = x
        ctrl._pendingY = y
        menu.close()
        delayTimer.start()
    }

    property int _pendingIndex: -1
    property real _pendingX: 0
    property real _pendingY: 0

    Timer {
        id: delayTimer
        interval: 100
        onTriggered: menu.setPosition(ctrl._pendingIndex, ctrl._pendingX, ctrl._pendingY)
    }

    ContextMenu {
        id: menu
        property int currentGameIndex: -1
        property var currentPrefixInfo: ({})

        function setPosition(index, mouseX, mouseY) {
            if (!ctrl.gameModel) return
            let game = ctrl.gameModel.get_game(index)
            if (!game) return

            let pinfo = {}
            try { pinfo = JSON.parse(ctrl.gameModel.game_prefix_info(index) || "{}") } catch (e) { pinfo = {} }
            currentPrefixInfo = pinfo

            let isFav = game.favourite || false
            let isHidden = game.hidden || false
            let isEpic = game.sourceKind === "epic" && game.sourceAppId && game.sourceAppId.length > 0
            let isGog = game.sourceKind === "gog" && game.sourceAppId && game.sourceAppId.length > 0
            let hasDesktopShortcut = ctrl.gameModel.has_desktop_shortcut(index)
            let hasMenuShortcut = ctrl.gameModel.has_menu_shortcut(index)

            let shortcuts = [
                { text: hasDesktopShortcut ? qsTr("Remove desktop shortcut") : qsTr("Create desktop shortcut"), action: "desktop_shortcut" },
                { text: hasMenuShortcut ? qsTr("Remove application menu shortcut") : qsTr("Create application menu shortcut"), action: "menu_shortcut" }
            ]
            if (ctrl.gameModel.steam_shortcut_available(index)) {
                let hasSteamShortcut = ctrl.gameModel.has_steam_shortcut(index)
                shortcuts.push({ text: hasSteamShortcut ? qsTr("Remove Steam shortcut") : qsTr("Create Steam shortcut"), action: "steam_shortcut" })
            }

            let built = [
                { text: qsTr("Play"), action: "play" },
                { text: qsTr("Show logs"), action: "logs" },
                { text: qsTr("Configure"), action: "configure" },
                { text: qsTr("Categories"), action: "categories" },
                { text: qsTr("Browse files"), action: "browse" },
                { text: isFav ? qsTr("Remove from favorites") : qsTr("Add to favorites"), action: "favorite" },
                { text: isHidden ? qsTr("Unhide") : qsTr("Hide"), action: "hide" },
                { text: qsTr("Shortcuts"), submenu: shortcuts },
                { text: qsTr("Duplicate"), action: "duplicate" }
            ]
            if (isEpic || isGog) {
                built.push({ text: qsTr("Check for updates"), action: "check_update", accent: true })
            }
            if (ctrl.gameModel.game_supports_repair(game.gameId)) {
                built.push({ text: qsTr("Repair"), action: "repair", accent: true })
            }
            if (isEpic) {
                built.push({ text: qsTr("Uninstall (Epic Games)"), action: "uninstall_store", danger: true })
            } else if (isGog) {
                built.push({ text: qsTr("Uninstall (GOG)"), action: "uninstall_store", danger: true })
            }
            let removeItem = { text: qsTr("Remove"), action: "remove", danger: true }
            if (pinfo.hasPrefix) {
                removeItem.shiftText = qsTr("Remove + prefix")
                removeItem.shiftAction = "remove_prefix"
            }
            built.push(removeItem)
            items = built

            currentGameIndex = index
            openAtCursor(mouseX, mouseY)
        }

        onItemClicked: (action) => {
            let idx = menu.currentGameIndex
            if (idx < 0 || !ctrl.gameModel) return

            switch (action) {
                case "play":
                    ctrl.playRequested(idx)
                    break
                case "logs": {
                    let g = ctrl.gameModel.get_game(idx)
                    if (g && g.gameId) ctrl.logsRequested(g.gameId, g.name || g.gameId)
                    break
                }
                case "configure":
                    ctrl.configureRequested(idx)
                    break
                case "categories":
                    ctrl.categoriesRequested(idx)
                    break
                case "browse":
                    ctrl.gameModel.browse_files(idx)
                    break
                case "favorite": {
                    let game = ctrl.gameModel.get_game(idx)
                    let newFav = !(game.favourite || false)
                    ctrl.gameModel.update_game_field(idx, "meta.favourite", newFav ? "true" : "false")
                    ctrl.gameModel.save_game(game.gameId)
                    break
                }
                case "hide": {
                    let game = ctrl.gameModel.get_game(idx)
                    let newHidden = !(game.hidden || false)
                    ctrl.gameModel.update_game_field(idx, "meta.hidden", newHidden ? "true" : "false")
                    ctrl.gameModel.save_game(game.gameId)
                    break
                }
                case "desktop_shortcut":
                    if (ctrl.gameModel.has_desktop_shortcut(idx)) ctrl.gameModel.remove_desktop_shortcut(idx)
                    else ctrl.gameModel.create_desktop_shortcut(idx)
                    break
                case "menu_shortcut":
                    if (ctrl.gameModel.has_menu_shortcut(idx)) ctrl.gameModel.remove_menu_shortcut(idx)
                    else ctrl.gameModel.create_menu_shortcut(idx)
                    break
                case "steam_shortcut":
                    if (ctrl.gameModel.has_steam_shortcut(idx)) ctrl.gameModel.remove_steam_shortcut(idx)
                    else ctrl.gameModel.create_steam_shortcut(idx)
                    break
                case "duplicate":
                    ctrl.gameModel.duplicate_game(idx)
                    break
                case "remove":
                    ctrl.gameModel.remove_game(idx)
                    ctrl.removeRequested(idx)
                    break
                case "remove_prefix": {
                    let g = ctrl.gameModel.get_game(idx)
                    let info = menu.currentPrefixInfo || {}
                    let nm = (g && g.name) ? g.name : qsTr("this game")
                    let others = (info.gameCount || 1) - 1
                    removeWithPrefixConfirm.title = qsTr("Remove %1 + prefix?").arg(nm)
                    removeWithPrefixConfirm.message = others > 0
                        ? qsTr("This removes %1 and deletes its prefix. %n other game(s) use this prefix and will lose it too. It won't be recoverable.", "", others).arg(nm)
                        : qsTr("This removes %1 and deletes its prefix. It won't be recoverable.").arg(nm)
                    removeWithPrefixConfirm.detail = info.path || (g && g.prefixPath) || ""
                    removeWithPrefixConfirm.show({ idx: idx })
                    break
                }
                case "uninstall_store": {
                    let g = ctrl.gameModel.get_game(idx)
                    if (g && g.gameId) {
                        uninstallConfirm.title = qsTr("Uninstall %1?").arg(g.name || qsTr("this game"))
                        uninstallConfirm.message = g.sourceKind === "gog"
                            ? qsTr("The game files will be deleted from disk. This cannot be undone.")
                            : qsTr("Legendary will delete the game files from disk. This cannot be undone.")
                        uninstallConfirm.show({ id: g.gameId, kind: g.sourceKind })
                    }
                    break
                }
                case "check_update": {
                    let g = ctrl.gameModel.get_game(idx)
                    if (g && g.gameId) {
                        if (g.sourceKind === "gog") ctrl.gameModel.check_gog_update(g.gameId)
                        else ctrl.gameModel.check_epic_update(g.gameId)
                    }
                    break
                }
                case "repair": {
                    let g = ctrl.gameModel.get_game(idx)
                    if (g && g.gameId) ctrl.gameModel.enqueue_game_repair(g.gameId)
                    break
                }
            }
        }
    }

    ConfirmDialog {
        id: uninstallConfirm
        anchors.fill: parent
        confirmText: qsTr("Uninstall")
        cancelText: qsTr("Keep")
        destructive: true
        onConfirmed: (payload) => {
            if (!payload || !payload.id || !ctrl.gameModel) return
            if (payload.kind === "gog") ctrl.gameModel.gog_uninstall(payload.id)
            else ctrl.gameModel.epic_uninstall(payload.id)
        }
    }

    ConfirmDialog {
        id: removeWithPrefixConfirm
        anchors.fill: parent
        confirmText: qsTr("Remove + delete")
        cancelText: qsTr("Cancel")
        destructive: true
        onConfirmed: (payload) => {
            if (payload && ctrl.gameModel) {
                ctrl.gameModel.remove_game_with_prefix(payload.idx)
                ctrl.removeRequested(payload.idx)
            }
        }
    }
}
