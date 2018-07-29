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

use std::{self, f32, u32};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use rand::{self, Rng};
use rlua::{self, Lua, UserData, UserDataMethods};

use sulis_core::util::ExtInt;
use sulis_core::config::CONFIG;
use sulis_core::resource::ResourceSet;
use sulis_rules::{AttackKind, DamageKind, Attack};
use {ActorState, EntityState, GameState, Location, area_feedback_text::ColorKind};
use {ai, animation::{self}, script::*};

#[derive(Clone, Debug)]
pub struct ScriptEntity {
    pub index: Option<usize>,
}

impl ScriptEntity {
    pub fn invalid() -> ScriptEntity {
        ScriptEntity { index: None }
    }

    pub fn new(index: usize) -> ScriptEntity {
        ScriptEntity { index: Some(index) }
    }

    pub fn from(entity: &Rc<RefCell<EntityState>>) -> ScriptEntity {
        ScriptEntity { index: Some(entity.borrow().index) }
    }

    pub fn check_not_equal(&self, other: &ScriptEntity) -> Result<()> {
        if self.index == other.index {
            warn!("Parent and target must not refer to the same entity for this method");
            Err(rlua::Error::FromLuaConversionError {
                from: "ScriptEntity",
                to: "ScriptEntity",
                message: Some("Parent and target must not match".to_string())
            })
        } else {
            Ok(())
        }
    }

    pub fn try_unwrap_index(&self) -> Result<usize> {
        match self.index {
            None => Err(rlua::Error::FromLuaConversionError {
                from: "ScriptEntity",
                to: "EntityState",
                message: Some("ScriptEntity does not have a valid index".to_string())
            }),
            Some(index) => Ok(index),
        }
    }

    pub fn try_unwrap(&self) -> Result<Rc<RefCell<EntityState>>> {
        match self.index {
            None => Err(rlua::Error::FromLuaConversionError {
                from: "ScriptEntity",
                to: "EntityState",
                message: Some("ScriptEntity does not have a valid index".to_string())
            }),
            Some(index) => {
                let mgr = GameState::turn_manager();
                let mgr = mgr.borrow();
                match mgr.entity_checked(index) {
                    None => Err(rlua::Error::FromLuaConversionError {
                        from: "ScriptEntity",
                        to: "EntityState",
                        message: Some("ScriptEntity refers to an entity that no longer exists.".to_string())
                    }),
                    Some(entity) => Ok(entity),
                }
            }
        }
    }
}

impl UserData for ScriptEntity {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("state_end", |_, _, ()| {
            Ok(ai::State::End)
        });

        methods.add_method("state_wait", |_, _, time: u32| {
            Ok(ai::State::Wait(time))
        });

        methods.add_method("vis_dist", |_, entity, ()| {
            let parent = entity.try_unwrap()?;
            let area_id = &parent.borrow().location.area_id;
            let area = GameState::get_area_state(area_id).unwrap();
            let dist = area.borrow().area.vis_dist as f32;
            Ok(dist)
        });

        methods.add_method("add_xp", |_, entity, amount: u32| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().actor.add_xp(amount);
            Ok(())
        });

        methods.add_method("set_flag", |_, entity, (flag, val): (String, Option<String>)| {
            let entity = entity.try_unwrap()?;
            let val = match &val {
                None => "true",
                Some(val) => val,
            };

            entity.borrow_mut().set_custom_flag(&flag, val);
            Ok(())
        });

        methods.add_method("clear_flag", |_, entity, flag: String| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().clear_custom_flag(&flag);
            Ok(())
        });

        methods.add_method("has_flag", |_, entity, flag: String| {
            let entity = entity.try_unwrap()?;
            let result = entity.borrow().has_custom_flag(&flag);
            Ok(result)
        });

        methods.add_method("get_flag", |_, entity, flag: String| {
            let entity = entity.try_unwrap()?;
            let result = entity.borrow().get_custom_flag(&flag);
            Ok(result)
        });

        methods.add_method("is_valid", |_, entity, ()| {
            let mgr = GameState::turn_manager();
            match entity.index {
                None => Ok(false),
                Some(index) => Ok(mgr.borrow().has_entity(index)),
            }
        });

        methods.add_method("is_party_member", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let is_member = entity.borrow().is_party_member();
            Ok(is_member)
        });

        methods.add_method("use_ability", |_, entity, ability: ScriptAbility| {
            let parent = entity.try_unwrap()?;
            info!("ability on activate script");
            GameState::execute_ability_on_activate(&parent, &ability.to_ability());
            Ok(())
        });

        methods.add_method("abilities", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            Ok(ScriptAbilitySet::from(&entity))
        });

        methods.add_method("targets", &targets);

        methods.add_method("remove_effects_with_tag", |_, entity, tag: String| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();

            let mgr = GameState::turn_manager();
            let mut mgr = mgr.borrow_mut();

            for effect_index in entity.actor.effects_iter() {
                let effect = mgr.effect_mut(*effect_index);
                if effect.tag == tag {
                    effect.mark_for_removal();
                }
            }

            Ok(())
        });

        methods.add_method("create_surface", |_, _, (name, points, duration):
            (String, Vec<HashMap<String, i32>>, Option<u32>)| {
            let duration = match duration {
                None => ExtInt::Infinity,
                Some(dur) => ExtInt::Int(dur),
            };
            let points: Vec<(i32, i32)> = points.into_iter().map(|p| {
                let x = p.get("x").unwrap();
                let y = p.get("y").unwrap();
                (*x, *y)
            }).collect();
            Ok(ScriptEffect::new_surface(points, &name, duration))
        });

        methods.add_method("create_effect", |_, entity, args: (String, Option<u32>)| {
            let duration = match args.1 {
                None => ExtInt::Infinity,
                Some(dur) => ExtInt::Int(dur),
            };
            let ability = args.0;
            let index = entity.try_unwrap_index()?;
            Ok(ScriptEffect::new_entity(index, &ability, duration))
        });

        methods.add_method("create_subpos_anim", |_, entity, duration_secs: f32| {
            let index = entity.try_unwrap_index()?;
            let duration = ExtInt::Int((duration_secs * 1000.0) as u32);
            Ok(ScriptSubposAnimation::new(index, duration))
        });

        methods.add_method("create_color_anim", |_, entity, duration_secs: Option<f32>| {
            let index = entity.try_unwrap_index()?;
            let duration = match duration_secs {
                None => ExtInt::Infinity,
                Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
            };
            Ok(ScriptColorAnimation::new(index, duration))
        });

        methods.add_method("create_particle_generator", |_, entity, args: (String, Option<f32>)| {
            let sprite = args.0;
            let index = entity.try_unwrap_index()?;
            let duration = match args.1 {
                None => ExtInt::Infinity,
                Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
            };
            Ok(ScriptParticleGenerator::new(index, sprite, duration))
        });

        methods.add_method("wait_anim", |_, entity, duration: f32| {
            let index = entity.try_unwrap_index()?;
            let image = ResourceSet::get_empty_image();
            let duration = ExtInt::Int((duration * 1000.0) as u32);
            Ok(ScriptParticleGenerator::new_anim(index, image.id(), duration))
        });

        methods.add_method("create_anim", |_, entity, (image, duration): (String, Option<f32>)| {
            let duration = match duration {
                None => ExtInt::Infinity,
                Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
            };
            let index = entity.try_unwrap_index()?;
            Ok(ScriptParticleGenerator::new_anim(index, image, duration))
        });

        methods.add_method("create_targeter", |_, entity, ability: ScriptAbility| {
            let index = entity.try_unwrap_index()?;
            Ok(TargeterData::new_ability(index, &ability.id))
        });

        methods.add_method("create_targeter_for_item", |_, entity, item: ScriptItem| {
            let index = entity.try_unwrap_index()?;
            Ok(TargeterData::new_item(index, item.index))
        });

        methods.add_method("move_towards_entity", |_, entity, (dest, dist):
                           (ScriptEntity, Option<f32>)| {
            let parent = entity.try_unwrap()?;
            let target = dest.try_unwrap()?;

            if let Some(dist) = dist {
                let (x, y) = {
                    let target = target.borrow();
                    (target.location.x as f32 + (target.size.width / 2) as f32,
                     target.location.y as f32 + (target.size.height / 2) as f32)
                };
                GameState::move_towards_point(&parent, Vec::new(), x, y, dist, None);
            } else {
                GameState::move_towards(&parent, &target);
            }

            Ok(())
        });

        methods.add_method("has_ap_to_attack", |_, entity, ()| {
            let parent = entity.try_unwrap()?;
            let result = parent.borrow().actor.has_ap_to_attack();
            Ok(result)
        });

        methods.add_method("can_reach", |_, entity, target: ScriptEntity| {
            let parent = entity.try_unwrap()?;
            let target = target.try_unwrap()?;
            let result = parent.borrow().can_reach(&target);
            Ok(result)
        });

        methods.add_method("can_move", |_, entity, ()| {
            let parent = entity.try_unwrap()?;
            let result = parent.borrow().can_move();
            Ok(result)
        });

        methods.add_method("teleport_to", |_, entity, dest: HashMap<String, i32>| {
            let (x, y) = unwrap_point(dest)?;
            let entity = entity.try_unwrap()?;
            let entity_index = entity.borrow().index;
            let mgr = GameState::turn_manager();

            let area_state = GameState::area_state();
            if !entity.borrow().location.is_in(&area_state.borrow()) {
                let old_area_state = GameState::get_area_state(
                    &entity.borrow().location.area_id).unwrap();

                let surfaces = old_area_state.borrow_mut().remove_entity(&entity);
                for surface in surfaces {
                    mgr.borrow_mut().remove_from_surface(entity_index, surface);
                }

                let new_loc = Location::new(x, y, &area_state.borrow().area);
                match area_state.borrow_mut().transition_entity_to(entity, entity_index, new_loc) {
                    Err(e) => {
                        warn!("Unable to move entity using script function");
                        warn!("{}", e);
                    }, Ok(_) => (),
                }
            } else {
                let mut area_state = area_state.borrow_mut();
                area_state.move_entity(&entity, x, y, 0);
            }

            Ok(())
        });

        methods.add_method("weapon_attack", |_, entity, target: ScriptEntity| {
            let target = target.try_unwrap()?;
            let parent = entity.try_unwrap()?;
            let (hit_kind, damage, text, color) = ActorState::weapon_attack(&parent, &target);

            let area_state = GameState::area_state();
            area_state.borrow_mut().add_feedback_text(text, &target, color, 3.0);

            let hit_kind = ScriptHitKind { kind: hit_kind, damage };
            Ok(hit_kind)
        });

        methods.add_method("anim_weapon_attack", |_, entity, (target, callback, use_ap):
                           (ScriptEntity, Option<CallbackData>, Option<bool>)| {
            entity.check_not_equal(&target)?;
            let parent = entity.try_unwrap()?;
            let target = target.try_unwrap()?;

            let cb: Option<Box<ScriptCallback>> = match callback {
                None => None,
                Some(cb) => Some(Box::new(cb)),
            };

            let use_ap = use_ap.unwrap_or(false);

            EntityState::attack(&parent, &target, cb, use_ap);
            Ok(())
        });

        methods.add_method("anim_special_attack", |_, entity,
            (target, attack_kind, accuracy_kind, min_damage, max_damage, ap, damage_kind, cb):
            (ScriptEntity, String, String, u32, u32, u32, String, Option<CallbackData>)| {

            entity.check_not_equal(&target)?;
            let parent = entity.try_unwrap()?;
            let target = target.try_unwrap()?;
            let damage_kind = DamageKind::from_str(&damage_kind);
            let attack_kind = AttackKind::from_str(&attack_kind, &accuracy_kind);
            let mut cbs: Vec<Box<ScriptCallback>> = Vec::new();
            if let Some(cb) = cb {
                cbs.push(Box::new(cb));
            }
            let time = CONFIG.display.animation_base_time_millis * 5;
            let anim = animation::melee_attack_animation::new(&Rc::clone(&parent), &target,
                                                              time, cbs, Box::new(move |att, def| {
                let attack = Attack::special(&parent.borrow().actor.stats, min_damage, max_damage,
                    ap, damage_kind, attack_kind.clone());

                ActorState::attack(att, def, &attack)
            }));

            GameState::add_animation(anim);
            Ok(())
        });

        methods.add_method("special_attack", |_, entity,
            (target, attack_kind, accuracy_kind, min_damage, max_damage, ap, damage_kind):
            (ScriptEntity, String, String, Option<u32>, Option<u32>, Option<u32>, Option<String>)| {
            let target = target.try_unwrap()?;
            let parent = entity.try_unwrap()?;

            let damage_kind = match damage_kind {
                None => DamageKind::Raw,
                Some(ref kind) => DamageKind::from_str(kind),
            };
            let attack_kind = AttackKind::from_str(&attack_kind, &accuracy_kind);

            let min_damage = min_damage.unwrap_or(0);
            let max_damage = max_damage.unwrap_or(0);
            let ap = ap.unwrap_or(0);

            let attack = Attack::special(&parent.borrow().actor.stats,
                min_damage, max_damage, ap, damage_kind, attack_kind);

            let (hit_kind, damage, text, color) = ActorState::attack(&parent, &target, &attack);

            let area_state = GameState::area_state();
            area_state.borrow_mut().add_feedback_text(text, &target, color, 3.0);

            let hit_kind = ScriptHitKind { kind: hit_kind, damage };
            Ok(hit_kind)
        });

        methods.add_method("take_damage", |_, entity, (min_damage, max_damage, damage_kind, ap):
                           (u32, u32, String, Option<u32>)| {
            let parent = entity.try_unwrap()?;
            let damage_kind = DamageKind::from_str(&damage_kind);

            let attack = Attack::special(&parent.borrow().actor.stats, min_damage, max_damage,
                ap.unwrap_or(0), damage_kind, AttackKind::Dummy);
            let damage = attack.roll_damage(&parent.borrow().actor.stats.armor, 1.0);

            let (text, color) = if !damage.is_empty() {
                let mut total = 0;
                for (_, amount) in damage {
                    total += amount;
                }

                parent.borrow_mut().remove_hp(total);
                (format!("{}", total), ColorKind::Hit)
            } else {
                ("0".to_string(), ColorKind::Miss)
            };

            let area_state = GameState::area_state();
            area_state.borrow_mut().add_feedback_text(text, &parent, color, 3.0);
            Ok(())
        });

        methods.add_method("heal_damage", |_, entity, amount: u32| {
            let parent = entity.try_unwrap()?;
            parent.borrow_mut().actor.add_hp(amount);
            let area_state = GameState::area_state();
            area_state.borrow_mut().add_feedback_text(format!("{}", amount), &parent,
                ColorKind::Heal, 3.0);

            Ok(())
        });

        methods.add_method("get_overflow_ap", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let ap = entity.borrow().actor.overflow_ap();
            Ok(ap)
        });

        methods.add_method("change_overflow_ap", |_, entity, ap| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().actor.change_overflow_ap(ap);
            Ok(())
        });

        methods.add_method("set_subpos", |_, entity, (x, y): (f32, f32)| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().sub_pos = (x, y);
            Ok(())
        });

        methods.add_method("remove_ap", |_, entity, ap| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().actor.remove_ap(ap);
            Ok(())
        });

        methods.add_method("name", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.actor.actor.name.to_string())
        });

        methods.add_method("get_ability", |_, entity, id: String| {
            let ability = match Module::ability(&id) {
                None => return Err(rlua::Error::FromLuaConversionError {
                    from: "String",
                    to: "ScriptAbility",
                    message: Some(format!("Ability '{}' does not exist", id))
                }),
                Some(ability) => ability,
            };
            let entity = entity.try_unwrap()?;
            if !entity.borrow().actor.actor.has_ability(&ability) {
                return Ok(None);
            }

            Ok(Some(ScriptAbility::from(&ability)))
        });

        methods.add_method("ability_level", |_, entity, ability: ScriptAbility| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();

            match entity.actor.actor.ability_level(&ability.id) {
                None => Ok(0),
                Some(level) => Ok(level + 1),
            }
        });

        methods.add_method("has_active_mode", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            for (_, ref state) in entity.actor.ability_states.iter() {
                if state.is_active_mode() { return Ok(true); }
            }
            Ok(false)
        });

        methods.add_method("stats", &create_stats_table);

        methods.add_method("inventory", |_, entity, ()| {
            Ok(ScriptInventory::new(entity.clone()))
        });

        methods.add_method("size_str", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.size().to_string())
        });
        methods.add_method("width", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.size.width)
        });
        methods.add_method("height", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.size.height)
        });
        methods.add_method("x", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let x = entity.borrow().location.x;
            Ok(x)
        });
        methods.add_method("y", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let y = entity.borrow().location.y;
            Ok(y)
        });
        methods.add_method("center_x", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let x = entity.borrow().location.x as f32 + entity.borrow().size.width as f32 / 2.0;
            Ok(x)
        });

        methods.add_method("center_y", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let y = entity.borrow().location.y as f32 + entity.borrow().size.height as f32 / 2.0;
            Ok(y)
        });

        methods.add_method("dist_to_entity", |_, entity, target: ScriptEntity| {
            let entity = entity.try_unwrap()?;
            let target = target.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.dist_to_entity(&target))
        });

        methods.add_method("dist_to_point", |_, entity, point: HashMap<String, i32>| {
            let (x, y) = unwrap_point(point)?;
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.dist_to_point(Point::new(x, y)))
        });
    }
}

pub fn unwrap_point(point: HashMap<String, i32>) -> Result<(i32, i32)> {
    let x = match point.get("x") {
        None => return Err(rlua::Error::FromLuaConversionError {
            from: "ScriptPoint",
            to: "Point",
            message: Some("Point must have x and y coordinates".to_string())
        }),
        Some(x) => *x,
    };

    let y = match point.get("y") {
        None => return Err(rlua::Error::FromLuaConversionError {
            from: "ScriptPoint",
            to: "Point",
            message: Some("Point must have x and y coordinates".to_string())
        }),
        Some(y) => *y,
    };

    Ok((x, y))
}

fn create_stats_table<'a>(lua: &'a Lua, parent: &ScriptEntity, _args: ()) -> Result<rlua::Table<'a>> {
    let parent = parent.try_unwrap()?;
    let parent = parent.borrow();
    let src = &parent.actor.stats;

    let stats = lua.create_table()?;
    stats.set("current_hp", parent.actor.hp())?;
    stats.set("current_ap", parent.actor.ap())?;

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
    stats.set("melee_accuracy", src.melee_accuracy)?;
    stats.set("ranged_accuracy", src.ranged_accuracy)?;
    stats.set("spell_accuracy", src.spell_accuracy)?;
    stats.set("defense", src.defense)?;
    stats.set("fortitude", src.fortitude)?;
    stats.set("reflex", src.reflex)?;
    stats.set("will", src.will)?;

    stats.set("attack_distance", src.attack_distance() + parent.size.diagonal / 2.0)?;
    stats.set("attack_is_melee", src.attack_is_melee())?;
    stats.set("attack_is_ranged", src.attack_is_ranged())?;

    stats.set("concealment", src.concealment)?;
    stats.set("concealment_ignore", src.concealment_ignore)?;
    stats.set("crit_threshold", src.crit_threshold)?;
    stats.set("graze_threshold", src.graze_threshold)?;
    stats.set("hit_threshold", src.hit_threshold)?;
    stats.set("graze_multiplier", src.graze_multiplier)?;
    stats.set("hit_multiplier", src.hit_multiplier)?;
    stats.set("crit_multiplier", src.crit_multiplier)?;
    stats.set("movement_rate", src.movement_rate)?;
    stats.set("attack_cost", src.attack_cost)?;

    if let Some(image) = src.get_ranged_projectile() {
        stats.set("ranged_projectile", image.id())?;
    }

    for (index, attack) in src.attacks.iter().enumerate() {
        stats.set(format!("damage_min_{}", index), attack.damage.min())?;
        stats.set(format!("damage_max_{}", index), attack.damage.max())?;
        stats.set(format!("armor_penetration_{}", index), attack.damage.ap())?;
    }

    Ok(stats)
}

#[derive(Clone, Debug)]
pub struct ScriptEntitySet {
    pub parent: usize,
    pub selected_point: Option<(i32, i32)>,
    pub affected_points: Vec<(i32, i32)>,
    pub indices: Vec<Option<usize>>,
}

impl ScriptEntitySet {
    pub fn append(&mut self, other: &ScriptEntitySet) {
        self.indices.append(&mut other.indices.clone());
        self.selected_point = other.selected_point.clone();
        self.affected_points.append(&mut other.affected_points.clone());
    }

    pub fn with_parent(parent: usize) -> ScriptEntitySet {
        ScriptEntitySet {
            parent,
            indices: Vec::new(),
            selected_point: None,
            affected_points: Vec::new(),
        }
    }

    pub fn new(parent: &Rc<RefCell<EntityState>>,
               entities: &Vec<Option<Rc<RefCell<EntityState>>>>) -> ScriptEntitySet {
        let parent = parent.borrow().index;

        let indices = entities.iter().map(|e| {
            match e {
                &None => None,
                &Some(ref e) => Some(e.borrow().index),
            }
        }).collect();
        ScriptEntitySet {
            parent,
            selected_point: None,
            affected_points: Vec::new(),
            indices,
        }
    }
}

impl UserData for ScriptEntitySet {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("to_table", |_, set, ()| {
            let table: Vec<ScriptEntity> = set.indices.iter().map(|i| ScriptEntity { index: *i }).collect();

            Ok(table)
        });

        methods.add_method("random_affected_points", |_, set, frac: f32| {
            let table: Vec<HashMap<&str, i32>> = set.affected_points.iter().filter_map(|p| {
                let roll = rand::thread_rng().gen_range(0.0, 1.0);
                if roll > frac {
                    None
                } else {
                    let mut map = HashMap::new();
                    map.insert("x", p.0);
                    map.insert("y", p.1);
                    Some(map)
                }
            }).collect();
            Ok(table)
        });

        methods.add_method("affected_points", |_, set, ()| {
            let table: Vec<HashMap<&str, i32>> = set.affected_points.iter().map(|p| {
                let mut map = HashMap::new();
                map.insert("x", p.0);
                map.insert("y", p.1);
                map
            }).collect();
            Ok(table)
        });

        methods.add_method("selected_point", |_, set, ()| {
            match set.selected_point {
                None => {
                    warn!("Attempted to get selected point from EntitySet where none is defined");
                    Err(rlua::Error::FromLuaConversionError {
                        from: "ScriptEntitySet",
                        to: "Point",
                        message: Some("EntitySet has no selected point".to_string())
                    })
                }, Some((x, y)) => {
                    let mut point = HashMap::new();
                    point.insert("x", x);
                    point.insert("y", y);
                    Ok(point)
                }
            }
        });
        methods.add_method("is_empty", |_, set, ()| Ok(set.indices.is_empty()));
        methods.add_method("first", |_, set, ()| {
            for index in set.indices.iter() {
                if let &Some(index) = index {
                    return Ok(ScriptEntity::new(index));
                }
            }

            warn!("Attempted to get first element of EntitySet that has no valid entities");
            Err(rlua::Error::FromLuaConversionError {
                from: "ScriptEntitySet",
                to: "ScriptEntity",
                message: Some("EntitySet is empty".to_string())
            })
        });

        methods.add_method("without_self", &without_self);
        methods.add_method("visible_within", &visible_within);
        methods.add_method("visible", |lua, set, ()| visible_within(lua, set, std::f32::MAX));
        methods.add_method("hostile", |lua, set, ()| is_hostile(lua, set));
        methods.add_method("friendly", |lua, set, ()| is_friendly(lua, set));
        methods.add_method("reachable", &reachable);
        methods.add_method("attackable", &attackable);
    }
}

fn targets(_lua: &Lua, parent: &ScriptEntity, _args: ()) -> Result<ScriptEntitySet> {
    let parent = parent.try_unwrap()?;
    let area_id = parent.borrow().location.area_id.to_string();

    let mgr = GameState::turn_manager();
    let mut indices = Vec::new();
    for entity in mgr.borrow().entity_iter() {
        if parent.borrow().is_hostile(&entity) && entity.borrow().actor.stats.hidden {
            continue;
        }

        let entity = entity.borrow();
        if !entity.location.is_in_area_id(&area_id) { continue; }

        indices.push(Some(entity.index));
    }

    let parent_index = parent.borrow().index;
    Ok(ScriptEntitySet {
        parent: parent_index,
        indices,
        selected_point: None,
        affected_points: Vec::new(),
    })
}

fn without_self(_lua: &Lua, set: &ScriptEntitySet, _: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        !Rc::ptr_eq(parent, entity)
    })
}

fn visible_within(_lua: &Lua, set: &ScriptEntitySet, dist: f32) -> Result<ScriptEntitySet> {
    filter_entities(set, dist, &|parent, entity, dist| {
        if parent.borrow().dist_to_entity(entity) > dist { return false; }

        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        area_state.has_visibility(&parent.borrow(), &entity.borrow())
    })
}

fn attackable(_lua: &Lua, set: &ScriptEntitySet, _args: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        parent.borrow().can_attack(entity, &area_state)
    })
}

fn reachable(_lua: &Lua, set: &ScriptEntitySet, _args: ()) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        parent.borrow().can_reach(entity)
    })
}

fn is_hostile(_lua: &Lua, set: &ScriptEntitySet) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        parent.borrow().is_hostile(entity)
    })
}

fn is_friendly(_lua: &Lua, set: &ScriptEntitySet) -> Result<ScriptEntitySet> {
    filter_entities(set, (), &|parent, entity, _| {
        !parent.borrow().is_hostile(entity)
    })
}

fn filter_entities<T: Copy>(set: &ScriptEntitySet, t: T,
                  filter: &Fn(&Rc<RefCell<EntityState>>, &Rc<RefCell<EntityState>>, T) -> bool)
    -> Result<ScriptEntitySet> {

    let parent = ScriptEntity::new(set.parent);
    let parent = parent.try_unwrap()?;

    let mgr = GameState::turn_manager();
    let mgr = mgr.borrow();

    let mut indices = Vec::new();
    for index in set.indices.iter() {
        let entity = match index {
            &None => continue,
            &Some(index) => mgr.entity_checked(index),
        };

        let entity = match entity {
            None => continue,
            Some(entity) => entity,
        };

        if !(filter)(&parent, &entity, t) { continue; }

        indices.push(*index);
    }

    Ok(ScriptEntitySet {
        parent: set.parent,
        indices,
        selected_point: set.selected_point,
        affected_points: set.affected_points.clone(),
    })
}
