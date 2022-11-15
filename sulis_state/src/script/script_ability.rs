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

use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;

use rlua::{self, Context, UserData, UserDataMethods};

use crate::script::{CallbackData, ScriptEntity};
use crate::{area_feedback_text::ColorKind, AreaFeedbackText, EntityState, GameState};
use sulis_module::{
    ability::{self, AIData, Range},
    Ability, Module,
};

type Result<T> = std::result::Result<T, rlua::Error>;

/// Represents the set of abilities that a given Entity has access to.
/// This will only include active abilities, not passive ones.
/// See `ScriptEntity`
/// # `len() -> Int`
/// Returns the number of abilities in this set
///
/// # `is_empty() -> Bool`
/// Returns true if there are no abilities in this set, false otherwise.
///
/// # `to_table() -> Table`
/// Creates and returns a Lua table which can be used to iterate over the
/// abilities in this set.
/// ## Examples
/// ```lua
///  abilities = parent:abilities()
///  table = abilities:to_table()
///  for i = 1, #table do
///    game:log(parent:name() .. " has ability " .. table[i]:name())
///  end
/// ```
///
/// # `can_activate() -> Bool`
/// Returns whether or the parent entity can currently activate at least
/// one ability in this set.  See `ScriptAbility#can_activate`
///
/// # `remove_kind(kind: String) -> ScriptAbilitySet`
/// Creates a new ScriptAbilitySet from this one, but with all abilities
/// with the specified AI Kind `kind` removed.  The kind is specified in the ability
/// definition.  Does not modify this set. Valid kinds are `Damage`, `Heal`, `Buff`,
/// `Debuff`, `Summon`, `Special`
///
/// ## Examples
/// ```lua
///   abilities = parent:abilities()
///   abilities_without_special = abilities:remove_kind("special")
/// ```
///
/// # `only_kind(kind: String) -> ScriptAbilitySet`
/// Creates a new ScriptAbilitySet from this one, but only including abilities
/// with the specified AI Kind `kind`.  Valid kinds are `Damage`, `Heal`, `Buff`,
/// `Debuff`, `Summon`, `Special`
///
/// # `only_group(group: String) -> ScriptAbilitySet`
/// Creates a new ScriptAbilitySet from this one, but only including abilities
/// with the specified AI group `group`. Valid group types are `Single` and `Multiple`.
///
/// # `only_range(range: String) -> ScriptAbilitySet`
/// Creates a new ScriptAbilitySet from this one, but only including abilities
/// with the specified AI range `range`.  Valid range types are `Personal`, `Touch`, `Attack`,
/// `Short`, `Visible`
///
/// # `only_target(target: String) -> ScriptAbilitySet`
/// Creates a new ScriptAbilitySet from this one, but only including abilities with the specified
/// AI target `target`.  Valid target types are `Entity`, `EmptyGround`, `AnyGround`.
///
/// # `sort_by_priority()`
/// Sorts this set in place, according to the AI priority of the abilities in the
/// set.  Lower priorities are sorted first.
/// ## Examples
/// ```lua
///   abilities = parent:abilities():only_range("Touch"):only_kind("Attack")
///   if abilities:is_empty() return end
///   abilities:sort_by_priority()
///   -- do something with the first ability
/// ```
#[derive(Clone)]
pub struct ScriptAbilitySet {
    pub parent: usize,
    pub abilities: Vec<ScriptAbility>,
}

impl ScriptAbilitySet {
    pub fn from(entity: &Rc<RefCell<EntityState>>) -> ScriptAbilitySet {
        let parent = entity.borrow().index();
        let mut abilities = Vec::new();
        for (id, _) in entity.borrow().actor.ability_states.iter() {
            let ability = Module::ability(id).unwrap();
            abilities.push(ScriptAbility::from(&ability));
        }

        ScriptAbilitySet { parent, abilities }
    }
}

impl UserData for ScriptAbilitySet {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("len", |_, set, ()| Ok(set.abilities.len()));

        methods.add_method("is_empty", |_, set, ()| Ok(set.abilities.is_empty()));

        methods.add_method("to_table", |_, set, ()| Ok(set.abilities.clone()));

        methods.add_method("can_activate", |_, set, ()| {
            let parent = ScriptEntity::new(set.parent).try_unwrap()?;
            let parent = parent.borrow();
            let abilities = set
                .abilities
                .iter()
                .filter_map(|ability| {
                    if parent.actor.can_activate(&ability.id) {
                        Some(ability.clone())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(ScriptAbilitySet {
                parent: set.parent,
                abilities,
            })
        });

        methods.add_method("remove_kind", |_, set, kind: String| {
            let kind = ability::AIKind::unwrap_from_str(&kind);

            let abilities = set
                .abilities
                .iter()
                .filter_map(|ability| {
                    if ability.ai_data.kind != kind {
                        Some(ability.clone())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(ScriptAbilitySet {
                parent: set.parent,
                abilities,
            })
        });

        methods.add_method("only_kind", |_, set, kind: String| {
            let kind = ability::AIKind::unwrap_from_str(&kind);

            let abilities = set
                .abilities
                .iter()
                .filter_map(|ability| {
                    if ability.ai_data.kind == kind {
                        Some(ability.clone())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(ScriptAbilitySet {
                parent: set.parent,
                abilities,
            })
        });

        methods.add_method("only_group", |_, set, group: String| {
            let group = ability::AIGroup::unwrap_from_str(&group);
            let abilities = set
                .abilities
                .iter()
                .filter_map(|ability| {
                    if ability.ai_data.group == group {
                        Some(ability.clone())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(ScriptAbilitySet {
                parent: set.parent,
                abilities,
            })
        });

        methods.add_method("only_range", |_, set, range: String| {
            let range = ability::AIRange::unwrap_from_str(&range);
            let abilities = set
                .abilities
                .iter()
                .filter_map(|ability| {
                    if ability.ai_data.range == range {
                        Some(ability.clone())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(ScriptAbilitySet {
                parent: set.parent,
                abilities,
            })
        });

        methods.add_method("only_target", |_, set, target: String| {
            let target = ability::AITarget::unwrap_from_str(&target);
            let abilities = set.abilities.iter().
                filter_map(|ability| {
                    if ability.ai_data.target == target {
                        Some(ability.clone())
                    } else {
                        None
                    }
                }).collect();
            Ok(ScriptAbilitySet {
                parent: set.parent,
                abilities,
            })
        });

        methods.add_method_mut("sort_by_priority", |_, set, ()| {
            set.abilities.sort_by_key(|a| a.ai_data.priority());
            Ok(())
        });
    }
}

/// Represents a specific active ability.  This is passed into ability
/// scripts in the `ability` field, and can also be obtained by iterating
/// over a `ScriptEntitySet`
///
/// # `id() -> String`
/// Returns the unique ID of this ability.
///
/// # `is_active_mode(target: ScriptEntity) -> Bool`
/// Returns true if this ability is a mode that is currently active on the `target`,
/// false otherwise.
///
/// # `activate(target: ScriptEntity, take_ap: Bool (Optional)`
/// Activates this ability for the target.  This will remove AP on the target, if
/// take_ap is not specified or specified and true.
///
/// # `deactivate(target: ScriptEntity)`
/// Deactivates this ability, a currently active mode, on the specified `target`.
/// Normally, you will verify that this is an active mode with `is_active_mode` before
/// calling this method.
///
/// # `cooldown(target: ScriptEntity, round: Int)`
/// Sets the active cooldown for this ability without actually activating it.  This
/// prevents the parent from using the ability for the specified number of rounds.
///
/// # `name() -> String`
/// Returns the name of this ability as defined in its resource file.
///
/// # `duration() -> Int`
/// Returns the duration, in rounds of this ability as defined in its resource file.
/// How this duration is used is up to the ability's script.
///
/// # `create_callback(parent: ScriptEntity) -> ScriptCallback`
/// Creates a script callback from this ability for the `parent`.  Methods
/// can then be added to the ScriptCallback, which are called when certain conditions
/// are met.  These methods will be called from this ability's script, as defined in
/// its resource file.
///
/// # `range() -> Float`
/// Returns the range of this ability as defined in its resource file.  Note that this
/// is not the AI helper range, but the range used for drawing the range indicator preview.
/// Does not include any range bonuses from upgrade levels.
/// Valid ranges are None, Personal, Touch, Attack, Radius(float), and Visible
/// Returns 0.0 for values of Personal, Touch, and Attack, as those depend on parent stats.
/// Returns 0.0 for a Range of None.
///
/// # `ai_data() -> Table`
/// Creates a Lua table including the AI data of this ability.  This includes
/// the `priority`, an integer, the `kind`, `group, `range`, and `target`, all Strings.  See
/// `ScriptAbilitySet::only_group`, `ScriptAbilitySet::only_range`,
/// `ScriptAbilitySet::only_kind`.
#[derive(Clone)]
pub struct ScriptAbility {
    pub id: String,
    name: String,
    duration: u32,
    ap: u32,
    range: Range,
    ai_data: AIData,
}

impl ScriptAbility {
    pub fn from(ability: &Rc<Ability>) -> ScriptAbility {
        let (duration, ai_data) = match ability.active {
            None => {
                error!(
                    "Attempted to get ScriptAbility for non-active '{}'",
                    ability.id
                );
                unreachable!();
            }
            Some(ref active) => {
                let duration = match active.duration {
                    ability::Duration::Rounds(rounds) => rounds,
                    ability::Duration::Mode => 0,
                    ability::Duration::Permanent => 0,
                    ability::Duration::Instant => 0,
                };

                (duration, active.ai.clone())
            }
        };

        let (range, ap) = match ability.active {
            None => (Range::None, 0),
            Some(ref active) => (active.range, active.ap),
        };

        ScriptAbility {
            id: ability.id.to_string(),
            name: ability.name.to_string(),
            duration,
            ap,
            ai_data,
            range,
        }
    }

    pub fn ai_data(&self) -> &AIData {
        &self.ai_data
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
            Some(_) => Ok(()),
        }
    }
}

impl UserData for ScriptAbility {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("id", |_, ability, ()| Ok(ability.id.to_string()));

        methods.add_method("is_active_mode", |_, ability, target: ScriptEntity| {
            ability.error_if_not_active()?;
            let target = target.try_unwrap()?;
            let mut target = target.borrow_mut();
            match target.actor.ability_state(&ability.id) {
                None => Ok(false),
                Some(ref ability_state) => Ok(ability_state.is_active_mode()),
            }
        });
        methods.add_method("activate", activate);
        methods.add_method("deactivate", deactivate);
        methods.add_method(
            "cooldown",
            |_, ability, (target, rounds): (ScriptEntity, u32)| {
                ability.error_if_not_active()?;
                let target = target.try_unwrap()?;
                let mut target = target.borrow_mut();
                match target.actor.ability_state(&ability.id) {
                    None => {
                        warn!("Target does not own specified ability");
                    }
                    Some(ref mut ability_state) => {
                        ability_state.set_cooldown_rounds(rounds);
                    }
                }
                Ok(())
            },
        );
        methods.add_method("name", |_, ability, ()| Ok(ability.name.to_string()));
        methods.add_method("duration", |_, ability, ()| Ok(ability.duration));

        methods.add_method("create_callback", |_, ability, parent: ScriptEntity| {
            ability.error_if_not_active()?;
            let index = parent.try_unwrap_index()?;
            let cb_data = CallbackData::new_ability(index, &ability.id);
            Ok(cb_data)
        });

        methods.add_method("range", |_, ability, ()| {
            Ok(match ability.range {
                Range::None | Range::Touch | Range::Attack | Range::Personal => 0.0,
                Range::Radius(val) => val,
                Range::Visible => {
                    let area = GameState::area_state();
                    let area = area.borrow();
                    area.area.area.vis_dist as f32
                }
            })
        });

        methods.add_method("ai_data", |lua, ability, ()| {
            let ai_data = lua.create_table()?;
            ai_data.set("priority", ability.ai_data.priority())?;
            ai_data.set("kind", ability.ai_data.kind())?;
            ai_data.set("group", ability.ai_data.group())?;
            ai_data.set("range", ability.ai_data.range())?;
            ai_data.set("target", ability.ai_data.target())?;

            if let Some(on_activate_fn) = &ability.ai_data.on_activate_fn {
                ai_data.set("on_activate_fn", on_activate_fn.to_string())?;
            }

            Ok(ai_data)
        });
    }
}

fn deactivate(_lua: Context, ability: &ScriptAbility, target: ScriptEntity) -> Result<()> {
    ability.error_if_not_active()?;
    let target = target.try_unwrap()?;
    target
        .borrow_mut()
        .actor
        .deactivate_ability_state(&ability.id);
    Ok(())
}

fn activate(
    _lua: Context,
    ability: &ScriptAbility,
    (target, take_ap): (ScriptEntity, Option<bool>),
) -> Result<()> {
    ability.error_if_not_active()?;
    let entity = target.try_unwrap()?;
    let take_ap = take_ap.unwrap_or(true);
    let base_ap = ability.ap as i32;

    let ability = ability.to_ability();
    let mgr = GameState::turn_manager();
    if take_ap && mgr.borrow().is_combat_active() {
        let bonus = entity.borrow().actor.stats.bonus_ability_action_point_cost;
        let ap = cmp::max(0, base_ap - bonus);
        entity.borrow_mut().actor.remove_ap(ap as u32);
        entity.borrow_mut().actor.remove_class_stats(&ability);
    }

    let area = GameState::area_state();
    let mut text = AreaFeedbackText::with_target(&entity.borrow(), &area.borrow());
    text.add_entry(ability.name.to_string(), ColorKind::Info);
    area.borrow_mut().add_feedback_text(text);
    area.borrow_mut()
        .range_indicators()
        .remove_ability(&ability);
    entity
        .borrow_mut()
        .actor
        .activate_ability_state(&ability.id);
    Ok(())
}
