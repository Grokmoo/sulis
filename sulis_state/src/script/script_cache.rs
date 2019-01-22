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

use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{Cell, RefCell};

use rlua::{self, ToLuaMulti, FromLuaMulti, ToLua};

use sulis_core::util::Point;
use sulis_module::{Ability, Item, Module};
use crate::{ai, EntityState};
use crate::script::{Result, ScriptState, ScriptEntity, ScriptEntitySet,
    ScriptItem, ScriptAbility, ScriptItemKind};

thread_local! {
    static SCRIPT_CACHE: RefCell<HashMap<String, Rc<ScriptState>>> = RefCell::new(HashMap::new());
    static REPORTING: Cell<bool> = Cell::new(true);
}

pub fn setup() -> Result<()> {
    let start = Instant::now();
    SCRIPT_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();

        cache.clear();
        for id in Module::all_scripts() {
            let script = get_script_from_id(&id)?;
            let mut state = ScriptState::new();
            state.load(&id, &script)?;
            cache.insert(id, Rc::new(state));
        }
        Ok(())
    })?;

    info!("Setup scripts in {:.3} millis", get_elapsed_millis(start.elapsed()));
    Ok(())
}

pub fn set_report_enabled(enabled: bool) {
    REPORTING.with(|r| r.set(enabled));
}

pub fn exec_func<Args, Ret>(id: &str, func: &str, args: Args) -> Result<Ret>
    where Args: for<'a> ToLuaMulti<'a>, Ret: for<'a> FromLuaMulti<'a> {

    let state = SCRIPT_CACHE.with(|cache| {
        let cache = cache.borrow();

        //setup the script if it does not already exist
        if !cache.contains_key(id) {
            return Err(rlua::Error::ToLuaConversionError {
                from: "String",
                to: "Script",
                message: Some(format!("Script '{}' does not exist", id)),
            });
        }

        Ok(Rc::clone(&cache.get(id).unwrap()))
    })?;

    let reporting = REPORTING.with(|r| r.get());

    state.exec_func(func, args, reporting)
}

pub fn ai_script(parent: &Rc<RefCell<EntityState>>, func: &str) -> Result<ai::State> {
    let script = get_script_id_from_entity(parent)?;
    let parent = ScriptEntity::from(parent);
    exec_func(&script, func, parent)
}

pub fn entity_script<T>(parent: &Rc<RefCell<EntityState>>, targets: ScriptEntitySet,
                        arg: Option<T>, func: &str) -> Result<()>
where T: for<'a> ToLua<'a> + Send {

    let script = get_script_id_from_entity(parent)?;
    let parent = ScriptEntity::from(parent);
    exec_func(&script, func, (parent, targets, arg))
}

pub fn item_on_activate(parent: &Rc<RefCell<EntityState>>,
                        kind: ScriptItemKind) -> Result<()> {
    let t: Option<usize> = None;
    item_script(parent, kind, ScriptEntitySet::new(parent, &Vec::new()),
    t, "on_activate")
}

pub fn item_on_target_select(parent: &Rc<RefCell<EntityState>>, kind: ScriptItemKind,
                             targets: Vec<Option<Rc<RefCell<EntityState>>>>,
                             selected_point: Point, affected_points: Vec<Point>, func: &str,
                             custom_target: Option<Rc<RefCell<EntityState>>>) -> Result<()> {
    let mut targets = ScriptEntitySet::new(parent, &targets);
    targets.selected_point = Some((selected_point.x, selected_point.y));
    let arg = match custom_target {
        None => None,
        Some(entity) => Some(ScriptEntity::from(&entity)),
    };
    targets.affected_points = affected_points.into_iter().map(|p| (p.x, p.y)).collect();
    item_script(parent, kind, targets, arg, func)
}

/// Runs a script on the given item, using the specified parent.  If `item_id` is None, then it
/// is assumed that the item exists on the parent at the specified `item_index`.  If it Some,
/// this is not assumed, but the specified index is still set on the item that is passed into
/// the script state.
pub fn item_script<T>(parent: &Rc<RefCell<EntityState>>, kind: ScriptItemKind,
                      targets: ScriptEntitySet, arg: Option<T>, func: &str) -> Result<()>
where T: for<'a> ToLua<'a> + Send {

    let item = ScriptItem::new(parent, kind)?;
    let item_src = item.try_item()?;
    let script = get_item_script_id(&item_src)?;
    let parent = ScriptEntity::from(parent);

    exec_func(&script, func, (parent, item, targets, arg))
}

pub fn ability_on_activate(parent: &Rc<RefCell<EntityState>>,
                           ability: &Rc<Ability>) -> Result<()> {
    let t: Option<usize> = None;
    ability_script(parent, ability, ScriptEntitySet::new(parent, &Vec::new()),
    t, "on_activate")
}

pub fn ability_on_target_select(parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>,
                                targets: Vec<Option<Rc<RefCell<EntityState>>>>,
                                selected_point: Point, affected_points: Vec<Point>, func: &str,
                                custom_target: Option<Rc<RefCell<EntityState>>>) -> Result<()> {
    let mut targets = ScriptEntitySet::new(parent, &targets);
    targets.selected_point = Some((selected_point.x, selected_point.y));
    let arg = match custom_target {
        None => None,
        Some(entity) => Some(ScriptEntity::from(&entity)),
    };
    targets.affected_points = affected_points.into_iter().map(|p| (p.x, p.y)).collect();
    ability_script(parent, ability, targets, arg, func)
}

pub fn ability_script<T>(parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>,
                         targets: ScriptEntitySet, arg: Option<T>,
                         func: &str) -> Result<()> where T: for<'a> ToLua<'a> + Send {

    let script = get_ability_script_id(ability)?;
    let parent = ScriptEntity::from(parent);
    let ability = ScriptAbility::from(ability);
    exec_func(&script, func, (parent, ability, targets, arg))
}

pub fn trigger_script<Args>(script_id: &str, func: &str, args: Args) -> Result<()>
    where Args: for<'a> ToLuaMulti<'a> {

    exec_func(script_id, func, args)
}

fn get_script_id_from_entity(entity: &Rc<RefCell<EntityState>>) -> Result<String> {
    let entity = entity.borrow();
    let id = entity.unique_id();
    match &entity.actor.actor.ai {
        None => {
            Err(rlua::Error::ToLuaConversionError {
                from: "Entity",
                to: "Script",
                message: Some(format!("Script called for entity '{}' with no AI", id)),
            })
        }, Some(ai) => Ok(ai.script.to_string()),
    }
}

fn get_item_script_id(item: &Rc<Item>) -> Result<String> {
    match &item.usable {
        None => Err(rlua::Error::ToLuaConversionError {
            from: "ScriptItem",
            to: "Item",
            message: Some(format!("The item is not usable {}", item.id).to_string()),
        }),
        Some(usable) => Ok(usable.script.to_string()),
    }
}

fn get_ability_script_id(ability: &Rc<Ability>) -> Result<String> {
    match &ability.active {
        None => Err(rlua::Error::ToLuaConversionError {
            from: "Rc<Ability>",
            to: "ScriptAbility",
            message: Some("The Ability is not active".to_string()),
        }),
        Some(active) => Ok(active.script.to_string()),
    }
}

fn get_script_from_id(id: &str) -> Result<String> {
    match Module::script(id) {
        None => Err(rlua::Error::ToLuaConversionError {
            from: "&str",
            to: "Script",
            message: Some(format!("No script found with id '{}'", id)),
        }),
        Some(script) => Ok(script)
    }
}

fn get_elapsed_millis(elapsed: Duration) -> f64 {
    (elapsed.as_secs() as f64) * 1000.0 +
        (elapsed.subsec_nanos() as f64) / 1_000_000.0
}
