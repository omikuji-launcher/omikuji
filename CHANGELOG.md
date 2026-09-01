## This is an example. Add new versions changelog below this.
Added:
- added three hundred images of Yangyang.
- added button to open a massive cake and I won't elaborate on the kind.

Fixes:
- fixed runners being useless to the society.
- fixed turtle shells being too weak.

Changed:
- changed how human anatomy works.

Removed:
- removed 45% of the working code.

## 0.13.0
Added:
- '-Latest' runner builds system
- Steam exposure for '-Latest' runners, with compatibilitytools.vdf naming so Steam
  doesn't deselect them on update
- Release filter for runner sources
- 'Run alongside' field for games with its own args field and gacha manifest support
- 'Check for updates on app launch' toggle working for gacha games too
- 'Look it up on ProtonDB' button for GOG and Epic games (behind `show details`)
- Art preview button in game settings
- Note chips for existing files in install dialogs
- Toggle to ignore Steam-provided runners

Changed:
- Bookmark symbol for games already in the library (GOG and Epic)
- Cuter download chip on cards, and proper resuming from dialog
- Welcome page tweaks
- Minor UI tweaks
- Flatter flatpak repo
- Documentation site rewritten around a full usage guide

Fixes:
- Zoom shortcuts doing nothing
- Portal name

## 0.12.0
Added:
- Epic and GOG DLCs installation flow
- Proper Error chip with scroll grab for dialogs
- Proper fade and chevrons on scrollable items
- System notification on completed download toggle
- URL regex and registry exe lookup for community scripts

Changed:
- Proper m3 style checkboxes
- Ofuda's settings page restyle (again)

## 0.11.0
Added:
- "run --notify-gui" cli flag for cross-process run state (updates, errors, etc.)
- Downloads bandwidth limit and various download params

Changed:
- Settings file ui.toml changed to app.toml
- Settings page layout improved (new tab for behaviour settings)

## 0.10.0
Added:
- Yostar gacha integration (Arknights Global, KR, JP)
- Library categories' context menu and drag function
- Runtime components page 'check updates' button
- Delete button for runtime components
- Welcome dialog

Fixes:
- Console mode pause shader when not focused

Changed:
- Don't fetch umu-run automatically at app startup
- Keep missing runner and layer label in game/app settings
- States ui.toml migration

## 0.9.0
Added:
- Guard for duplicate game launches and wine tools spawns
- Per-game layers (dxvk, vkd3d, etc.) versions
- Spritz-wine as a default runner seed
- 'when' field for community scripts
- 'starting' play button state for gog, epic and gachas updates check
- Changelogs dialog
- '--talk-name=org.kde.StatusNotifierWatcher' for flatpak build

Changed:
- App icon
- Wrapped default library categories with qsTr for translation
- Base proton_verb on wheter the game is running
- better import system for gog, epic and gachas
