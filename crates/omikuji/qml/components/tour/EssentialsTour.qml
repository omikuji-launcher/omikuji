import QtQuick
import omikuji 1.0

QtObject {
    required property var app
    required property var tour

    readonly property string tourId: "essentials"
    readonly property Component briefcaseSpirit: Component { BriefcaseSpirit {} }

    readonly property var steps: [
        {
            keys: ["library.view"],
            title: qsTr("Your library"),
            body: qsTr("Every game you add or install from a store ends up here.")
        },
        {
            keys: ["topbar.add"],
            title: qsTr("Adding a game"),
            body: qsTr("Got a game that isn't from a store? Press + to add it."),
            until: () => tour.has("add.game")
        },
        {
            keys: ["add.game"],
            title: qsTr("Adding a game"),
            body: qsTr("Pick 'Add game' to set it up by hand."),
            until: () => app.activeModal === "addGame"
        },
        {
            keys: ["game.name"],
            title: qsTr("Game's name"),
            body: qsTr("This is what shows up on its card in the library.")
        },
        {
            keys: ["game.runner_type"],
            title: qsTr("What kind of game is this?"),
            body: qsTr("'Wine' runs Windows games, 'Native' runs Linux binaries, 'Steam' and 'Flatpak' run the game from them. We'll use 'Wine' for this tour!")
        },
        {
            keys: ["rail.runner"],
            title: qsTr("Runner options"),
            body: qsTr("What the game launches and what it runs through live in the Runner tab."),
            hint: qsTr("Open the Runner tab"),
            until: () => tour.has("game.exe")
        },
        {
            keys: ["game.exe"],
            title: qsTr("The executable path"),
            body: qsTr("Add the path of the executable that starts the game.")
        },
        {
            keys: ["game.runner"],
            title: qsTr("Pick a runner"),
            body: qsTr("The Wine or Proton build the game runs with. Do you have any installed yet?")
        },
        {
            keys: ["game.layers"],
            title: qsTr("Translation layers"),
            body: qsTr("Layers are on by default. Turning one off really disables it, so only touch them if you know what you're doing."),
            note: qsTr("You can also pick which version each layer uses, the docs explain it better.")
        },
        {
            keys: ["rail.system"],
            title: qsTr("System options"),
            body: qsTr("Here you can find general game options."),
            note: qsTr("Environment variables, Discord RPC, GameMode, MangoHud, etc."),
            hint: qsTr("Open the System tab"),
            until: () => tour.has("game.env")
        },
        {
            keys: ["game.env"],
            title: qsTr("Environment variables"),
            body: qsTr("Here you can add or remove environment variables."),
            note: qsTr("Left field is the variable name, right field is the value.")
        },
        {
            keys: [],
            title: qsTr("Your turn"),
            body: qsTr("Fill in the fields shown if you wish to add a game right now or close the dialog to carry on with the tour."),
            until: () => app.activeModal === ""
        },
        {
            keys: ["nav.stores"],
            title: qsTr("Stores"),
            body: qsTr("Stores behave a little differently, but nothing too special."),
            note: qsTr("Steam is read straight from your config. Epic, GOG and Amazon show their libraries once you log in. Gachas install directly.")
        },
        {
            keys: ["nav.store.epic"],
            title: qsTr("Let's peek at Epic"),
            body: qsTr("We'll look at Epic Games, it's the most common one and behaves like GOG and Amazon."),
            hint: qsTr("Open Epic Games"),
            until: () => app.currentView === "epic"
        },
        {
            keys: ["store.account"],
            title: qsTr("The login"),
            body: qsTr("Press 'Open Login Page', log in and paste the authorization code it gives you."),
            note: qsTr("If the store's tool is missing, an 'Install' button shows up here first. Once you're logged in, your library shows up here as cards, ready to install or import.")
        },
        {
            keys: ["demo.store"],
            demo: "epic",
            title: qsTr("Store's library"),
            body: qsTr("This is how your games would look. Of course your games will never beat these ones.")
        },
        {
            keys: ["demo.store.card"],
            demo: "epic",
            title: qsTr("Installing a game"),
            body: qsTr("Press '+' on a card to open the install/import dialog."),
            hint: qsTr("Press + or Enter on the card"),
            until: () => tour.has("install.path")
        },
        {
            keys: ["install.path"],
            demo: "epic",
            title: qsTr("Installation path"),
            body: qsTr("The folder the game gets installed into.")
        },
        {
            keys: ["install.prefix_runner"],
            demo: "epic",
            title: qsTr("Prefix and runner"),
            body: qsTr("Where the game's Wine prefix lives and the runner to run the game with.")
        },
        {
            keys: ["install.dlc"],
            demo: "epic",
            title: qsTr("DLC"),
            body: qsTr("Pick which DLC to install alongside the game, if any available.")
        },
        {
            keys: ["install.confirm"],
            demo: "epic",
            title: qsTr("Install"),
            body: qsTr("This queues the download. Once it's done, the game shows up in your library with the options you picked here."),
            note: qsTr("We won't press it, clearly.")
        },
        {
            keys: ["install.cancel"],
            demo: "epic",
            title: qsTr("Close it"),
            body: qsTr("Let's close this for now."),
            hint: qsTr("Press Cancel"),
            until: () => !tour.has("install.path")
        },
        {
            keys: ["demo.store.moyu"],
            demo: "epic",
            title: qsTr("Importing a game"),
            body: qsTr("A game from a store (Epic Games, GOG, Amazon or Gachas) has to be imported from that store's page, otherwise Omikuji can't keep track of it."),
            note: qsTr("That covers its state, updates, launching, etc."),
            hint: qsTr("Press + or Enter on the card"),
            until: () => tour.has("install.path")
        },
        {
            keys: ["install.path"],
            demo: "epic",
            title: qsTr("Point it at the game's folder"),
            body: qsTr("Point the installation path at the game's folder on disk so Omikuji can import it."),
            note: qsTr("If Epic, GOG or Amazon already know about the game (their JSON registries), the path is filled in for you. For Gachas, point the path, pick the game's region and you're done.")
        },
        {
            keys: ["install.existing"],
            demo: "epic",
            title: qsTr("Found it"),
            body: qsTr("This shows up when game files are found at that path.")
        },
        {
            keys: ["install.confirm"],
            demo: "epic",
            title: qsTr("Import"),
            body: qsTr("If the game can be imported, the button turns into 'Import', which adds it to your library.")
        },
        {
            keys: [],
            centered: true,
            art: briefcaseSpirit,
            title: qsTr("Enjoy Omikuji~"),
            body: qsTr("Don't fret if you meet Moyu around. He's overworking.")
        }
    ]
}
