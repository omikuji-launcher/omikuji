import QtQuick
import "../dialogs"

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

        function setPosition(index, mouseX, mouseY) {
            if (!ctrl.gameModel) return
            let game = ctrl.gameModel.get_game(index)
            if (!game) return

            let isFav = game.favourite || false
            let isEpic = game.sourceKind === "epic" && game.sourceAppId && game.sourceAppId.length > 0
            let isGog = game.sourceKind === "gog" && game.sourceAppId && game.sourceAppId.length > 0
            let hasDesktopShortcut = ctrl.gameModel.has_desktop_shortcut(index)
            let hasMenuShortcut = ctrl.gameModel.has_menu_shortcut(index)

            let built = [
                { text: "Play", action: "play" },
                { text: "Show logs", action: "logs" },
                { text: "Configure", action: "configure" },
                { text: "Categories", action: "categories" },
                { text: "Browse files", action: "browse" },
                { text: isFav ? "Remove from favorites" : "Add to favorites", action: "favorite" },
                { text: hasDesktopShortcut ? "Remove desktop shortcut" : "Create desktop shortcut", action: "desktop_shortcut" },
                { text: hasMenuShortcut ? "Remove application menu shortcut" : "Create application menu shortcut", action: "menu_shortcut" },
                { text: "Duplicate", action: "duplicate" }
            ]
            if (isEpic || isGog) {
                built.push({ text: "Check for updates", action: "check_update", accent: true })
            }
            if (isEpic) {
                built.push({ text: "Uninstall (Epic Games)", action: "uninstall_epic", danger: true })
            }
            built.push({ text: "Remove", action: "remove", danger: true })
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
                case "desktop_shortcut":
                    if (ctrl.gameModel.has_desktop_shortcut(idx)) ctrl.gameModel.remove_desktop_shortcut(idx)
                    else ctrl.gameModel.create_desktop_shortcut(idx)
                    break
                case "menu_shortcut":
                    if (ctrl.gameModel.has_menu_shortcut(idx)) ctrl.gameModel.remove_menu_shortcut(idx)
                    else ctrl.gameModel.create_menu_shortcut(idx)
                    break
                case "duplicate":
                    ctrl.gameModel.duplicate_game(idx)
                    break
                case "remove":
                    ctrl.gameModel.remove_game(idx)
                    ctrl.removeRequested(idx)
                    break
                case "uninstall_epic": {
                    let g = ctrl.gameModel.get_game(idx)
                    if (g && g.gameId) {
                        epicUninstallConfirm.title = "Uninstall " + (g.name || "this game") + "?"
                        epicUninstallConfirm.message = "Legendary will delete the game files from disk. This cannot be undone."
                        epicUninstallConfirm.show({ id: g.gameId })
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
            }
        }
    }

    ConfirmDialog {
        id: epicUninstallConfirm
        anchors.fill: parent
        confirmText: "Uninstall"
        cancelText: "Keep"
        destructive: true
        onConfirmed: (payload) => {
            if (payload && payload.id && ctrl.gameModel) ctrl.gameModel.epic_uninstall(payload.id)
        }
    }
}
