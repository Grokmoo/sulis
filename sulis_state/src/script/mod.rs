//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

mod area_targeter;
pub use self::area_targeter::AreaTargeter;

mod script_ability;
pub use self::script_ability::ScriptAbility;
pub use self::script_ability::ScriptAbilitySet;

pub mod script_callback;
pub use self::script_callback::CallbackData;
pub use self::script_callback::ScriptCallback;
pub use self::script_callback::ScriptHitKind;
pub use self::script_callback::FuncKind;

mod script_effect;
use self::script_effect::ScriptEffect;
use self::script_effect::ScriptActiveSurface;

mod script_entity;
pub use self::script_entity::ScriptEntity;
pub use self::script_entity::ScriptEntitySet;

mod script_inventory;
pub use self::script_inventory::ScriptInventory;
pub use self::script_inventory::ScriptUsableItem;

mod script_color_animation;
use self::script_color_animation::ScriptColorAnimation;

mod script_particle_generator;
use self::script_particle_generator::ScriptParticleGenerator;

mod script_subpos_animation;
use self::script_subpos_animation::ScriptSubposAnimation;

pub mod targeter;
use self::targeter::TargeterData;

use std;
use std::rc::Rc;
use std::cell::RefCell;

use rlua::{self, Function, Lua, UserData, UserDataMethods};

use sulis_core::config::Config;
use sulis_core::util::{Point};
use sulis_rules::QuickSlot;
use sulis_module::{ability, Ability, Item, Module, OnTrigger, on_trigger};
use {EntityState, ItemState, GameState, ai, area_feedback_text::ColorKind, quest_state,
    animation::Anim, Location};

type Result<T> = std::result::Result<T, rlua::Error>;

pub struct ScriptState {
    lua: Lua,
}

impl ScriptState {
    pub fn new() -> ScriptState {
        let lua = Lua::new();

        {
            let globals = lua.globals();
            match globals.set("game", ScriptInterface {}) {
                Ok(()) => (),
                Err(e) => {
                    warn!("Error setting up Lua globals");
                    warn!("{}", e);
                },
            }
        }

        ScriptState { lua }
    }

    fn execute_script<'lua, T: rlua::FromLuaMulti<'lua>>(&'lua self,
                                                         parent: &Rc<RefCell<EntityState>>,
                                                         function_args: &str,
                                                         mut script: String,
                                                         function: &str) -> Result<T> {
        let globals = self.lua.globals();
        globals.set("parent", ScriptEntity::from(&parent))?;

        debug!("Loading script for '{}'", function);

        script.push('\n');
        script.push_str(function);
        script.push_str(function_args);
        let func: Function = self.lua.load(&script, Some(function))?;

        debug!("Calling script function '{}'", function);

        func.call::<_, T>("")
    }

    pub fn console(&self, script: String, party: &Vec<Rc<RefCell<EntityState>>>) -> Result<String> {
        assert!(party.len() > 0);
        self.lua.globals().set("player", ScriptEntity::from(&party[0]))?;

        let party_table = self.lua.create_table()?;
        for (index, member) in party.iter().enumerate() {
            party_table.set(index + 1, ScriptEntity::from(member))?;
        }

        self.lua.globals().set("party", party_table)?;

        self.lua.eval::<_, String>(&script, None)
}

    pub fn ai_script(&self, parent: &Rc<RefCell<EntityState>>, func: &str) -> Result<ai::State> {
        let script = match &parent.borrow().actor.actor.ai {
            None => unreachable!(),
            Some(ai_template) => ai_template.script(),
        };

        self.lua.globals().set("state", ai::State::End)?;
        self.execute_script(parent, "(parent, state)", script, func)?;

        self.lua.globals().get("state")
    }

    pub fn item_on_activate(&self, parent: &Rc<RefCell<EntityState>>,
                            kind: ScriptItemKind) -> Result<()> {
        let t: Option<(&str, usize)> = None;
        self.item_script(parent, kind,
                         ScriptEntitySet::new(parent, &Vec::new()), t, "on_activate")
    }

    pub fn item_on_target_select(&self,
                                 parent: &Rc<RefCell<EntityState>>,
                                 kind: ScriptItemKind,
                                 targets: Vec<Option<Rc<RefCell<EntityState>>>>,
                                 selected_point: Point,
                                 affected_points: Vec<Point>,
                                 func: &str,
                                 custom_target: Option<Rc<RefCell<EntityState>>>) -> Result<()> {
        let mut targets = ScriptEntitySet::new(parent, &targets);
        targets.selected_point = Some((selected_point.x, selected_point.y));
        let arg = match custom_target {
            None => None,
            Some(entity) => Some(("custom_target", ScriptEntity::from(&entity))),
        };
        targets.affected_points = affected_points.into_iter().map(|p| (p.x, p.y)).collect();
        self.item_script(parent, kind, targets, arg, func)
    }

    pub fn entity_script<'a, T>(&'a self, parent: &Rc<RefCell<EntityState>>,
                                targets: ScriptEntitySet,
                                arg: Option<(&str, T)>,
                                func: &str) -> Result<()> where T: rlua::prelude::ToLua<'a> + Send {
        self.lua.globals().set("targets", targets)?;

        let script = match &parent.borrow().actor.actor.ai {
            None => {
                warn!("Script called for entity {} with no AI", parent.borrow().actor.actor.name);
                return Err(rlua::Error::ToLuaConversionError {
                    from: "Entity",
                    to: "Script",
                    message: Some(format!("No script found for entity")),
                });
            },
            Some(ai_template) => ai_template.script(),
        };

        let mut args_string = "(parent, targets".to_string();
        if let Some((arg_str, arg)) = arg {
            args_string.push_str(", ");
            args_string.push_str(arg_str);
            self.lua.globals().set(arg_str, arg)?;
        }
        args_string.push(')');
        self.execute_script(parent, &args_string, script.to_string(), func)
    }

    /// Runs a script on the given item, using the specified parent.  If `item_id` is None, then it
    /// is assumed that the item exists on the parent at the specified `item_index`.  If it Some,
    /// this is not assumed, but the specified index is still set on the item that is passed into
    /// the script state.
    pub fn item_script<'a, T>(&'a self, parent: &Rc<RefCell<EntityState>>,
                              kind: ScriptItemKind,
                              targets: ScriptEntitySet,
                              arg: Option<(&str, T)>,
                              func: &str) -> Result<()> where T: rlua::prelude::ToLua<'a> + Send {
        let script_item = ScriptItem::new(parent, kind)?;

        let item = script_item.try_item()?;
        let script = get_item_script(&item)?;
        self.lua.globals().set("item", script_item)?;
        self.lua.globals().set("targets", targets)?;

        let mut args_string = "(parent, item, targets".to_string();
        if let Some((arg_str, arg)) = arg {
            args_string.push_str(", ");
            args_string.push_str(arg_str);

            self.lua.globals().set(arg_str, arg)?;
        }
        args_string.push(')');
        self.execute_script(parent, &args_string, script.to_string(), func)
    }

    pub fn ability_on_activate(&self, parent: &Rc<RefCell<EntityState>>,
                                       ability: &Rc<Ability>) -> Result<()> {
        let t: Option<(&str, usize)> = None;
        self.ability_script(parent, ability,
                            ScriptEntitySet::new(parent, &Vec::new()), t, "on_activate")
    }

    pub fn ability_on_target_select(&self,
                                    parent: &Rc<RefCell<EntityState>>,
                                    ability: &Rc<Ability>,
                                    targets: Vec<Option<Rc<RefCell<EntityState>>>>,
                                    selected_point: Point,
                                    affected_points: Vec<Point>,
                                    func: &str,
                                    custom_target: Option<Rc<RefCell<EntityState>>>) -> Result<()> {
        let mut targets = ScriptEntitySet::new(parent, &targets);
        targets.selected_point = Some((selected_point.x, selected_point.y));
        let arg = match custom_target {
            None => None,
            Some(entity) => Some(("custom_target", ScriptEntity::from(&entity))),
        };
        targets.affected_points = affected_points.into_iter().map(|p| (p.x, p.y)).collect();
        self.ability_script(parent, ability, targets, arg, func)
    }

    pub fn ability_script<'a, T>(&'a self, parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>,
                                 targets: ScriptEntitySet, arg: Option<(&str, T)>,
                                 func: &str) -> Result<()> where T: rlua::prelude::ToLua<'a> + Send {
        let script = get_ability_script(ability)?;
        self.lua.globals().set("ability", ScriptAbility::from(ability))?;
        self.lua.globals().set("targets", targets)?;

        let mut args_string = "(parent, ability, targets".to_string();
        if let Some((arg_str, arg)) = arg {
            args_string.push_str(", ");
            args_string.push_str(arg_str);

            self.lua.globals().set(arg_str, arg)?;
        }
        args_string.push(')');
        self.execute_script(parent, &args_string, script.to_string(), func)
    }

    pub fn trigger_script(&self, script_id: &str, func: &str, parent: &Rc<RefCell<EntityState>>,
                          target: &Rc<RefCell<EntityState>>) -> Result<()> {
        let script = get_script_from_id(script_id)?;
        self.lua.globals().set("target", ScriptEntity::from(target))?;

        self.execute_script(parent, "(parent, target)", script, func)
    }
}

fn get_script_from_id(id: &str) -> Result<String> {
    match Module::script(id) {
        None => Err(rlua::Error::ToLuaConversionError {
            from: "&str",
            to: "Script",
            message: Some(format!("No script found with id '{}'", id).to_string()),
        }),
        Some(ref script) => Ok(script.to_string())
    }
}

fn get_item_script(item: &Rc<Item>) -> Result<&str> {
    match &item.usable {
        None => Err(rlua::Error::ToLuaConversionError {
            from: "ScriptItem",
            to: "Item",
            message: Some(format!("The item is not usable {}", item.id).to_string()),
        }),
        Some(usable) => Ok(&usable.script)
    }
}

fn get_ability_script(ability: &Rc<Ability>) -> Result<&str> {
    match ability.active {
        None => Err(rlua::Error::ToLuaConversionError {
            from: "Rc<Ability>",
            to: "ScriptAbility",
            message: Some("The Ability is not active".to_string()),
        }),
        Some(ref active) => Ok(&active.script),
    }
}

fn get_targeter() -> Result<Rc<RefCell<AreaTargeter>>> {
    let area_state = GameState::area_state();
    let mut area_state = area_state.borrow_mut();
    match area_state.targeter() {
        None => {
            warn!("Error getting targeter");
            Err(rlua::Error::ToLuaConversionError {
                from: "Lua",
                to: "Targeter",
                message: Some("No targeter is present".to_string()),
            })
        },
        Some(tg) => Ok(tg),
    }
}

/// The ScriptInterface, accessible in all Lua scripts as the global `game`.
/// The following methods are available on this object (documentation WIP):
///
/// # `entity_with_id(id: String) -> Table<ScriptEntity>`
/// Returns a `ScriptEntity` object for the entity with the given unique
/// id, if such an entity can be found.  Otherwise, returns the invalid `ScriptEntity`.  The ID is
/// the unique id associated with the individual entity, which is typically the same as the actor
/// ID for actors that are only used once, and is generated for encounter created actors.
/// ## Examples
/// ```lua
///   entity = game:entity_with_id("id1")
///   if not entity:is_valid() return end
///   game:log("Found entity with name " .. entity:name())
///```
///
/// # `entities_with_ids(ids: Table) -> ScriptEntity`
/// Returns a list of `ScriptEntity` objects for each of the specified
/// ids that are found.  The list may be empty.  Also see `entity_with_id(id)`.
/// ## Examples
/// ```lua
///  entities = game:entities_with_ids({"id1", "id2"})
///  for i = 1, #entities do
///    game:log("Found entity with name " .. entities[i]:name())
///  end
/// ```
///
///# `activate_targeter()`
/// Activates the current targeter, if one exists.  You should first
/// validate the targeter exists with `has_targeter()` and then
/// set a valid position with `check_targeter_position(x, y)`.
///
/// ## Examples
/// ```lua
/// if game:has_targeter() then
///   x = target:x()
///   y = target:y()
///   if game:check_targeter_position(x, y) then
///     game:activate_targeter()
///   end
/// end
/// ```
///
/// # `cancel_targeter()`
/// Deactivates the current targeter.  Throws an error if there is no
/// active targeter.  You should verify a targeter is active with `has_targeter()`
/// before calling this method.
///
/// # `check_targeter_position(x: Int, y: Int) -> bool`
/// Sets the selected coordinates for the targeter and then checks if
/// the position is valid to activate.  See also `activate_targeter()`
///
/// # `get_targeter_affected() -> ScriptEntitySet`
/// Returns the set of entities currently affected by the active targeter,
/// if there is one.  If there is no active targeter, throws an error.  You
/// should verify a targeter is active with `has_targeter()` before calling
/// this method.
///
/// # `has_targeter() -> Bool`
/// Returns true if a targeter is currently active, false otherwise.
/// Useful for AI activating of abilities.
///
/// # `cancel_blocking_anims()`
/// Cancels all current blocking animations on all entities.  Blocking animations
/// are those that normally cause the player to wait for their completion before
/// performing another action.  This includes movement, attacks, and most
/// fixed duration particle effects.
///
/// # `fade_out_in()`
/// Causes the main view to fade out, then back in again.  This duration of the
/// fades is defined in the theme for the `WindowFade` widget.
///
/// # `init_party_day()`
/// Starts a new day for the player character and party.  This resets all skill
/// uses and sets maximum hit points.  This is normally used in a script when the
/// party rests.
///
/// # `show_confirm(message: String, accept: String, cancel: String,
/// id: String, func: String)`
/// Shows a simple confirmation dialog with the specified `message`, and specified text
/// on the `accept` and `cancel` buttons.  If the user cancels, no action is taken.  If the
/// user accepts, the specified `func` is called from the script with `id`.
///
/// # `log(message: String)`
/// Logs the specified string to the game's output at info level.  This is primarily useful
/// for debugging purposes.
///
/// # `ap_display_factor() -> Int`
/// Gets the ap display factor, which is the factor that the internal AP representation is
/// divided by when displayed.  Any AP values that are displayed to the user must be
/// divided by this factor.
///
/// # `anim_base_time() -> Float`
/// Returns the animation base time, which affects how long animations last.  Generally,
/// animations should multiply some base time by this factor when determining the duration
/// of animations.  This value is user configurable in the options menu.
///
/// # `atan2(x: Float, y: Float) -> Float`
/// Computes the four quadrant arctan function.  See `f32::atan2`
///
/// # `run_script_delayed(script_id: String, func: String, delay: Float)`
/// Causes the specified `func` from the script with `script_id` to be run after `delay`
/// seconds.  The script is actually run on the first frame after `delay` seconds have
/// elapsed.  The game can normally achieve a comfortable 60 fps on the vast majority of
/// hardware, but be aware that this is not always the case.
///
/// # `create_callback(parent: ScriptEntity, script: String) -> ScriptCallback`
/// Creates a new script callback.  This callback will utilize the specified script
/// file for all methods.  See `ScriptCallback` for more.
///
/// # `set_quest_state(quest: String, state: String)`
/// Sets the specified `quest` to the `state`.  `state` must be one of `Hidden`, `Visible`,
/// `Active`, or `Complete`.  `quest` must be the ID of a valid quest definition.
///
/// # `set_quest_entry_state(quest: String, entry: String, state: String)`
/// Sets the specified `entry` within the specified `quest` to `state`.  `state` must be one
/// of `Hidden`, `Visible, `Active`, or `Complete`.  `quest` must be the ID of a valid quest
/// definition, and `entry` must be an entry within that quest.
///
/// # `get_quest_state(quest: String) -> String`
/// Returns the current `state` of the specified `quest`.  `state` will be one of
/// `Hidden`, `Visible`, `Active`, or `Complete`.
///
/// # `get_quest_entry_state(quest: String, entry: String)`
/// Returns the current `state` of the specified `entry` in the given `quest`.
///
/// # `set_world_map_location_visible(location: String, visible: Bool)`
/// Sets the specified `location` in the world map to the specified `visible`.  The
/// location must be defined in the world_map section of the campaign definition file.
///
/// # `set_world_map_location_enabled(location: String, enabled: Bool)`
/// Sets the specified `location` in the world map `enabled`.  If disabled, a user
/// viewing the world map cannot travel to that location.  The location  must be defined
/// in the world_map section of the campaign definition file.
///
/// # `is_passable(entity: ScriptEntity, x: Int, y: Int) -> Bool`
/// Returns true if the specified coordinates in the current area are passable for
/// the entity, false otherwise.
///
/// # `spawn_actor_at(id: String, x: Int, y: Int) -> ScriptEntity`
/// Attempts the spawn an instance of the actor with the specified `id` at the
/// coordinates `x`, `y` in the current area.  If successful, returns the
/// ScriptEntity that was just spawned.  If not, returns the invalid ScriptEntity
///
/// # `spawn_encounter_at(x: Int, y: Int)`
/// Causes the encounter in the current area at `x`, `y` to spawn entities based
/// on its encounter definition.  If the entities are hostile and within player
/// visibility, will initiate combat.
///
/// # `enable_trigger_at(x: Int, y: Int)`
/// Sets the trigger in the current area at `x`, `y` to enabled.  This means the
/// trigger will fire when its condition (such as player entering its coordinates)
/// are met.  This method will only have an effect on triggers which are set to
/// be initially_disabled in their defintion, or which have been disabled via
/// `disable_trigger_at`.
///
/// # `disable_trigger_at(x: Int, y: Int)`
/// Sets the trigger in the current area at `x`, `y` to disabled.  This means the
/// trigger will not fire regardless of whether its condition is met.
///
/// # `enable_prop_at(x: Int, y: Int)`
/// Sets the prop in the current area at `x`, `y` to enabled.  When enabled, props
/// can be interacted with if relevant for the given prop (doors or containers).
///
/// # `disable_prop_at(x: Int, y: Int)`
/// Sets the prop in the current area at `x`, `y` to disabled.  When disabled, props
/// cannot be interacted with regardless of whether they otherwise are interactive.
///
/// # `toggle_prop_at(x: Int, y: Int)`
/// Toggles the enabled / disabled state of the prop at `x`, `y`.  See `enable_prop_at` and
/// `disable_prop_at`
///
/// # `say_line(line: String, target: ScriptEntity (Optional))`
/// The specified `target`, or the player if no target is specified, will say the line
/// of text specified by `line`.  This is represented by the text appearing on the main
/// area view overhead of the target entity.  The text fades away after several seconds.
///
/// # `start_conversation(id: String, target: ScriptEntity (Optional))`
/// Starts the conversation with the specified `id`, with the `target` or the player if the
/// target is not specified.  The conversation is defined in the conversation data file
/// for the relevant id.
///
/// # `show_game_over_window(text: String)`
/// Shows the game over window, indicating that the player cannot continue
/// in the current module without loading.  This can be used to show victory
/// or defeat.  The specified `text` is displayed.
///
/// # `load_module(id: String)`
/// Causes the module with the specified `id` to be loaded.  The player, their money,
/// items, and party stash are transfered to the module intact.  Player script state is
/// not currently transfered, and any other data in the current module is also not
/// transfered, including the player's party.  This action is performed asynchronously
/// on the next UI update, the remained of the current script will be executed.
///
/// # `player -> ScriptEntity`
/// Returns a reference to the player character ScriptEntity.
///
/// # `show_cutscene(id: String)`
/// Causes the cutscene with the specified `id` to show.  This blocks the user interface
/// until the cutscene is complete or the player skips it.  The cutscene is launched
/// asynchronously on the next frame, so the remaineder of this script script will execute
/// immediately.
///
/// # `scroll_view(x: Int, y: Int)`
/// Causes the view of the current area to scroll to the specified `x`, `y` coordinates.
/// This done using a smooth scroll effect.  The scroll begins on the next frame, so the
/// remainder of the current script will continue to execute immediately.
///
/// # `num_effects_with_tag(tag: String) -> Int`
/// Returns the number of currently active effects, in any area, with the specified effect
/// tag.  This can be used in scripts to enforce a global limit on a specific effect type.
///
/// # `has_party_member(id: String) -> Bool`
/// Returns true if one of the current party members has the specified `id`, false otherwise
///
/// # `add_party_member(id: String)`
/// Searches for an entity with the specified `id`.  If it is found, adds that entity as a
/// member of the player's party, making them controllable by the player.  If an entity with
/// the `id` is not found, throws an error.
///
/// # `party_coins() -> Int`
/// Returns the current amount of party coins.  Note that this value must be divided by the
/// item_value_display_factor in the module rules in order to get the displayed amount of
/// coins.
///
/// # `add_party_coins(amount: Int)`
/// Adds the specified number of coins to the party.  Note that this value is divided by
/// the item_value_display_factor to get the displayed coinage.
///
/// # `add_party_item(id: String, adjective: String (Optional))`
/// Creates an item with the specified `id`, and `adjective`, if specified.  If there is
/// no item definition with this ID or the adjective is specified but there is no
/// adjective with that ID, throws an error.  Otherwise, the item is added to the party
/// stash.
///
/// # `add_party_xp(amount: Int)`
/// Adds the specified amount of XP to the party.  Each current party member is given
/// this amount of XP.
///
/// # `transition_party_to(area: String, x: Int, y: Int)`
/// Moves the party to the specified coordinates within the specified area.  If the area
/// or coordinates are invalid, this will currently leave the game in a bad state where
/// the player is forced to load to continue.  The player is moved to the exact coordinates,
/// whereas other party members are moved to nearby coordinates.
///
pub struct ScriptInterface { }

impl UserData for ScriptInterface {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("has_targeter", |_, _, ()| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            Ok(area_state.targeter().is_some())
        });

        methods.add_method("check_targeter_position", |_, _, (x, y): (i32, i32)| {
            let targeter = get_targeter()?;
            let mut targeter = targeter.borrow_mut();
            targeter.on_mouse_move(x, y);
            Ok(targeter.is_valid_to_activate())
        });

        methods.add_method("get_targeter_affected", |_, _, ()| {
            let targeter = get_targeter()?;
            let targeter = targeter.borrow_mut();
            let parent = targeter.parent();
            let affected = targeter.cur_affected().iter().map(|e| Some(Rc::clone(e))).collect();
            Ok(ScriptEntitySet::new(&parent, &affected))
        });

        methods.add_method("cancel_targeter", |_, _, ()| {
            let targeter = get_targeter()?;
            targeter.borrow_mut().on_cancel();
            Ok(())
        });

        methods.add_method("activate_targeter", |_, _, ()| {
            let targeter = get_targeter()?;
            targeter.borrow_mut().on_activate();
            Ok(())
        });

        methods.add_method("cancel_blocking_anims", |_, _, ()| {
            GameState::remove_all_blocking_animations();
            Ok(())
        });

        methods.add_method("fade_out_in", |_, _, ()| {
            let pc = GameState::player();
            let cb = OnTrigger::FadeOutIn;
            GameState::add_ui_callback(vec![cb], &pc, &pc);
            Ok(())
        });

        methods.add_method("init_party_day", |_, _, ()| {
            for member in GameState::party() {
                member.borrow_mut().actor.init_day();
            }
            Ok(())
        });

        methods.add_method("show_confirm", |_, _, (msg, accept, cancel, id, func):
                           (String, String, String, String, String)| {
            let pc = GameState::player();
            let data = on_trigger::DialogData {
                message: msg,
                accept_text: accept,
                cancel_text: cancel,
                on_accept: Some(on_trigger::ScriptData {
                    id,
                    func
                }),
            };
            let cb = OnTrigger::ShowConfirm(data);
            GameState::add_ui_callback(vec![cb], &pc, &pc);
            Ok(())
        });

        methods.add_method("log", |_, _, val: String| {
            info!("[LUA]: {}", val);
            Ok(())
        });

        methods.add_method("ap_display_factor", |_, _, ()| {
            let rules = Module::rules();
            Ok(rules.display_ap)
        });

        methods.add_method("anim_base_time", |_, _, ()| {
            let secs = Config::animation_base_time_millis() as f32 / 1000.0;
            Ok(secs)
        });

        methods.add_method("atan2", |_, _, (x, y): (f32, f32)| {
            Ok(y.atan2(x))
        });

        methods.add_method("run_script_delayed", |_, _, (script, func, delay): (String, String, f32)| {
            let player = GameState::player();
            let parent = player.borrow().index();
            let mut cb_data = CallbackData::new_trigger(parent, script);
            cb_data.add_func(FuncKind::OnAnimComplete, func)?;

            let mut anim = Anim::new_non_blocking_wait(&player, (delay * 1000.0) as u32);
            anim.add_completion_callback(Box::new(cb_data));

            GameState::add_animation(anim);
            Ok(())
        });

        methods.add_method("create_callback", |_, _, (parent, script):
                           (ScriptEntity, String)| {
            let index = parent.try_unwrap_index()?;
            let cb_data = CallbackData::new_trigger(index, script);
            Ok(cb_data)
        });

        methods.add_method("set_quest_state", |_, _, (quest, state): (String, String)| {
            let state = quest_state::EntryState::from_str(&state);
            if let None = Module::quest(&quest) {
                warn!("Set quest state for invalid quest '{}'", quest);
            }
            GameState::set_quest_state(quest, state);
            Ok(())
        });

        methods.add_method("set_quest_entry_state", |_, _, (quest, entry, state): (String, String, String)| {
            let state = quest_state::EntryState::from_str(&state);
            match Module::quest(&quest) {
                None => warn!("Set quest entry state for invalid quest '{}'", quest),
                Some(ref quest) => {
                    if !quest.entries.contains_key(&entry) {
                        warn!("Set quest entry state for invalid entry '{}' in '{:?}'", entry, quest);
                    }
                }
            }

            GameState::set_quest_entry_state(quest, entry, state);
            Ok(())
        });

        methods.add_method("get_quest_state", |_, _, quest: String| {
            if let None = Module::quest(&quest) {
                warn!("Requested state for invalid quest '{}'", quest);
            }
            Ok(format!("{:?}", GameState::get_quest_state(quest)))
        });

        methods.add_method("get_quest_entry_state", |_, _, (quest, entry): (String, String)| {
            match Module::quest(&quest) {
                None => warn!("Requested entry state for invalid quest '{}'", quest),
                Some(ref quest) => {
                    if !quest.entries.contains_key(&entry) {
                        warn!("Requested entry state for invalid entry '{}' in '{:?}'", entry, quest);
                    }
                }
            }
            Ok(format!("{:?}", GameState::get_quest_entry_state(quest, entry)))
        });

        methods.add_method("set_world_map_location_visible",
                           |_, _, (location, vis): (String, bool)| {
            GameState::set_world_map_location_visible(&location, vis);
            Ok(())
        });

        methods.add_method("set_world_map_location_enabled",
                           |_, _, (location, en): (String, bool)| {
            GameState::set_world_map_location_enabled(&location, en);
            Ok(())
        });

        methods.add_method("is_passable", |_, _, (entity, x, y): (ScriptEntity, i32, i32)| {
            let area_state = GameState::area_state();
            let area_state = area_state.borrow();
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            let entities_to_ignore = vec![entity.index()];
            Ok(area_state.is_passable(&entity, &entities_to_ignore, x, y))
        }
        );

        methods.add_method("spawn_actor_at", |_, _, (id, x, y): (String, i32, i32)| {
            let actor = match Module::actor(&id) {
                None => {
                    warn!("Unable to spawn actor '{}': not found", id);
                    return Ok(ScriptEntity::invalid());
                }, Some(actor) => actor,
            };

            let area_state = GameState::area_state();
            let location = Location::new(x, y, &area_state.borrow().area);
            let result = match area_state.borrow_mut()
                .add_actor(actor, location, None, false, None) {
                Ok(index) => Ok(ScriptEntity::new(index)),
                Err(e) => {
                    warn!("Error spawning actor in area: {}", e);
                    Ok(ScriptEntity::invalid())
                }
            };

            let mgr = GameState::turn_manager();
            mgr.borrow_mut().check_ai_activation_for_party(&mut area_state.borrow_mut());

            result
        });

        methods.add_method("spawn_encounter_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();

            if !area_state.spawn_encounter_at(x, y) {
                warn!("Unable to find encounter for script spawn at {},{}", x, y);
            }

            let mgr = GameState::turn_manager();
            mgr.borrow_mut().check_ai_activation_for_party(&mut area_state);

            Ok(())
        });

        methods.add_method("enable_trigger_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            if !area_state.set_trigger_enabled_at(x, y, true) {
                warn!("Unable to find trigger at {},{}", x, y);
            }
            Ok(())
        });

        methods.add_method("disable_trigger_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            if !area_state.set_trigger_enabled_at(x, y, false) {
                warn!("Unable to find trigger at {},{}", x, y);
            }
            Ok(())
        });

        methods.add_method("enable_prop_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            if !area_state.set_prop_enabled_at(x, y, true) {
                warn!("Unable to find prop at {},{}", x, y);
            }
            Ok(())
        });

        methods.add_method("disable_prop_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            if !area_state.set_prop_enabled_at(x, y, false) {
                warn!("Unable to find prop at {},{}", x, y);
            }
            Ok(())
        });

        methods.add_method("toggle_prop_at", |_, _, (x, y): (i32, i32)| {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            let index = match area_state.prop_index_at(x, y) {
                None => {
                    warn!("Unable to find prop at {},{}", x, y);
                    return Ok(());
                }, Some(prop) => prop,
            };
            area_state.toggle_prop_active(index);

            Ok(())
        });

        methods.add_method("say_line", |_, _, (line, target): (String, Option<ScriptEntity>)| {
            let pc = GameState::player();
            let target = match target {
                None => Rc::clone(&pc),
                Some(ref entity) => {
                    entity.try_unwrap()?
                }
            };

            let cb = OnTrigger::SayLine(line);
            GameState::add_ui_callback(vec![cb], &pc, &target);
            Ok(())
        });

        methods.add_method("start_conversation", |_, _, (id, target): (String, Option<ScriptEntity>)| {
            let pc = GameState::player();
            let target = match target {
                None => Rc::clone(&pc),
                Some(ref entity) => {
                    entity.try_unwrap()?
                }
            };

            let cb = OnTrigger::StartConversation(id);
            GameState::add_ui_callback(vec![cb], &pc, &target);
            Ok(())
        });

        methods.add_method("show_game_over_window", |_, _, text: String| {
            let pc = GameState::player();
            let cb = OnTrigger::GameOverWindow(text);
            GameState::add_ui_callback(vec![cb], &pc, &pc);
            Ok(())
        });

        methods.add_method("load_module", |_, _, id: String| {
            let pc = GameState::player();
            let cb = OnTrigger::LoadModule(id);
            GameState::add_ui_callback(vec![cb], &pc, &pc);
            Ok(())
        });

        methods.add_method("player", |_, _, ()| {
            Ok(ScriptEntity::from(&GameState::player()))
        });

        methods.add_method("show_cutscene", |_, _, id: String| {
            let pc = GameState::player();
            let cb = OnTrigger::ShowCutscene(id);
            GameState::add_ui_callback(vec![cb], &pc, &pc);
            Ok(())
        });

        methods.add_method("scroll_view", |_, _, (x, y): (i32, i32)| {
            let pc = GameState::player();
            let cb = OnTrigger::ScrollView(x, y);
            GameState::add_ui_callback(vec![cb], &pc, &pc);
            Ok(())
        });

        methods.add_method("num_effects_with_tag", |_, _, tag: String| {
            let mgr = GameState::turn_manager();
            let mgr = mgr.borrow();

            let mut count = 0;
            for effect in mgr.effect_iter() {
                if effect.tag == tag {
                    count += 1;
                }
            }
            Ok(count)
        });

        methods.add_method("entities_with_ids", |_, _, ids: Vec<String>| {
            Ok(entities_with_ids(ids))
        });

        methods.add_method("entity_with_id", |_, _, id: String| {
            match entity_with_id(id) {
                Some(entity) => Ok(ScriptEntity::from(&entity)),
                None => Ok(ScriptEntity::invalid()),
            }
        });

        methods.add_method("has_party_member", |_, _, id: String| {
            Ok(GameState::has_party_member(&id))
        });

        methods.add_method("add_party_member", |_, _, id: String| {
            let entity = match entity_with_id(id) {
                Some(entity) => entity,
                None => return Err(rlua::Error::ToLuaConversionError {
                    from: "ID",
                    to: "ScriptENtity",
                    message: Some("Entity with specified id does not exist".to_string())
                })
            };

            GameState::add_party_member(entity);

            Ok(())
        });

        methods.add_method("party_coins", |_, _, ()| {
            let coins = GameState::party_coins();
            Ok(coins)
        });

        methods.add_method("add_party_coins", |_, _, amount: i32| {
            GameState::add_party_coins(amount);
            let stash = GameState::party_stash();
            let stash = &stash.borrow();
            stash.listeners.notify(stash);

            let factor = Module::rules().item_value_display_factor;
            let pc = GameState::player();
            let line = if amount >= 0 {
                format!("Gained {} coins", (amount as f32 / factor) as i32)
            } else {
                format!("Lost {} coins", (-amount as f32 / factor) as i32)
            };

            let cb = OnTrigger::SayLine(line);
            GameState::add_ui_callback(vec![cb], &pc, &pc);

            Ok(())
        });

        methods.add_method("add_party_item", |_, _, (item, adj): (String, Option<String>)| {
            let adjectives: Vec<_> = adj.into_iter().collect();

            let stash = GameState::party_stash();
            let item = match Module::create_get_item(&item, &adjectives) {
                None => return Err(rlua::Error::FromLuaConversionError {
                    from: "String",
                    to: "Item",
                    message: Some(format!("Item '{}' does not exist", item)),
                }),
                Some(item) => item,
            };

            let item_state = ItemState::new(item);
            stash.borrow_mut().add_item(1, item_state);
            Ok(())
        });

        methods.add_method("add_party_xp", |_, _, amount: u32| {
            for member in GameState::party().iter() {
                member.borrow_mut().add_xp(amount);
            }

            let pc = GameState::player();
            let line = format!("Gained {} xp", amount);
            let cb = OnTrigger::SayLine(line);
            GameState::add_ui_callback(vec![cb], &pc, &pc);
            Ok(())
        });

        methods.add_method("transition_party_to", |_, _, (area, x, y): (String, i32, i32)| {
            let location = Some(area);
            GameState::transition(&location, x, y);
            Ok(())
        });
    }
}

#[derive(Clone, Debug)]
pub enum ScriptItemKind {
    Stash(usize),
    Quick(QuickSlot),
    WithID(String),
}

impl ScriptItemKind {
    pub fn item_checked(&self, parent: &Rc<RefCell<EntityState>>) -> Option<ItemState> {
        match self {
            ScriptItemKind::Stash(index) => {
                let stash = GameState::party_stash();
                let stash = stash.borrow();
                match stash.items().get(*index) {
                    None => None,
                    Some(&(_, ref item)) => Some(item.clone()),
                }
            },
            ScriptItemKind::Quick(slot) => {
                match parent.borrow().actor.inventory().quick(*slot) {
                    None => None,
                    Some(item) => Some(item.clone()),
                }
            },
            ScriptItemKind::WithID(id) => {
                match Module::item(id) {
                    None => None,
                    Some(item) => Some(ItemState::new(item)),
                }
            }
        }
    }

    pub fn item(&self, parent: &Rc<RefCell<EntityState>>) -> ItemState {
        match self {
            ScriptItemKind::Stash(index) => {
                let stash = GameState::party_stash();
                let stash = stash.borrow();
                match stash.items().get(*index) {
                    None => unreachable!(),
                    Some(&(_, ref item)) => item.clone(),
                }
            },
            ScriptItemKind::Quick(slot) => {
                match parent.borrow().actor.inventory().quick(*slot) {
                    None => unreachable!(),
                    Some(item) => item.clone(),
                }
            },
            ScriptItemKind::WithID(id) => {
                match Module::item(id) {
                    None => unreachable!(),
                    Some(item) => ItemState::new(item),
                }
            }
        }
    }
}

/// A ScriptItem, representing a specific item in a player or creature inventory,
/// quick slot, or the party stash, depending on the `ScriptItemKind`.
/// This is passed as the `item` field when using usable items with an associated
/// script.
///
/// # `activate(target: ScriptEntity)`
/// Activates this usable item.  This will remove the AP associated with using this
/// item from the specified `target`.  If the item is consumable, the item will be
/// consumed on calling this method.
///
/// This method is generally used when called from the `on_activate` script of a
/// usable item, once the script has determined that the item should definitely be
/// used.
///
/// # `name() -> String`
/// Returns the name of this Item.
///
/// # `duration() -> Int`
/// Returns the duration, in rounds, of this item, as defined in the item's resource
/// definition.  How this value is used (or not) is up to the script to define.
///
/// # `create_callback(parent: ScriptEntity)`
/// Creates a `ScriptCallback` with the specified parent for this item.  Methods
/// can then be added to the ScriptCallback to cause it to be called when certain
/// events happen.  These methods will be called from this item's script, as
/// defined in its resource file.
#[derive(Clone)]
pub struct ScriptItem {
    parent: usize,
    kind: ScriptItemKind,
    id: String,
    name: String,
    ap: u32,
}

impl ScriptItem {
    pub fn new(parent: &Rc<RefCell<EntityState>>, kind: ScriptItemKind) -> Result<ScriptItem> {
        let item = match kind.item_checked(parent) {
            None => return Err(rlua::Error::FromLuaConversionError {
                from: "ScriptItem",
                to: "Item",
                message: Some(format!("Item with kind {:?} does not exist", kind)),
            }),
            Some(item) => item,
        };

        let ap = match &item.item.usable {
            None => 0,
            Some(usable) => usable.ap,
        };

        Ok(ScriptItem {
            parent: parent.borrow().index(),
            kind,
            id: item.item.id.to_string(),
            name: item.item.name.to_string(),
            ap,
        })
    }

    fn try_item(&self) -> Result<Rc<Item>> {
        let parent = ScriptEntity::new(self.parent).try_unwrap()?;
        let item = self.kind.item_checked(&parent);

        match item {
            None => Err(rlua::Error::FromLuaConversionError {
                from: "ScriptItem",
                to: "Item",
                message: Some(format!("The item '{}' no longer exists in the parent", self.id)),
            }),
            Some(item_state) => Ok(Rc::clone(&item_state.item)),
        }
    }
}

impl UserData for ScriptItem {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("activate", &activate_item);
        methods.add_method("name", |_, item, ()| {
            Ok(item.name.to_string())
        });
        methods.add_method("duration", |_, item, ()| {
            let item = item.try_item()?;
            match &item.usable {
                None => Ok(0),
                Some(usable) => match usable.duration {
                    ability::Duration::Rounds(amount) => Ok(amount),
                    _ => Ok(0),
                },
            }
        });
        methods.add_method("create_callback", |_, item, parent: ScriptEntity| {
            let index = parent.try_unwrap_index()?;
            let cb_data = CallbackData::new_item(index, item.id.to_string());
            Ok(cb_data)
        });
    }
}

fn entities_with_ids(ids: Vec<String>) -> Vec<ScriptEntity> {
    let mut result = Vec::new();

    let area_state = GameState::area_state();
    let area_id = area_state.borrow().area.id.to_string();

    let mgr = GameState::turn_manager();
    for entity in mgr.borrow().entity_iter() {
        {
            let entity = entity.borrow();
            if !entity.location.is_in_area_id(&area_id) { continue; }
            if !ids.iter().any(|uid| uid == entity.unique_id()) { continue; }
        }

        result.push(ScriptEntity::from(&entity));
    }

    result
}

pub fn entity_with_id(id: String) -> Option<Rc<RefCell<EntityState>>> {
    let mgr = GameState::turn_manager();
    for entity in mgr.borrow().entity_iter() {
        {
            let entity = entity.borrow();
            if entity.unique_id() != &id { continue; }
        }

        return Some(entity)
    }

    None
}

fn activate_item(_lua: &Lua, script_item: &ScriptItem, target: ScriptEntity) -> Result<()> {
    let item = script_item.try_item()?;
    let target = target.try_unwrap()?;

    let mgr = GameState::turn_manager();
    if mgr.borrow().is_combat_active() {
        target.borrow_mut().actor.remove_ap(script_item.ap);
    }

    let area = GameState::area_state();
    let name = item.name.to_string();
    area.borrow_mut().add_feedback_text(name, &target, ColorKind::Info);

    match item.usable {
        None => unreachable!(),
        Some(ref usable) => {
            if usable.consumable {
                let parent = ScriptEntity::new(script_item.parent).try_unwrap()?;
                match &script_item.kind {
                    ScriptItemKind::Quick(slot) => {
                        let item = parent.borrow_mut().actor.clear_quick(*slot);
                        add_another_to_quickbar(&parent, item, *slot);
                    }, ScriptItemKind::Stash(index) => {
                        // throw away item
                        let stash = GameState::party_stash();
                        let _ = stash.borrow_mut().remove_item(*index);
                    }, ScriptItemKind::WithID(_) => (),
                };
            }
        }
    }

    Ok(())
}

fn add_another_to_quickbar(parent: &Rc<RefCell<EntityState>>,
                           item: Option<ItemState>, slot: QuickSlot) {
    let item = match item {
        None => return,
        Some(item) => item,
    };

    let stash = GameState::party_stash();
    let index = match stash.borrow().items().find_index(&item) {
        None => return,
        Some(index) => index,
    };

    let mut stash = stash.borrow_mut();
    if let Some(item) = stash.remove_item(index) {
        // we know the quick slot is empty because it was just cleared
        let _ = parent.borrow_mut().actor.set_quick(item, slot);
    }
}
