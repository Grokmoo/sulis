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

use sulis_rules::StatList;
use sulis_module::Faction;
use {EntityState, GameState};
use script::{CallbackData, Result, ScriptAbility, ScriptEffect, TargeterData};

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

        methods.add_method("create_effect", |_, entity, args: (String, u32)| {
            let duration = args.1;
            let ability = args.0;
            Ok(ScriptEffect::new(entity.index, &ability, duration))
        });

        methods.add_method("create_targeter", |_, entity, ability: ScriptAbility| {
            Ok(TargeterData::new(entity.index, &ability.id))
        });

        methods.add_method("attack", |_, entity, (target, callback): (ScriptEntity, CallbackData)| {
            let area_state = GameState::area_state();
            let target = area_state.borrow().get_entity(target.index);
            let parent = area_state.borrow().get_entity(entity.index);

            EntityState::attack(&parent, &target, Some(Box::new(callback)));
            Ok(())
        });

        methods.add_method("remove_ap", |_, entity, ap| {
            let area_state = GameState::area_state();
            let entity = area_state.borrow().get_entity(entity.index);
            entity.borrow_mut().actor.remove_ap(ap);
            Ok(())
        });

        methods.add_method("stats", |_, entity, ()| {
            let area_state = GameState::area_state();
            let parent = area_state.borrow().get_entity(entity.index);

            let stats = parent.borrow().actor.stats.clone();

            Ok(ScriptEntityStats { stats })
        });
    }
}

#[derive(Clone)]
pub struct ScriptEntityStats {
    stats: StatList,
}

impl UserData for ScriptEntityStats {
    fn add_methods(methods: &mut UserDataMethods<Self>) {

    }
}

#[derive(Clone)]
pub struct ScriptEntitySet {
    pub parent: usize,
    pub indices: Vec<usize>,
}

impl ScriptEntitySet {
    pub fn new(parent: &Rc<RefCell<EntityState>>, entities: &Vec<Rc<RefCell<EntityState>>>) -> ScriptEntitySet {
        let parent = parent.borrow().index;
        let indices = entities.iter().map(|e| e.borrow().index).collect();
        ScriptEntitySet {
            parent,
            indices,
        }
    }
}

impl UserData for ScriptEntitySet {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("collect", |_, set, ()| {
            let table: Vec<ScriptEntity> = set.indices.iter().map(|i| ScriptEntity::new(*i)).collect();

            Ok(table)
        });

        methods.add_method("first", |_, set, ()| {
            if set.indices.is_empty() {
                panic!("EntitySet is empty");
            }

            Ok(ScriptEntity::new(*set.indices.first().unwrap()))
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
