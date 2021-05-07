//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Error;
use std::rc::Rc;

use rlua::{self, Context, UserData, UserDataMethods};

use crate::script::{Result, ScriptActiveSurface, ScriptEntity};
use crate::{
    is_threat, is_within, is_within_attack_dist, is_within_touch_dist, EntityState, GameState,
};
use sulis_core::util::{gen_rand, invalid_data_error};
use sulis_module::Faction;

/// Represents a set of ScriptEntities, which can be created from a variety of
/// sources.  This is passed to many script functions as a `targets` variable.
/// It includes a parent ScriptEntity, a list of target ScriptEntities,
/// optionally a selected point (for a targeter that has been activated), and
/// optionally a list of affected points (again for a targeter).
///
/// # `num_targets() -> Int`
/// Returns the number of targets in this set.
///
/// # `to_table() -> Table`
/// Creates a table of this set.  Iterating over the table will allow you
/// to access each entity in this set.
/// ## Examples
/// ```lua
///   table = targets:to_table()
///   for i = 1, #table do
///    game:log("target: " .. table[i]:name())
///   end
/// ```
///
/// # `random_affected_points(frac: Float) -> Table`
/// Returns a table of a randomly selected subset of the affected points in this
/// set.  The probability of any individual point ending up in the returned set
/// is set by `frac`.
///
/// # `surface() -> ScriptActiveSurface`
/// Returns the surface associated with this target set, if it is defined.  Otherwise
/// throws an error.
///
/// # `affected_points() -> Table`
/// Returns a table containing all the affected points in this set.
/// ## Examples
/// ```lua
///   points = targets:affected_points()
///   for i = 1, #points do
///     point = points[i]
///     game:log("point " .. point.x .. ", " .. point.y)
///   end
/// ```
///
/// # `selected_point() -> Table`
/// Returns a table representing the selected point for this set, if one is defined.
/// The table will have `x` and `y` elements defined.  If there is no selected point,
/// throws an error.
///
/// # `is_empty() -> Bool`
/// Returns whether or not there are any targets in this ScriptEntitySet.  Does not
/// take affected points or selected_point into consideration.
///
/// # `first() -> ScriptEntity`
/// Returns the first ScriptEntity as a target in this set, or throws an error if the
/// set is empty.
///
/// # `parent() -> ScriptEntity`
/// Returns the parent ScriptEntity of this set.  When this is passed to a function as
/// `targets`, usually, but not always, the `parent` argument is the same as this.
///
/// # `without_self() -> ScriptEntitySet`
/// Creates a new ScriptEntitySet which contains all the data in this set, except
/// it does not include the parent entity as a target.
///
/// # `visible_within(dist: Float) -> ScriptEntitySet`
/// Creates a new ScriptEntitySet containing all the data in this set, except all
/// targets that are not visible or are outside the specified dist from the parent
/// are removed.
///
/// # `visible() -> ScriptEntitySet`
/// Creates a new ScriptEntitySet with all the data from this set, except only targets
/// that are visible to the parent are present.
///
/// # `hostile() -> ScriptEntitySet`
/// Creates a new ScriptEntitySet with all the data from this set, except only targets
/// that are hostile to the parent are present.
///
/// # `friendly() -> ScriptEntitySet`
/// Creates a new ScriptEntitySet with all the data from this set, except only targets
/// that are friendly to the parent are present.
///
/// # `hostile_to(faction: String) -> ScriptEntitySet`
/// Creates a new ScriptEntitySet filtered to only those targets that are hostile to
/// the specified Faction
///
/// # `friendly_to(faction: String) -> ScriptEntitySet`
/// Creates a new ScriptEntitySet filtered to only those targets that are friendly to
/// the specified Faction
///
/// # `touchable() -> ScriptEntitySet`
/// Creates a new ScriptEntitySet with all the data from this set, except only targets
/// which the parent can touch (without any weapon) are parents.
///
/// # `attackable() -> ScriptEntitySet`
/// Creates a new ScriptEntitySet with all the data from this set, except only targets
/// which the parent can attack with their current weapon are present.  If the parent
/// does not have enough AP or otherwise cannot attack, the set will be empty.
///
/// # `threatening() -> ScriptEntitySet`
/// Creates a new ScriptEntitySet with all the data from this set, except only targets
/// which can hit the parent with a melee weapon currently or in the future without moving
/// are present.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ScriptEntitySet {
    pub parent: usize,
    pub selected_point: Option<(i32, i32)>,
    pub affected_points: Vec<(i32, i32)>,
    pub indices: Vec<Option<usize>>,

    // surface is set when passing into script as argument, but should
    // never be saved as part of a callback
    #[serde(skip)]
    pub surface: Option<ScriptActiveSurface>,
}

impl ScriptEntitySet {
    pub fn update_entity_refs_on_load(
        &mut self,
        entities: &HashMap<usize, Rc<RefCell<EntityState>>>,
    ) -> ::std::result::Result<(), Error> {
        match entities.get(&self.parent) {
            None => {
                return invalid_data_error(&format!(
                    "Invalid parent {} for ScriptEntitySet",
                    self.parent
                ));
            }
            Some(ref entity) => self.parent = entity.borrow().index(),
        }

        let mut indices = Vec::new();
        for index in self.indices.drain(..) {
            match index {
                None => indices.push(None),
                Some(index) => match entities.get(&index) {
                    None => {
                        return invalid_data_error(&format!(
                            "Invalid target {} for ScriptEntitySet",
                            index
                        ));
                    }
                    Some(ref entity) => indices.push(Some(entity.borrow().index())),
                },
            }
        }
        self.indices = indices;

        Ok(())
    }

    pub fn append(&mut self, other: &ScriptEntitySet) {
        self.indices.append(&mut other.indices.clone());
        self.selected_point = other.selected_point;
        self.affected_points
            .append(&mut other.affected_points.clone());
        self.surface = other.surface.clone();
    }

    pub fn with_parent(parent: usize) -> ScriptEntitySet {
        ScriptEntitySet {
            parent,
            indices: Vec::new(),
            selected_point: None,
            affected_points: Vec::new(),
            surface: None,
        }
    }

    pub fn from_pair(
        parent: &Rc<RefCell<EntityState>>,
        target: &Rc<RefCell<EntityState>>,
    ) -> ScriptEntitySet {
        let parent = parent.borrow().index();
        let indices = vec![Some(target.borrow().index())];

        ScriptEntitySet {
            parent,
            selected_point: None,
            affected_points: Vec::new(),
            indices,
            surface: None,
        }
    }

    pub fn new(
        parent: &Rc<RefCell<EntityState>>,
        entities: &[Option<Rc<RefCell<EntityState>>>],
    ) -> ScriptEntitySet {
        let parent = parent.borrow().index();

        let indices = entities
            .iter()
            .map(|e| e.as_ref().map(|e| e.borrow().index()))
            .collect();
        ScriptEntitySet {
            parent,
            selected_point: None,
            affected_points: Vec::new(),
            indices,
            surface: None,
        }
    }
}

impl UserData for ScriptEntitySet {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("num_targets", |_, set, ()| Ok(set.indices.len()));

        methods.add_method("to_table", |_, set, ()| {
            let table: Vec<ScriptEntity> = set
                .indices
                .iter()
                .map(|i| ScriptEntity { index: *i })
                .collect();

            Ok(table)
        });

        methods.add_method("random_affected_points", |_, set, frac: f32| {
            let table: Vec<HashMap<&str, i32>> = set
                .affected_points
                .iter()
                .filter_map(|p| {
                    let roll = gen_rand(0.0, 1.0);
                    if roll > frac {
                        None
                    } else {
                        let mut map = HashMap::new();
                        map.insert("x", p.0);
                        map.insert("y", p.1);
                        Some(map)
                    }
                })
                .collect();
            Ok(table)
        });

        methods.add_method("surface", |_, set, ()| match &set.surface {
            None => {
                warn!("Attempted to get surface from target set with no surface defined");
                Err(rlua::Error::FromLuaConversionError {
                    from: "ScriptEntitySet",
                    to: "Surface",
                    message: Some("EntitySet has no surface".to_string()),
                })
            }
            Some(surf) => Ok(surf.clone()),
        });

        methods.add_method("affected_points", |_, set, ()| {
            let table: Vec<HashMap<&str, i32>> = set
                .affected_points
                .iter()
                .map(|p| {
                    let mut map = HashMap::new();
                    map.insert("x", p.0);
                    map.insert("y", p.1);
                    map
                })
                .collect();
            Ok(table)
        });

        methods.add_method("selected_point", |_, set, ()| match set.selected_point {
            None => {
                warn!("Attempted to get selected point from EntitySet where none is defined");
                Err(rlua::Error::FromLuaConversionError {
                    from: "ScriptEntitySet",
                    to: "Point",
                    message: Some("EntitySet has no selected point".to_string()),
                })
            }
            Some((x, y)) => {
                let mut point = HashMap::new();
                point.insert("x", x);
                point.insert("y", y);
                Ok(point)
            }
        });
        methods.add_method("is_empty", |_, set, ()| Ok(set.indices.is_empty()));
        methods.add_method("first", |_, set, ()| {
            if let Some(index) = set.indices.iter().flatten().next() {
                return Ok(ScriptEntity::new(*index));
            }

            warn!("Attempted to get first element of EntitySet that has no valid entities");
            Err(rlua::Error::FromLuaConversionError {
                from: "ScriptEntitySet",
                to: "ScriptEntity",
                message: Some("EntitySet is empty".to_string()),
            })
        });

        methods.add_method("parent", |_, set, ()| Ok(ScriptEntity::new(set.parent)));

        methods.add_method("without_self", &without_self);
        methods.add_method("visible_within", &visible_within);
        methods.add_method("visible", |lua, set, ()| {
            visible_within(lua, set, std::f32::MAX)
        });
        methods.add_method("hostile_to", |lua, set, faction| {
            hostile_to(lua, set, faction)
        });
        methods.add_method("friendly_to", |lua, set, faction| {
            friendly_to(lua, set, faction)
        });
        methods.add_method("hostile", |lua, set, ()| is_hostile(lua, set));
        methods.add_method("friendly", |lua, set, ()| is_friendly(lua, set));
        methods.add_method("touchable", &touchable);
        methods.add_method("attackable", &attackable);
        methods.add_method("threatening", &threatening);
    }
}

fn without_self(_lua: Context, set: &ScriptEntitySet, _: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| !Rc::ptr_eq(parent, entity))
}

fn visible_within(_lua: Context, set: &ScriptEntitySet, dist: f32) -> Result<ScriptEntitySet> {
    filter_entities(set, dist, &|parent, entity, dist| {
        let parent = &*parent.borrow();
        let entity = &*entity.borrow();
        if !is_within(parent, entity, dist) {
            return false;
        }

        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        area_state.has_visibility(parent, entity)
    })
}

fn attackable(_lua: Context, set: &ScriptEntitySet, _args: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        let parent = &*parent.borrow();
        let entity = &*entity.borrow();

        is_within_attack_dist(parent, entity)
    })
}

fn threatening(_lua: Context, set: &ScriptEntitySet, _args: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        let entity = &*entity.borrow();
        let parent = &*parent.borrow();

        is_threat(entity, parent)
    })
}

fn touchable(_lua: Context, set: &ScriptEntitySet, _args: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        let entity = &*entity.borrow();
        let parent = &*parent.borrow();
        is_within_touch_dist(parent, entity)
    })
}

fn hostile_to(_lua: Context, set: &ScriptEntitySet, faction: String) -> Result<ScriptEntitySet> {
    let faction = match Faction::option_from_str(&faction) {
        None => {
            warn!("Attempted to check hostile_to invalid faction {}", faction);
            return Err(rlua::Error::FromLuaConversionError {
                from: "String",
                to: "Faction",
                message: Some("Invalid faction ID".to_string()),
            });
        }
        Some(faction) => faction,
    };
    filter_entities(set, (), &|_, entity, _| {
        faction.is_hostile(entity.borrow().actor.faction())
    })
}

fn friendly_to(_lua: Context, set: &ScriptEntitySet, faction: String) -> Result<ScriptEntitySet> {
    let faction = match Faction::option_from_str(&faction) {
        None => {
            warn!("Attempted to check friendly_to invalid faction {}", faction);
            return Err(rlua::Error::FromLuaConversionError {
                from: "String",
                to: "Faction",
                message: Some("Invalid faction ID".to_string()),
            });
        }
        Some(faction) => faction,
    };

    filter_entities(set, (), &|_, entity, _| {
        faction.is_friendly(entity.borrow().actor.faction())
    })
}

fn is_hostile(_lua: Context, set: &ScriptEntitySet) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        parent.borrow().is_hostile(&entity.borrow())
    })
}

fn is_friendly(_lua: Context, set: &ScriptEntitySet) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        !parent.borrow().is_hostile(&entity.borrow())
    })
}

fn filter_entities<T: Copy>(
    set: &ScriptEntitySet,
    t: T,
    filter: &dyn Fn(&Rc<RefCell<EntityState>>, &Rc<RefCell<EntityState>>, T) -> bool,
) -> Result<ScriptEntitySet> {
    let parent = ScriptEntity::new(set.parent);
    let parent = parent.try_unwrap()?;

    let mgr = GameState::turn_manager();
    let mgr = mgr.borrow();

    let mut indices = Vec::new();
    for index in set.indices.iter() {
        let entity = match index {
            None => continue,
            Some(index) => mgr.entity_checked(*index),
        };

        let entity = match entity {
            None => continue,
            Some(entity) => entity,
        };

        if !(filter)(&parent, &entity, t) {
            continue;
        }

        indices.push(*index);
    }

    Ok(ScriptEntitySet {
        parent: set.parent,
        indices,
        selected_point: set.selected_point,
        affected_points: set.affected_points.clone(),
        surface: set.surface.clone(),
    })
}
