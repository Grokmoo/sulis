# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/).

[Semantic Versioning](https://semver.org/spec/v2.0.0.html) is used with major version changes for breaking save game and data format compatibility.

## [1.0.0] - 2023-07-17
Given it has been a couple years since major work and saves should remain compatible from this point onwards, I've decided to go ahead and bump the version to 1.0.0.

### Added
- Scroll bars in the UI can now be dragged.
- Smooth map scrolling via keyboard keys and the mouse is now supported.
- Continue button on the main menu will load your most recent save.
- Added separate keybindings for scrolling the console history.

### Changed
- It will no longer be possible to drop (and thus potentially lose) key quest items.
- Character sprites should better fit in the box when building a character.
- Rebalanced several enemy types and boss encounters.
- Improved several Rogue abilities.
- Swap weapons now costs only one action point.
- Improved the flow and scripting for several Prologue events.
- Improved the Rock Armor and Air Shield abilities.
- Prevent situations where a dead hostile could still heal.

### Fixed
- Player characters will no longer be able to pathfind through unexplored terrain.
- Enemies making an attack or using an ability will now always be revealed on the map.
- End turn will not auto-trigger when a character is still performing animations.
- Logging to a file should now work as intended in all situations.
- Fixed some typos in various campaign text.
- Prevent some sequence breaks in the Prologue.
- All classes will now retain their abilities at very high levels.
- Improve frame rate limiting to prevent extremely high frame refresh rates.
- Fixed a rare crash caused by Fog of War calculations.

## [0.6.0] - 2021-01-15

### Added
- The third and final act of the Twin Expanse is now ready to play.
- Higher level legendary enchantments and items are now available.
- New sewer tileset and miscellaneous town tiles
- Several new abilities have been added to give more options for reducing enemy armor.
- The Disease status now has a proper icon and may be treated with healing kits.
- Injuries may now be removed with a healing kit.
- Script triggers may now be set to fire when a prop is activated.
- Script entity spawning now optionally takes the desired area.

### Changed
- General fixes and balancing to the Endless Dungeon campaign.
- Running away from combat is much more doable in most cases.
- Improved Rogue backstab ability substantially.
- Rebalanced a number of abilities and item bonuses.
- Improved display mode and resolution selection.
- Party members will no longer accrue injuries if reduced to 0 hit points outside of combat.
- Rebalanced weapon damage and armor penetration.  Two handed weapons do less damage.
- Reduced armor damage reduction, especially at high armor levels.
- Pruned a number of unneeded dependencies to reduce compilation time.
- AI script method name can now be overridden for more flexibility in the script.

### Fixed
- Larger monsters now draw in the correct order to prevent overlap issues.
- Scroll to player on area load now works correctly.
- Character text should no longer overlap on the load menu.
- Prevented an infinite XP exploit in a Vaalyun's quest conversation.
- The Grapple ability will now disable the target's ability use.
- Added missing sound effects on Dwarven items and the Flaming Fingers spell.
- OnAreaLoad script triggers should now fire multiple times when specified to do so.
- Fixed formatting and display of panic messages when a crash occurs.
- Fixed a rare crash when computing targets for an ability.
- Fixed a rare crash when calculating movement rate.

## [0.5.0] - 2020-05-05

### Added
- Full audio support for all areas, actions, and abilities.  Includes music and various sound effects.
- Players and enemies can now move through their allies during combat, as long as they end up in a valid spot
- Critical hit screen shake feature.  Can be disabled in Options.
- Scroll to the active character in combat.  Can be disabled in Options.
- The AI can now make use of almost all spells and abilities in an intelligent manner
- Show the player's level and class for all newly created save files

### Changed
- Balanced movement rate and movement animation speed for various monsters
- Modal abilities now enable and disable in a more intuitive way
- Script triggers may now fire multiple times if desired
- Default keybindings were changed to be less surprising for some players
- Reorganized the character sheet
- Improved logging and built in benchmarking 
- A large amount of internal code cleanup was done to bring things a bit closer to Rust best practices

### Fixed
- Fixed specific cases and added a general timeout to prevent combat becoming stuck on an AI turn
- Fix a case where procedural gen could create an unreachable room
- Prevent the player from leaving a targeter open when ending their turn
- Improved animation for move backs when movement almost ended on an invalid square
- An instance where the party was placed in the wrong location when moving between areas
- Prevent the player from gaining information on unexplored areas with the mouseover
- A rare crash that could occur when the mouse went out of screen bounds
- Some typos and issues in conversations and descriptive text

## [0.4.0] - 2020-02-14

### Added
- The second act of the Twin Expanse campaign is now fully playable
- New mid to high level spells and abilities for the different classes
- Larger and more powerful monsters with new abilities are present in Act 2.
- The game shows an indicator for ability range on hover
- Many new tiles and placeable features in the editor
- New keybindings for nearly every button on the main UI as well as mouse bindings
- Keybindings are now shown when hovering over buttons
- A random hint / tip is now shown on the main menu
- Items can now have cosmetic variations
- New animated props and randomized animation timings where appropriate

### Changed
- Reworked and expanded the AI with different profiles and much more intelligent behavior
- The level up process and associated screens have been overhauled
- The class and kit selection process has been reworked and expanded
- Rebalanced many weapons and abilities
- Reworked racial abilities and starting stats
- The game is much more intelligent about showing when abilities are disabled on the abilities bar
- Many improved animations for magical effects
- Improved handling, logging, and debugging for scripting errors
- Edge scrolling is now enabled by default.  This can be changed in the Main Menu Options
- Significant code refactoring and cleanup.  Much more to come.
- Texture cache now better allocates space for drawing entities

### Fixed
- Ability and attack range and the indicator preview are now computed consistently
- Out of date or invalid config files should be automatically recreated
- Transitions on the very edge of the map could cause problems in some cases
- Fixed OpenGL shader support on OS X
- The abilities bar now handles high level characters with many abilities
- Party members should no longer be bumped to impassable or invalid locations at combat start
- Inconsistent treatment of the Escape Key
- Stunned characters now have their AP restored at the end of combat
- Several bugs around prop passability / line of sight
- Rare crash when computing visibility
- Editor crash with very high or low elevations
- Overlapping props will no longer cause potential issues such as item loss

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
