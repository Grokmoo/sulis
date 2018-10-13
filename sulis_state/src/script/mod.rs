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
    animation::Anim};

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
/// # `entity_with_id(id) -> Table<ScriptEntity>`
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
/// # `entities_with_ids(ids) -> ScriptEntity`
/// Returns a list of `ScriptEntity` objects for each of the specified
/// ids that are found.  The list may be empty.  Also see `entity_with_id(id)`.
/// ## Examples
/// ```lua
///  entities = game:entities_with_ids({"id1", "id2"})
///  for i = 1, #entities do
///    game:log("Found entity with name " .. entities[i]:name())
///  end
///
/// ```
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

fn entity_with_id(id: String) -> Option<Rc<RefCell<EntityState>>> {
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
