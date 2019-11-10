# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).

[Semantic Versioning](https://semver.org/spec/v2.0.0.html) is used with major version changes for breaking save game and data format compatibility.

## [0.3.1] - 2019-11-09

### Added
- Two new playable classes - the Warlock and Bard, both with new associated mechanics and stats
- Signficantly increased item and bonus variety in all loot
- While in combat, your selected movement path is now previewed before you move
- While in combat, your remaining AP after a given action is now previewed
- A number of new scripting hooks and functions added to the Lua interface

### Changed
- Reworked class kits to give more customization at character creation
- Reworked the companions in the Twin Expanse to fit together better
- Improved dialog flow and highlight the current speaker
- Reworked several abilities in the general ability tree
- Most abilities can now only be activated in combat

### Fixed
- Characters with zero AP still get a turn
- Fixed Male / Female selection in Character Creator (thanks shmutalov!)
- A number of breaking issues in the Twin Expanse Act 1 were resolved

## [0.3.0] - 2019-09-09
This version is save compatible with 0.2.x.
### Added
- Procedural dungeon generator and a dungeon crawl campaign, The Endless Dungeon
- It is now possible to run away from combat (as long as you are fast enough)
- on_tick and and on_round script triggers
- Logger output in user config file is more flexible
- Feature editor type in the Editor
- The default zoom level is now configurable
- Attacks and abilities now show their range when being activated

### Changed
- Better combat feedback and icons to represent different attack types
- Improved character sheet
- Dialog window has a much improved UI flow and better focus
- Better lua script sandboxing and instruction counting
- Improved script performance
- Rewrote theme system from scratch for more extensibility and integration into resources system
- Reorganized code to remove under-utilized modules

### Fixed
- Hide ability was not working correctly with attacks
- UI tweaks to improve movement to neighboring tiles
- Fixed mouseover flicker that occurred on some display drivers
- Several Keyboard focus issues fixed

## [0.2.2] - 2019-01-19
This version remains save compatible with 0.2.x.
### Added
- The Twin Expanse Act 1 is now fully playable all the way through
- Reworked ability trees and added new abilities for all classes
- A variety of new loot modifiers (magic items) and improved loot lists
- A wounds system for when party members fall in combat
- Day / Night cycle and time tracking for travel times
- Merchant inventories now refresh over time
- Random name generator in character customization
- A new Accessory filter in item windows
- Custom icons show up on the character portrait for permanent effects
- Editor ease of use features, area properties, transition improvements
- Many new area and creature assets

### Changed
- Reworked and rebalanced how Attributes influence various stats
- Performed an initial balance pass on Races, Classes, and Abilities
- User interface layout, appearance, and modal improvements
- Cleaned up and rebalanced previously released campaign content
- Use base64 in several places in area data files to massively decrease file size

### Fixed
- Many bug fixes in ability, item, and campaign scripts
- Color deserialization now correctly allows for maximum value
- UI Block script method now actually blocks the UI
- Creatures will no longer appear invisible after many other creatures have been loaded
- Move then attack and similar actions now calculate distance correctly
- Enemies now have quick items removed on use
- Several crash fixes in area transition and particle generator code

## [0.2.1] - 2018-12-06
This version should be save compatible with 0.2.0.
### Added
- A new playable character class, the Druid, is available, with a brand new ability tree
- Summoning and shapeshifting abilities for the Druid
- Several new weapons including druid themed weapons
- Several new mage abilities have been added
- Many new script hooks are available for new content and mods

### Changed
- Reworked critical hits to only occur on a high natural roll, for more exciting combat
- Reworked armor to cap damage reduction and make high armor somewhat less powerful
- Hidden characters will no longer automatically trigger combat
- Several lists and panels that did not have scrolling now have it
- The player's AP bar now shows fractional AP to allow you to see how much movement you have available
- Damage type is now indicated via the feedback color on hits
- Improved how affected targets are determined for area of effect spells
- You can now use the character portraits as targets for abilities

### Fixed
- Flanking calculations were considering ranged attacks incorrectly
- Several edge case bugs associated with calculating player visibility
- Fixed some annoying behavior around attack to move when AP is low
- Attack and ability usage feedback text is more consistent
- Many fixes to abilities that were broken or had unintentional effects

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
