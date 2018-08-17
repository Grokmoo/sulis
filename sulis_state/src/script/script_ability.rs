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

use rlua::{self, Lua, UserData, UserDataMethods};

use sulis_module::{ability::{self, AIData}, Ability, Module};
use {EntityState, GameState, area_feedback_text::ColorKind};
use script::{ScriptEntity, CallbackData};

type Result<T> = std::result::Result<T, rlua::Error>;

#[derive(Clone)]
pub struct ScriptAbilitySet {
    pub parent: usize,
    pub abilities: Vec<ScriptAbility>,
}

impl ScriptAbilitySet {
    pub fn from(entity: &Rc<RefCell<EntityState>>) -> ScriptAbilitySet {
        let parent = entity.borrow().index;
        let mut abilities = Vec::new();
        for (id, _) in entity.borrow().actor.ability_states.iter() {
            let ability = Module::ability(id).unwrap();
            abilities.push(ScriptAbility::from(&ability));
        }

        ScriptAbilitySet {
            parent,
            abilities,
        }
    }
}

impl UserData for ScriptAbilitySet {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("len", |_, set, ()| {
            Ok(set.abilities.len())
        });

        methods.add_method("is_empty", |_, set, ()| {
            Ok(set.abilities.is_empty())
        });

        methods.add_method("to_table", |_, set, ()| {
            Ok(set.abilities.clone())
        });

        methods.add_method("can_activate", |_, set, ()| {
            let parent = ScriptEntity::new(set.parent).try_unwrap()?;
            let parent = parent.borrow();
            let abilities = set.abilities.iter().filter_map(|ability| {
                if parent.actor.can_activate(&ability.id) {
                    Some(ability.clone())
                } else {
                    None
                }
            }).collect();
            Ok(ScriptAbilitySet { parent: set.parent, abilities })
        });

        methods.add_method("remove_kind", |_ , set, kind: String| {
            let kind = ability::AIKind::from_str(&kind);

            let abilities = set.abilities.iter().filter_map(|ability| {
                if ability.ai_data.kind != kind {
                    Some(ability.clone())
                } else {
                    None
                }
            }).collect();
            Ok(ScriptAbilitySet { parent: set.parent, abilities })
        });

        methods.add_method("only_kind", |_, set, kind: String| {
            let kind = ability::AIKind::from_str(&kind);

            let abilities = set.abilities.iter().filter_map(|ability| {
                if ability.ai_data.kind == kind {
                    Some(ability.clone())
                } else {
                    None
                }
            }).collect();
            Ok(ScriptAbilitySet { parent: set.parent, abilities })
        });

        methods.add_method("only_group", |_, set, group: String| {
            let group = ability::AIGroup::from_str(&group);
            let abilities = set.abilities.iter().filter_map(|ability| {
                if ability.ai_data.group == group {
                    Some(ability.clone())
                } else {
                    None
                }
            }).collect();
            Ok(ScriptAbilitySet { parent: set.parent, abilities })
        });

        methods.add_method("only_range", |_, set, range: String| {
            let range = ability::AIRange::from_str(&range);
            let abilities = set.abilities.iter().filter_map(|ability| {
                if ability.ai_data.range == range {
                    Some(ability.clone())
                } else {
                    None
                }
            }).collect();
            Ok(ScriptAbilitySet { parent: set.parent, abilities })
        });

        methods.add_method_mut("sort_by_priority", |_, set, ()| {
            set.abilities.sort_by_key(|a| a.ai_data.priority());
            Ok(())
        });
    }
}

#[derive(Clone)]
pub struct ScriptAbility {
    pub id: String,
    name: String,
    duration: u32,
    ap: u32,
    ai_data: AIData,
}

impl ScriptAbility {
    pub fn from(ability: &Rc<Ability>) -> ScriptAbility {
        let (duration, ai_data) = match ability.active {
            None => unreachable!(),
            Some(ref active) => {
                let duration = match active.duration {
                    ability::Duration::Rounds(rounds) => rounds,
                    ability::Duration::Mode => 0,
                    ability::Duration::Permanent => 0,
                    ability::Duration::Instant => 0,
                };

                (duration, active.ai)
            }
        };

        let ap = match ability.active {
            None => 0,
            Some(ref active) => active.ap,
        };

        ScriptAbility {
            id: ability.id.to_string(),
            name: ability.name.to_string(),
            duration,
            ap,
            ai_data,
        }
    }

    pub fn to_ability(&self) -> Rc<Ability> {
        Module::ability(&self.id).unwrap()
    }

    fn error_if_not_active(&self) -> Result<()> {
        let ability = match Module::ability(&self.id) {
            None => unreachable!(),
            Some(ability) => ability,
        };

        match ability.active {
            None => Err(rlua::Error::FromLuaConversionError {
                from: "ScriptAbility",
                to: "ActiveAbility",
                message: Some(format!("The ability '{}' is not active", self.id)),
            }),
            Some(_) => Ok(())
        }
    }
}

impl UserData for ScriptAbility {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("is_active_mode", |_, ability, target: ScriptEntity| {
            ability.error_if_not_active()?;
            let target = target.try_unwrap()?;
            let mut target = target.borrow_mut();
            match target.actor.ability_state(&ability.id) {
                None => Ok(false),
                Some(ref ability_state) => Ok(ability_state.is_active_mode()),
            }
        });
        methods.add_method("activate", &activate);
        methods.add_method("deactivate", |_, ability, target: ScriptEntity| {
            ability.error_if_not_active()?;
            let target = target.try_unwrap()?;
            target.borrow_mut().actor.deactivate_ability_state(&ability.id);
            Ok(())
        });
        methods.add_method("name", |_, ability, ()| {
            Ok(ability.name.to_string())
        });
        methods.add_method("duration", |_, ability, ()| Ok(ability.duration));

        methods.add_method("create_callback", |_, ability, parent: ScriptEntity| {
            ability.error_if_not_active()?;
            let index = parent.try_unwrap_index()?;
            let cb_data = CallbackData::new_ability(index, &ability.id);
            Ok(cb_data)
        });

        methods.add_method("ai_data", |lua, ability, ()| {
            let ai_data = lua.create_table()?;
            ai_data.set("priority", ability.ai_data.priority())?;
            ai_data.set("kind", ability.ai_data.kind())?;
            ai_data.set("group", ability.ai_data.group())?;
            ai_data.set("range", ability.ai_data.range())?;

            Ok(ai_data)
        });
    }
}

fn activate(_lua: &Lua, ability: &ScriptAbility, (target, take_ap): (ScriptEntity, Option<bool>)) -> Result<()> {
    ability.error_if_not_active()?;
    let entity = target.try_unwrap()?;
    let take_ap = take_ap.unwrap_or(true);

    let mgr = GameState::turn_manager();
    if take_ap && mgr.borrow().is_combat_active() {
        entity.borrow_mut().actor.remove_ap(ability.ap);
    }

    let area = GameState::area_state();
    area.borrow_mut().add_feedback_text(ability.name.to_string(), &entity, ColorKind::Info);
    entity.borrow_mut().actor.activate_ability_state(&ability.id);
    Ok(())
}
