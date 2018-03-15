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

use std;
use std::rc::Rc;
use std::cell::RefCell;

use rlua::{Lua, UserData, UserDataMethods};

use sulis_module::Faction;
use {EntityState, GameState};
use script::{Result, ScriptEffect};

#[derive(Clone)]
pub struct ScriptEntity {
    pub index: usize,
}

impl ScriptEntity {
    pub fn new(index: usize) -> ScriptEntity {
        ScriptEntity { index }
    }

    pub fn from(entity: &Rc<RefCell<EntityState>>) -> ScriptEntity {
        ScriptEntity { index: entity.borrow().index }
    }
}

impl UserData for ScriptEntity {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("targets", &targets);

        methods.add_method("to_string", |_, entity, ()| {
            Ok(entity.index.to_string())
        });

        methods.add_method("create_effect", |_, entity, duration: u32| {
            Ok(ScriptEffect::new(entity.index, duration))
        });
    }
}

#[derive(Clone)]
pub struct ScriptEntitySet {
    parent: usize,
    indices: Vec<usize>,
}

impl UserData for ScriptEntitySet {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("collect", |_, set, ()| {
            let table: Vec<ScriptEntity> = set.indices.iter().map(|i| ScriptEntity::new(*i)).collect();

            Ok(table)
        });

        methods.add_method("visible_within", &visible_within);
        methods.add_method("visible", |lua, set, ()| visible_within(lua, set, std::f32::MAX));
        methods.add_method("hostile", |lua, set, ()| is_faction(lua, set, Faction::Hostile));
        methods.add_method("friendly", |lua, set, ()| is_faction(lua, set, Faction::Friendly));
        methods.add_method("reachable", &reachable);
        methods.add_method("attackable", &attackable);
    }
}

fn targets(_lua: &Lua, parent: &ScriptEntity, _args: ()) -> Result<ScriptEntitySet> {
    let area_state = GameState::area_state();
    let area_state = area_state.borrow();

    let mut indices = Vec::new();
    for entity in area_state.entity_iter() {
        indices.push(entity.borrow().index);
    }

    Ok(ScriptEntitySet { indices, parent: parent.index })
}

fn visible_within(_lua: &Lua, set: &ScriptEntitySet, dist: f32) -> Result<ScriptEntitySet> {
    filter_entities(set, dist, &|parent, entity, dist| {
        if parent.borrow().dist_to_entity(entity) > dist { return false; }

        let area_state = GameState::area_state();
        let area = &area_state.borrow().area;
        parent.borrow().has_visibility(entity, area)
    })
}

fn attackable(_lua: &Lua, set: &ScriptEntitySet, _args: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        let area_state = GameState::area_state();
        let area = &area_state.borrow().area;

        parent.borrow().can_attack(entity, area)
    })
}

fn reachable(_lua: &Lua, set: &ScriptEntitySet, _args: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        parent.borrow().can_reach(entity)
    })
}

fn is_faction(_lua: &Lua, set: &ScriptEntitySet, faction: Faction) -> Result<ScriptEntitySet> {
    filter_entities(set, faction, &|_, entity, faction| {
        entity.borrow().actor.actor.faction == faction
    })
}

fn filter_entities<T: Copy>(set: &ScriptEntitySet, t: T,
                  filter: &Fn(&Rc<RefCell<EntityState>>, &Rc<RefCell<EntityState>>, T) -> bool)
    -> Result<ScriptEntitySet> {

    let area_state = GameState::area_state();

    let parent = area_state.borrow().get_entity(set.parent);

    let mut indices = Vec::new();
    for index in set.indices.iter() {
        let entity = area_state.borrow().get_entity(*index);

        if !(filter)(&parent, &entity, t) { continue; }

        indices.push(*index);
    }

    Ok(ScriptEntitySet { indices, parent: set.parent })
}
