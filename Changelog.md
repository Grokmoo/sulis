# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).

[Semantic Versioning](https://semver.org/spec/v2.0.0.html) is used with major version changes for breaking save game and data format compatibility.

## [0.2.0] - 2018-11-09
Data from the 0.1.x releases, such as save games and characters, will not work with this release.
### Added
- The first half of The Twin Expanse Act 1 is available.  New quests, encounters, characters, and items.
- New playable races, Dracon and Trollkin
- Quest Log and quest framework
- Campaign / World Map view and travel
- Resting, per-rest abilities, and stats persisting outside of combat
- Monster special abilities and resistances
- Expanded Item Adjectives with bonuses, icons, and stat changes
- A complete rework of asset loading, allowing asset overrides on a key by key basis
- Implemented basic modding support
- New art assets for many monster types
- Many new tiles for creating areas, including indoor and town areas
- Main Menu page showing credits / attribution and project links
- Improved configuration and confirmation process in Options menu, added scrolling and zoom config
- Character export functionality available from Character Sheet
- Scripting API docs are available at https://www.sulisgame.com/dev-modding
- Linking of multiple campaigns into a group

### Changed
- More robust AI scripting support
- Improved the script console output and history
- Cleaned up editor UI
- Balance changes to many abilities

### Fixed
- Fix many crash bugs associated with script interactions
- Fix UI related crashes
- Fixed tracking of actors that have been killed

## [0.1.1] - 2018-08-22
### Added
- Display options and keybindings are now configurable in the main menu
- UI zoom setting (mousewheel) is now saved
- Death and damage animations

### Changed
- Improved UI for mousing over objects in the main view
- Initiative is now a stat-influenced random roll at the start of combat
- Improved UI flow when first starting the game and creating a character
- All rogues now start with Hide

### Fixed
- Fix party members getting stuck on top of each other when starting combat
- Fix UI not getting updated from some item actions
- Twin Expanse campaign script fixes
- Mouse cursor and hover states now always up to date
- Repeated party move orders should no longer fail
- Fixed area visibility artifacts on party movement

## [0.1.0] - 2018-08-16
### Added
- Initial playable release.
- Includes a short (about two hours) campaign.
- Character Creation and Customization
- Turn based, party combat system
- Inventory, equipment, buying and selling of items
- Basic dialog system
- Fully scriptable combat AI, abilities, items, and triggers
