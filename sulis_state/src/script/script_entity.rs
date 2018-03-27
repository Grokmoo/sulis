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
use std::f32;
use std::rc::Rc;
use std::cell::RefCell;

use rlua::{self, Lua, UserData, UserDataMethods};

use animation::{Animation, MeleeAttackAnimation};
use sulis_rules::{AttackKind, DamageKind, Attack};
use sulis_core::config::CONFIG;
use sulis_module::Faction;
use {EntityState, GameState};
use script::{CallbackData, Result, ScriptAbility, ScriptCallback, ScriptEffect, ScriptParticleGenerator, TargeterData};

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
            info!("Got here");
            let duration = args.1;
            let ability = args.0;
            Ok(ScriptEffect::new(entity.index, &ability, duration))
        });

        methods.add_method("create_particle_generator", |_, entity, args: (String, Option<f32>)| {
            let duration_secs = args.1.unwrap_or(f32::INFINITY);
            let sprite = args.0;
            Ok(ScriptParticleGenerator::new(entity.index, sprite, duration_secs))
        });

        methods.add_method("create_anim", |_, entity, (image, duration): (String, Option<f32>)| {
            let duration = duration.unwrap_or(f32::INFINITY);
            Ok(ScriptParticleGenerator::new_anim(entity.index, image, duration))
        });

        methods.add_method("create_targeter", |_, entity, ability: ScriptAbility| {
            Ok(TargeterData::new(entity.index, &ability.id))
        });

        methods.add_method("weapon_attack", |_, entity, target: ScriptEntity| {
            let area_state = GameState::area_state();
            let target = area_state.borrow().get_entity(target.index);
            let parent = area_state.borrow().get_entity(entity.index);
            let (_, text, color) = parent.borrow_mut().actor.weapon_attack(&target);

            area_state.borrow_mut().add_feedback_text(text, &target, color);

            Ok(())
        });

        methods.add_method("anim_weapon_attack", |_, entity, (target, callback):
                           (ScriptEntity, Option<CallbackData>)| {
            let area_state = GameState::area_state();
            let target = area_state.borrow().get_entity(target.index);
            let parent = area_state.borrow().get_entity(entity.index);

            let cb: Option<Box<ScriptCallback>> = match callback {
                None => None,
                Some(cb) => Some(Box::new(cb)),
            };

            EntityState::attack(&parent, &target, cb);
            Ok(())
        });

        methods.add_method("anim_special_attack", |_, entity,
                           (target, attack_kind, min_damage, max_damage, damage_kind, cb):
                           (ScriptEntity, String, u32, u32, String, Option<CallbackData>)| {
            let area_state = GameState::area_state();

            let target = area_state.borrow().get_entity(target.index);
            let parent = area_state.borrow().get_entity(entity.index);

            let damage_kind = DamageKind::from_str(&damage_kind);
            let attack_kind = AttackKind::from_str(&attack_kind);

            let time = CONFIG.display.animation_base_time_millis * 5;
            let mut anim = MeleeAttackAnimation::new(&parent, &target, time, Box::new(move |att, def| {
                let attack = Attack::special(min_damage, max_damage, damage_kind, attack_kind.clone());

                att.borrow_mut().actor.attack(def, &attack)
            }));

            if let Some(cb) = cb {
                anim.set_callback(Some(Box::new(cb)));
            }
            GameState::add_animation(Box::new(anim));
            Ok(())
        });

        methods.add_method("special_attack", |_, entity,
                           (target, attack_kind, min_damage, max_damage, damage_kind):
                           (ScriptEntity, String, u32, u32, String)| {
            let area_state = GameState::area_state();

            let target = area_state.borrow().get_entity(target.index);
            let parent = area_state.borrow().get_entity(entity.index);

            let damage_kind = DamageKind::from_str(&damage_kind);
            let attack_kind = AttackKind::from_str(&attack_kind);

            let attack = Attack::special(min_damage, max_damage, damage_kind, attack_kind);

            let (_hit_kind, text, color) = parent.borrow_mut().actor.attack(&target, &attack);

            area_state.borrow_mut().add_feedback_text(text, &target, color);

            Ok(())
        });

        methods.add_method("change_overflow_ap", |_, entity, ap| {
            let area_state = GameState::area_state();
            let entity = area_state.borrow().get_entity(entity.index);
            entity.borrow_mut().actor.change_overflow_ap(ap);
            Ok(())
        });

        methods.add_method("remove_ap", |_, entity, ap| {
            let area_state = GameState::area_state();
            let entity = area_state.borrow().get_entity(entity.index);
            entity.borrow_mut().actor.remove_ap(ap);
            Ok(())
        });

        methods.add_method("stats", &create_stats_table);

        methods.add_method("x", |_, entity, ()| {
            let area_state = GameState::area_state();
            let entity = area_state.borrow().get_entity(entity.index);
            let x = entity.borrow().location.x as f32 + entity.borrow().size.width as f32 / 2.0;
            Ok(x)
        });

        methods.add_method("y", |_, entity, ()| {
            let area_state = GameState::area_state();
            let entity = area_state.borrow().get_entity(entity.index);
            let y = entity.borrow().location.y as f32 + entity.borrow().size.height as f32 / 2.0;
            Ok(y)
        });

        methods.add_method("dist", |_, entity, target: ScriptEntity| {
            let area_state = GameState::area_state();
            let entity = area_state.borrow().get_entity(entity.index);
            let target = area_state.borrow().get_entity(target.index);
            let entity = entity.borrow();
            Ok(entity.dist_to_entity(&target))
        });
    }
}

fn create_stats_table<'a>(lua: &'a Lua, parent: &ScriptEntity, _args: ()) -> Result<rlua::Table<'a>> {
    let area_state = GameState::area_state();
    let parent = area_state.borrow().get_entity(parent.index);
    let src = &parent.borrow().actor.stats;

    let stats = lua.create_table()?;
    stats.set("strength", src.attributes.strength)?;
    stats.set("dexterity", src.attributes.dexterity)?;
    stats.set("endurance", src.attributes.endurance)?;
    stats.set("perception", src.attributes.perception)?;
    stats.set("intellect", src.attributes.intellect)?;
    stats.set("wisdom", src.attributes.wisdom)?;

    stats.set("base_armor", src.armor.base())?;
    let armor = lua.create_table()?;
    for kind in DamageKind::iter() {
        armor.set(kind.to_str(), src.armor.amount(*kind))?;
    }
    stats.set("armor", armor)?;

    stats.set("bonus_reach", src.bonus_reach)?;
    stats.set("bonus_range", src.bonus_range)?;
    stats.set("max_hp", src.max_hp)?;
    stats.set("initiative", src.initiative)?;
    stats.set("accuracy", src.accuracy)?;
    stats.set("defense", src.defense)?;
    stats.set("fortitude", src.fortitude)?;
    stats.set("reflex", src.reflex)?;
    stats.set("will", src.will)?;

    stats.set("attack_distance", src.attack_distance() + parent.borrow().size.diagonal / 2.0)?;
    stats.set("attack_is_melee", src.attack_is_melee())?;
    stats.set("attack_is_ranged", src.attack_is_ranged())?;

    Ok(stats)
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

        methods.add_method("is_empty", |_, set, ()| Ok(set.indices.is_empty()));
        methods.add_method("first", |_, set, ()| {
            if set.indices.is_empty() {
                warn!("Attempted to get first element of empty EntitySet");
                return Err(rlua::Error::FromLuaConversionError {
                    from: "ScriptEntitySet",
                    to: "ScriptEntity",
                    message: Some("EntitySet is empty".to_string())
                });
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
