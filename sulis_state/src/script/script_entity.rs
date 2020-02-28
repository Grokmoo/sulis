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
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::{self, f32, u32};

use rlua::{self, Context, UserData, UserDataMethods};

use crate::{ability_state::DisabledReason, dist, is_within_attack_dist, is_within_touch_dist};
use crate::{ai, animation, entity_attack_handler, script::*, AreaFeedbackText};
use crate::{area_feedback_text::ColorKind, EntityState, GameState, Location};
use sulis_core::config::Config;
use sulis_core::resource::ResourceSet;
use sulis_core::util::ExtInt;
use sulis_module::{
    ability::AIData, Actor, Attack, AttackKind, Attribute, DamageKind, Faction, HitFlags, HitKind,
    ImageLayer, InventoryBuilder, MOVE_TO_THRESHOLD, area::Destination,
};

/// Represents a single entity for Lua scripts.  Also can represent an invalid,
/// non-existant entity in some cases.  Many script functions pass a parent
/// which is a script entity, and often targets, which is a `ScriptEntitySet`
/// that a ScriptEntity can be extracted from.
///
/// # `state_end() -> AIState`
/// Returns the AI state telling the caller to end the AI turn.
/// ## Examples
/// ```lua
///   function ai_action(parent, state)
///     -- tell ai to end turn immediately
///     return parent:state_end()
///   end
/// ```
///
/// # `state_wait(time: Int) -> AIState`
/// Returns the AI state telling the caller to wait for the specified
/// number of milliseconds (`time`) times the base animation time, and then
/// call the AI again.  See `state_end`
///
/// # `vis_dist() -> Float`
/// Returns the currently visibility distance for this entity (how
/// many tiles on the map it can see).  This is dependant on the
/// area that the entity is in.
///
/// # `add_ability(ability_id: String)`
/// Adds the ability with the specified ID to this entity
///
/// # `remove_ability(ability_id: String)`
/// Removes the ability with the specified ID from this entity
///
/// # `add_levels(class: String, levels: Int)`
/// Adds the specified number of levels of the specified class to this entity
///
/// # `add_xp(amount: Int)`
/// Adds the specified `amount` of XP to the entity.  For adding XP to
/// the party, you generally want to use `game:add_party_xp(amount)`
/// instead.
///
/// # `add_to_party(show_portrait: Bool (Optional))`
/// Adds this entity to the player's party.  `show_portrait` is whether the entity
/// shows up in the portraits area of the UI.  Defaults to true.
///
/// # `set_disabled(disabled: Bool)`
/// Sets this entity to disabled status or not.  Disabled status is only checked when
/// an entity is dead, so this is only useful from the campaign `on_party_death` script.
/// Disabled party members will not be removed from the party, and will be set back to 1
/// hit point when combat ends.  You can use this for custom death behavior in
/// a campaign.
///
/// # `remove_from_party()`
/// Removes this entity from the player's party
///
/// # `get_relationship(other: ScriptEntity) -> Int`
/// Returns a positive 1 if this entity is friendly or neutral to the specified other
/// entity, or a negative 1 if it is hostile.
///
/// # `is_hostile(other: ScriptEntity) -> Bool`
/// Returns true if this entity is hostile to the specified entity, false otherwise
///
/// # `is_friendly(other: ScriptEntity) -> Bool`
/// Returns true if this entity is friendly to the specified entity, false otherwise
///
/// # `get_faction() -> String`
/// Returns the ID of the faction that this entity currently belongs to
///
/// # `set_faction(faction: String)`
/// Sets this entity to the specified `faction`.  Valid factions are currently
/// `Hostile`, `Neutral`, or `Friendly`.  Hostiles will attack the player and
/// friendlies on sight, but will not engage neutrals.
///
/// # `set_flag(flag: String, value: String (Optional))`
/// Sets a `flag` to be stored on this entity.  This value will persist as part of the
/// save game and can be used to store custom state.  If the value is not specified,
/// sets the flag exists (for querying with `has_flag()`), but does not neccessarily
/// set a specific value.
///
/// # `add_num_flag(flag: String, value: Float)`
/// Adds the specified `value` to the amount stored in the specified `flag`.  If the
/// flag is not currently present, sets the flag to the specified value.
///
/// # `get_flag(flag: String) -> String`
/// Returns the value of the specified `flag` on this entity.  Returns the lua
/// value of `Nil` if the flag does not exist.
///
/// # `has_flag(flag: String) -> Bool`
/// Returns true if the specified `flag` is set to any value on this entity, false
/// otherwise
///
/// # `get_num_flag(flag: String) -> Float`
/// Returns the numeric value of this `flag` set on this entity, or 0.0 if it has
/// not been set.
///
/// # `clear_flag(flag: String)`
/// Clears the `flag` from this entity, as if it had never been set.  Works for both
/// numeric and standard flags.  If the flag had not previously been set, does nothing.
/// After this method, `has_flag(flag)` will return `false`.
///
/// # `is_valid() -> Bool`
/// Returns true if this ScriptEntity references a valid entity that can be queried and
/// acted on, false otherwise.
///
/// # `is_dead() -> Bool`
/// Returns true if this entity is dead (zero hit points), false otherwise.  Dead entities
/// cannot be currently interacted with in meaningful ways.
///
/// # `is_party_member() -> Bool`
/// Returns true if this entity is a member of the player's party (or if it is the player),
/// false otherwise.
///
/// # `use_ability(ability: ScriptAbility, allow_invalid: Bool (Optional)) -> Bool`
/// The parent entity attempts to use the `ability`.  Returns true if the ability use was
/// successful, false if it was not.  After activating, the script will often need to handle
/// a targeter (depending on the ability), using the methods on `ScriptInterface` (the `game`
/// object).  If `allow_invalid` is set to true, the ability will fire even if the parent
/// could not normally use it at this time.
///
/// # `use_item(item: ScriptUsableItem) -> Bool`
/// Attempts to use the specified `item`.  Returns true if the item use was successful, false
/// if it was not.  See `use_ability`.
///
/// # `swap_weapons() -> Bool`
/// Attempts to swap weapons from the currently held weapon set to the alternate weapon slots.
/// Returns true if this is succesful, false if it is not.  The entity must have enough AP
/// to complete the action.
///
/// # `abilities() -> ScriptAbilitySet`
/// Returns the ScriptAbilitySet with all the abilities that this entity can potentially activate.
///
/// # `targets() -> ScriptEntitySet`
/// Creates a ScriptEntitySet consisting of all possible targets for abilities or items used
/// by this entity.  This includes all known entities in the same area as the parent entity.
///
/// # `targets_from(targets: Table) -> ScriptEntitySet`
/// Creates a ScriptEntitySet for this parent with the specified array-like table of targets
/// as the targets.  Allows the script complete control over the set of targets
///
/// # `has_effect_with_tag(tag: String) -> Bool`
/// Returns true if this entity has one or more active effects with the specified tag,
/// false otherwise.
///
/// # `get_effects_with_tag(tag: String) -> Table of ScriptAppliedEffect`
/// Returns an array-like table containing all of the effects currently applied to this
/// entity with the specified tag.
///
/// # `get_auras_with_tag(tag: String) -> Table of ScriptAppliedEffect`
/// Returns an array-like table containing all auras owned by this entity with the
/// specified tag.  Note that this includes only owned auras, not auras from another
/// entity that are affected this entity.
///
/// # `remove_effects_with_tag(tag: String)`
/// Removes all currently active effects applied to this entity that have the specified tag.
///
/// # `create_effect(name: String, duration: Int (Optional)) -> ScriptEffect`
/// Creates a new effect with the specified `name` and `duration`.  If `duration` is not
/// specified, it is infinite, and will remain until removed or deactivated for a mode.
/// The effect will not be in effect until you call `apply()` on it.
///
/// # `create_surface(name: String, points: Table, duration: Int (Optional)) -> ScriptEffect`
/// Creates a surface effect with this entity as the parent.  This is a special case of
/// `create_effect`, above.  The effect must have `apply()`
/// called in order to actually be put into effect.  See `ScriptEffect`.
/// The `points` used by this method is a table of tables with `x` and `y` elements.  This
/// can be constructed by hand, or obtained from a `ScriptEntitySet` as the `affected_points`.
///
/// # `create_image_layer_anim(duration: Floag (Optional)) -> ScriptImageLayerAnimation`
/// Creates an image layer animation that will add (or override) image layers of the entity
/// for the specified duraiton.  If `duration` is not specified, the animation lasts forever
/// or until the attached effect is removed.
///
/// # `create_scale_anim(duration: Float (Optional)) -> ScriptScaleAnimation`
/// Creates a scale animation that will change the size of the entity by a
/// factor, for the specified duration.  If `duration` is not specified, the
/// animation lasts forever or until the attached effect is removed.
///
/// # `create_subpos_anim(duration: Float (Optional)) -> ScriptSubposAnimation`
/// Creates an entity subpos animation, that can be used to temporarily move
/// the location of the entity with pixel accuracy on the screen, for the specified
/// `duration` in seconds.  The animation is set up with further calls before
/// calling `activate()`.
///
/// # `create_color_anim(duration: Float (Optional)) -> ScriptColorAnimation`
/// Creates an entity color animation, which changes the primary and secondary
/// colors of the parent entity.  If `duration` is specified, lasts for that many seconds.
/// Otherwise, will last forever, or more typically until the attached effect is removed.
///
/// # `create_particle_generator(image: String, duration: Float (Optional)) ->
/// ScriptParticleGenerator`
/// Creates a Particle Generator animation.  Despite the name, can also be used for more
/// traditional frame based animations by using a single particle (see `create_anim`.
/// If `duration` is specified, lasts for that number of seconds.  Otherwise, will last
/// forever, or more typically until the attached effect is removed.  The specified image
/// must be the ID of a defined image.
///
/// # `create_anim(image: String, duration: Float (Optional)) -> ScriptParticleGenerator`
/// Creates a particle generator animation set up for a single particle frame based
/// animation.  The `image` should normally be the ID of a timer image with specified frames.
/// The `duration` is in seconds, or not specified to make the animation repeat until
/// the parent effect is removed (if there is one).  The anim must have `activate()` called
/// once setup is complete.
///
/// # `create_targeter(ability: ScriptAbility) -> TargeterData`
/// Creates a new targeter for the specified ability.  The ability's script will be used for
/// all functions.  This targeter can then be configured
/// before calling `activate()` to put it into effect.  Upon the user or ai script selecting
/// a target, `on_target_select` is called.
///
/// # `create_targeter_for_item(item: ScriptItem) -> TargeterData`
/// Creates a new targeter for the specified item.  The item's script will be used for all
/// functions.  The targeter can then be configured before calling `activate()`.  See
/// `create_targeter` above.
///
/// # `move_towards_entity(target: ScriptEntity, distance: Float (Optional), max_len: Int
/// (Optional)) -> Bool`
/// Causes this entity to attempt to begin moving towards the specified `target`.  If this
/// entity cannot move at all towards the desired target, returns false, otherwise, returns
/// true and creates a move animation that will proceed to be run asynchronously.
/// Optionally, a `distance` can be specified which is the distance this entity should be
/// within the target to complete the move.  If no distance is specified, the entity
/// attempts to move within attack range.  Can optionally specify a maximum path distance.
///
/// # `move_towards_point(x: Float, y: Float, distance: Float (Optional)) -> Bool`
/// Causes this entity to attempt to begin moving towards the specified point at
/// `x` and `y`.  If `distance` is specified, attempts to move within that distance
/// of the point.  Otherwise, attempts to move so the parent entity's coordinates
/// are equal to the nearest integers to `x` and `y`.  If the entity cannot move at
/// all or a path cannot be found, this returns false.  Otherwise, returns true and
/// an asynchronous move animation is initiated.
///
/// # `dist_to_entity(target: ScriptEntity) -> Float`
/// Computes the current euclidean distance to the specified `target`, in tiles.
/// This should not be used for targeting purposes.  Use the ScriptEntitySet's
/// filtering methods instead.
///
/// # `dist_to_point(point: Table) -> Float`
/// Computes the euclidean distance to the specified `point`, in tiles.  Point is
/// a table of the form `{x: x_coord, y: y_coord}`
///
/// # `has_ap_to_attack() -> Bool`
/// Returns true if this entity has enough AP to issue a single attack, false otherwise.
///
/// # `is_within_touch_dist(target: ScriptEntity) -> Bool`
/// Returns whether this entity is close enough to touch the specified target.
///
/// # `is_within_attack_dist(target: ScriptEntity) -> Bool`
/// Returns whether this entity is close enough to attack the target with
/// its current weapon.
///
/// # `has_visibility(target: ScriptEntity) -> Bool`
/// Returns true if this entity can see the `target`, false otherwise.
///
/// # `can_move() -> Bool`
/// Returns true if this entity can move at all (even 1 square), false otherwise.
///
/// # `teleport_to(dest: Table)`
/// Instantly moves this entity to the `dest`, which is a table of the form
/// `{ x: x_coord, y: y_coord }`.  Will not move the entity if the dest
/// position is invalid (outside area bounds, impassable).
///
/// # `weapon_attack(target: ScriptEntity) -> ScriptHitKind`
/// Immediately rolls a random attack against the specified `target`, using this
/// entities stats vs the defender. Returns the hit type, one of crit, hit,
/// graze, or miss.
///
/// # 'anim_weapon_attack(target: ScriptEntity, callback: CallbackData (Optional),
/// use_ap: Bool (Optional))`
/// Attempts to perform a standard weapon attack against the `target`.  The attack
/// is animated, so this method immediately returns but the attack happens
/// asynchronously.  Upon completion of the attack, the `callback` (if specified)
/// is run.  If `use_ap` is specified to false, no ap is deducted from the parent
/// for the attack.  By default, the standard amount of ap is deducted.
///
/// # `special_attack(target: ScriptEntity, attack_kind: String, accuracy_kind: String,
/// min_damage: Float, max_damage: Float, ap_damage: Float, damage_kind: String)`
/// Immediately rolls a random non-standard attack against the `target`, using the specified
/// parameters.  See `anim_special_attack`.
///
/// # `anim_special_attack(target: ScriptEntity, attack_kind: String, accuracy_kind: String,
/// min_damage: Float, max_damage: Float, ap_damage: Float, damage_kind: String,
/// callback: CallbackData (Optional))`
/// Animates a non standard attack against the `target` with the specified parameters.
/// AttackKind is one of `Melee`, `Ranged`, or `Spell`, and determines which of the attackers
/// attack types to use.  AccuracyKind is one of `Fortitude`, `Reflex`, `Will`, or `Dummy`
/// and determines which of the defenders defense stats to use.
/// The amount of damage is rolled randomly, between the `min_damage` and `max_damage`, with
/// the specified (`ap_damage`) amount of armor piercing.  This damage is then compared
/// against the defender's armor as normal.
/// If specified, the callback is called after the animation completes.  No ap is deducted
/// for this attack.
///
/// # `remove()`
/// Sets this entity to be removed (as if dead) on the next frame update.  This method
/// is called asynchronously, so the entity will not yet be removed immediately after
/// this method.
///
/// # `take_damage(attacker: ScriptEntity, min_damage: Float, max_damage: Float,
/// damage_kind: String, ap: Int (Optional))`
/// Causes this entity to take the specified amount of damage.  Hit points are removed,
/// based on this entity's armor.  The damage is rolled randomly between `min_damage` and
/// `max_damage`, with the specified (`ap`) amount of armor piercing.
///
/// # `heal_damage(amount: Float)`
/// Adds the specified number of hit points to this entity.  The entity's maximum hit
/// points cannot be exceeded in this way.
///
/// # `add_class_stat(stat: String, amount: Float)`
/// Adds the specified amount of the specified stat for this entity.  The entity's maximum
/// class stat cannot be exceeded.
///
/// # `remove_class_stat(stat: String, amount: Float)`
/// Removes the specified amount of the class stat for this entity.
///
/// # `get_overflow_ap() -> Int`
/// Returns the current amount of overflow ap for this entity.  This is AP that will become
/// available as bonus AP (up to the maximum per round AP) on this entity's next turn.
///
/// # `change_overflow_ap(ap: Int)`
/// Modifies the amount of available overflow ap for this entity.  See `get_overflow_ap`.
///
/// # `set_subpos(x: Float, y: Float)`
/// Sets the pixel precise position of this entity to the specified value.  An entity should
/// generally not be left with non-zero values for either `x` or `y`.
///
/// # `add_ap(amount: Int)`
/// Adds the specified `amount` of AP to this entity.  Keep in mind the `display_ap`
/// factor that this amount is divided by for display purposes.
///
/// # `remove_ap(amount: Int)`
/// Removes the specified `amount` of AP from this entity.  Keep in mind the `display_ap`
/// factor that this amount is divided by for display purposes.
///
/// # `base_class() -> String`
/// Returns the ID of the base class of this entity, or the class that this entity took at
/// level 1.
///
/// # `id() -> String`
/// Returns the ID of this entity.  This should be unique, but it is currently possible to have
/// more than one entity with the same ID (the game does provide a warning in this case).
///
/// # `name() -> String`
/// Returns the name of this entity.
///
/// # `has_ability(ability_id: String) -> Bool`
/// Returns true if this entity possesses the ability with the specified `ability_id`, false
/// otherwise.
///
/// # `get_abilities_with_group(group_id: String) -> Table<ScriptAbility>`
/// Returns an array table with all the active abilities owned by this entity with
/// the specified ability group.
///
/// # `get_ability(ability_id: String) -> ScriptAbility`
/// Returns a `ScriptAbility` representing the ability with the specified `ability_id`.  Throws
/// an error if this entity does not possess the ability.
///
/// # `ability_level(ability: ScriptAbility) -> Int`
/// Returns the level of the specified `ability` for this entity.  This is zero if the entity
/// does not possess the ability, one if it possesses just the base ability, and larger numbers
/// depending on the number of upgrades possessed.
///
/// # `ability_level_from_id(ability_id: String) -> Int`
/// Returns the level of the ability with the specified ID.  See `ability_level`
///
/// # `has_active_mode() -> Bool`
/// Returns true if this entity has at least one currently active mode ability, false
/// otherwise.
///
/// # `get_active_mode() -> Bool`
/// Returns the first active mode for this entity, if one exists.
///
/// # `stats() -> Table`
/// Creates and returns a stats table for this entity.  This includes all stats shown on the
/// character sheet.
///
/// # `inventory() -> ScriptInventory`
/// Returns a `ScriptInventory` object representing this entity's inventory.
///
/// # `race() -> String`
/// Returns the ID of the race of this entity
///
/// # `image_layer_offset(layer: String) -> Table`
/// Gets the image layer offset, in tiles for the given image layer
/// for this entity.  The table has members `x` and `y` with the offset value.
/// The layer must be a valid ImageLayer, one of HeldMain, HeldOff, Ears, Hair,
/// Beard, Head, Hands, Foreground, Torso, Legs, Feet, Background, Cloak, Shadow
///
/// # `size_str() -> String`
/// Returns the ID of the size of this entity, i.e. 2by2 or 3by3.
///
/// # `location() -> Table`
/// Returns a table with 'x', 'y', and 'area' entries for the location of this
/// entity.  This is more efficient than calling individual methods for each
/// component.
///
/// # `area() -> String`
/// Returns the ID of the area that this entity is currently located in
///
/// # `width() -> Int`
/// Returns the width of this entity in tiles
///
/// # `height() -> Int`
/// Returns the height of this entity in tiles
///
/// # `x() -> Int`
/// Returns the x coordinate of this entity's position in tiles
///
/// # `y() -> Int`
/// Returns the y coordinate of this entity's position in tiles
///
/// # `center_x() -> Float`
/// Returns the position of this entity's center (x + width / 2) as a float.
///
/// # `center_y() -> Float`
/// Returns the position of this entity's center (y + height / 2) as a float.
///
/// # `is_threatened() -> Bool`
/// Returns whether or not this entity is currently threatened by a hostile
/// with a melee weapon
///
/// # `is_threatened_by(target: ScriptEntity) -> Bool`
/// Returns true if this entity is threatened by the speciied target with its
/// melee weapon, false otherwise
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
        ScriptEntity {
            index: Some(entity.borrow().index()),
        }
    }

    pub fn is_party_member(&self) -> bool {
        let entity = match self.try_unwrap() {
            Ok(entity) => entity,
            Err(_) => return false,
        };
        let entity = entity.borrow();
        entity.is_party_member()
    }

    pub fn check_not_equal(&self, other: &ScriptEntity) -> Result<()> {
        if self.index == other.index {
            warn!("Parent and target must not refer to the same entity for this method");
            Err(rlua::Error::FromLuaConversionError {
                from: "ScriptEntity",
                to: "ScriptEntity",
                message: Some("Parent and target must not match".to_string()),
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
                message: Some("ScriptEntity does not have a valid index".to_string()),
            }),
            Some(index) => Ok(index),
        }
    }

    pub fn try_unwrap(&self) -> Result<Rc<RefCell<EntityState>>> {
        match self.index {
            None => Err(rlua::Error::FromLuaConversionError {
                from: "ScriptEntity",
                to: "EntityState",
                message: Some("ScriptEntity does not have a valid index".to_string()),
            }),
            Some(index) => {
                let mgr = GameState::turn_manager();
                let mgr = mgr.borrow();
                match mgr.entity_checked(index) {
                    None => Err(rlua::Error::FromLuaConversionError {
                        from: "ScriptEntity",
                        to: "EntityState",
                        message: Some(
                            "ScriptEntity refers to an entity that no longer exists.".to_string(),
                        ),
                    }),
                    Some(entity) => Ok(entity),
                }
            }
        }
    }
}

impl UserData for ScriptEntity {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("state_end", |_, _, ()| Ok(ai::State::End));

        methods.add_method("state_wait", |_, _, time: u32| Ok(ai::State::Wait(time)));

        methods.add_method("vis_dist", |_, entity, ()| {
            let parent = entity.try_unwrap()?;
            let area_id = &parent.borrow().location.area_id;
            let area = GameState::get_area_state(area_id).unwrap();
            let dist = area.borrow().area.area.vis_dist as f32;
            Ok(dist)
        });

        methods.add_method("add_xp", |_, entity, amount: u32| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().actor.add_xp(amount);
            Ok(())
        });

        methods.add_method("remove_ability", |_, entity, ability: String| {
            let entity = entity.try_unwrap()?;

            let actor = {
                let old_actor = &entity.borrow().actor.actor;
                let xp = entity.borrow().actor.xp();
                Actor::from(
                    old_actor,
                    None,
                    xp,
                    Vec::new(),
                    vec![ability],
                    InventoryBuilder::default(),
                )
            };

            entity.borrow_mut().actor.replace_actor(actor);

            Ok(())
        });

        methods.add_method("add_ability", |_, entity, ability: String| {
            let entity = entity.try_unwrap()?;

            let ability = match Module::ability(&ability) {
                None => {
                    warn!("Invalid ability '{}' in script", ability);
                    return Ok(());
                }
                Some(ability) => ability,
            };

            let actor = {
                let old_actor = &entity.borrow().actor.actor;
                let xp = entity.borrow().actor.xp();
                Actor::from(
                    old_actor,
                    None,
                    xp,
                    vec![ability],
                    Vec::new(),
                    InventoryBuilder::default(),
                )
            };

            entity.borrow_mut().actor.replace_actor(actor);
            Ok(())
        });

        methods.add_method("add_levels", |_, entity, (class, levels): (String, u32)| {
            let entity = entity.try_unwrap()?;

            let class = match Module::class(&class) {
                None => {
                    warn!("Invalid class '{}' in script", class);
                    return Ok(());
                }
                Some(class) => class,
            };

            let actor = {
                let old_actor = &entity.borrow().actor.actor;
                let xp = entity.borrow().actor.xp();
                Actor::from(
                    old_actor,
                    Some((class, levels)),
                    xp,
                    Vec::new(),
                    Vec::new(),
                    InventoryBuilder::default(),
                )
            };

            entity.borrow_mut().actor.replace_actor(actor);
            entity.borrow_mut().actor.init_day();

            Ok(())
        });

        methods.add_method("set_disabled", |_, entity, disabled: bool| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().actor.set_disabled(disabled);
            Ok(())
        });

        methods.add_method("add_to_party", |_, entity, show_portrait: Option<bool>| {
            let entity = entity.try_unwrap()?;
            GameState::add_party_member(entity, show_portrait.unwrap_or(true));
            Ok(())
        });

        methods.add_method("remove_from_party", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            GameState::remove_party_member(entity);
            Ok(())
        });

        methods.add_method("get_relationship", |_, entity, other: ScriptEntity| {
            let entity = entity.try_unwrap()?;
            let other = other.try_unwrap()?;

            let result = if entity.borrow().is_hostile(&other.borrow()) {
                -1
            } else {
                1
            };

            Ok(result)
        });

        methods.add_method("is_hostile", |_, entity, other: ScriptEntity| {
            let entity = entity.try_unwrap()?;
            let other = other.try_unwrap()?;
            let result = entity.borrow().is_hostile(&other.borrow());
            Ok(result)
        });

        methods.add_method("is_friendly", |_, entity, other: ScriptEntity| {
            let entity = entity.try_unwrap()?;
            let other = other.try_unwrap()?;
            let result = entity.borrow().is_friendly(&other.borrow());
            Ok(result)
        });

        methods.add_method("get_faction", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.actor.faction().to_str())
        });

        methods.add_method("set_faction", |_, entity, faction: String| {
            let entity = entity.try_unwrap()?;

            match Faction::option_from_str(&faction) {
                None => warn!("Invalid faction '{}' in script", faction),
                Some(faction) => entity.borrow_mut().actor.set_faction(faction),
            }

            let mgr = GameState::turn_manager();
            let area_state = GameState::area_state();

            mgr.borrow_mut()
                .check_ai_activation(&entity, &mut area_state.borrow_mut());

            Ok(())
        });

        methods.add_method("get_num_flag", |_, entity, flag: String| {
            let entity = entity.try_unwrap()?;
            let val = entity.borrow().get_num_flag(&flag);
            Ok(val)
        });

        methods.add_method("add_num_flag", |_, entity, (flag, val): (String, f32)| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().add_num_flag(&flag, val);
            Ok(())
        });

        methods.add_method(
            "set_flag",
            |_, entity, (flag, val): (String, Option<String>)| {
                let entity = entity.try_unwrap()?;
                let val = match &val {
                    None => "true",
                    Some(val) => val,
                };

                entity.borrow_mut().set_custom_flag(&flag, val);
                Ok(())
            },
        );

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

        methods.add_method("is_dead", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let result = entity.borrow().actor.is_dead();
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

        methods.add_method(
            "use_ability",
            |_, entity, (ability, allow_invalid): (ScriptAbility, Option<bool>)| {
                let allow_invalid = allow_invalid.unwrap_or(false);

                let parent = entity.try_unwrap()?;
                if !allow_invalid {
                    let can_toggle = parent.borrow().actor.can_toggle(&ability.id);
                    if can_toggle != DisabledReason::Enabled {
                        return Ok(false);
                    }
                }
                let index = parent.borrow().index();
                let func = get_on_activate_fn(parent.borrow().is_party_member(), ability.ai_data());
                Script::ability_on_activate(index, func, &ability.to_ability());
                Ok(true)
            },
        );

        methods.add_method("use_item", |_, entity, item: ScriptUsableItem| {
            let slot = item.slot;
            let parent = entity.try_unwrap()?;
            if !parent.borrow().actor.can_use_quick(slot) {
                return Ok(false);
            }
            let func = get_on_activate_fn(parent.borrow().is_party_member(), item.ai_data());
            Script::item_on_activate(&parent, func, ScriptItemKind::Quick(slot));
            Ok(true)
        });

        methods.add_method("swap_weapons", |_, entity, ()| {
            let parent = entity.try_unwrap()?;
            if !parent.borrow().actor.can_swap_weapons() {
                return Ok(false);
            }

            EntityState::swap_weapon_set(&parent);
            Ok(true)
        });

        methods.add_method("abilities", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            Ok(ScriptAbilitySet::from(&entity))
        });

        methods.add_method("targets", &targets);

        methods.add_method("targets_from", |_, entity, targets: Vec<ScriptEntity>| {
            let parent = entity.try_unwrap_index()?;
            let indices = targets.into_iter().map(|target| target.index).collect();
            let targets = ScriptEntitySet {
                parent,
                selected_point: None,
                affected_points: Vec::new(),
                indices,
                surface: None,
            };
            Ok(targets)
        });

        methods.add_method("get_effects_with_tag", |_, entity, tag: String| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            let mgr = GameState::turn_manager();
            let mgr = mgr.borrow();

            let mut result = Vec::new();
            for effect_index in entity.actor.effects_iter() {
                let effect = mgr.effect(*effect_index);
                if effect.tag != tag {
                    continue;
                }

                let sae = ScriptAppliedEffect::new(effect, *effect_index);
                result.push(sae);
            }

            Ok(result)
        });

        methods.add_method("get_auras_with_tag", |_, entity, tag: String| {
            let entity_index = entity.try_unwrap_index()?;
            let mgr = GameState::turn_manager();
            let mgr = mgr.borrow();

            let indices = mgr.auras_for(entity_index);

            let mut result = Vec::new();
            for effect_index in indices {
                let effect = mgr.effect(effect_index);
                if effect.tag != tag {
                    continue;
                }
                let sae = ScriptAppliedEffect::new(effect, effect_index);
                result.push(sae);
            }
            Ok(result)
        });

        methods.add_method("has_effect_with_tag", |_, entity, tag: String| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            let mgr = GameState::turn_manager();
            let mgr = mgr.borrow();

            for effect_index in entity.actor.effects_iter() {
                let effect = mgr.effect(*effect_index);
                if effect.tag == tag {
                    return Ok(true);
                }
            }

            Ok(false)
        });

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

        methods.add_method(
            "create_surface",
            |_, _, (name, points, duration): (String, Vec<HashMap<String, i32>>, Option<u32>)| {
                let duration = match duration {
                    None => ExtInt::Infinity,
                    Some(dur) => ExtInt::Int(dur),
                };
                let points: Vec<(i32, i32)> = points
                    .into_iter()
                    .map(|p| {
                        let x = p.get("x").unwrap();
                        let y = p.get("y").unwrap();
                        (*x, *y)
                    })
                    .collect();
                Ok(ScriptEffect::new_surface(points, &name, duration))
            },
        );

        methods.add_method("create_effect", |_, entity, args: (String, Option<u32>)| {
            let duration = match args.1 {
                None => ExtInt::Infinity,
                Some(dur) => ExtInt::Int(dur),
            };
            let ability = args.0;
            let index = entity.try_unwrap_index()?;
            Ok(ScriptEffect::new_entity(index, &ability, duration))
        });

        methods.add_method(
            "create_image_layer_anim",
            |_, entity, duration_secs: Option<f32>| {
                let index = entity.try_unwrap_index()?;
                let duration = match duration_secs {
                    None => ExtInt::Infinity,
                    Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
                };

                Ok(ScriptImageLayerAnimation::new(index, duration))
            },
        );

        methods.add_method(
            "create_scale_anim",
            |_, entity, duration_secs: Option<f32>| {
                let index = entity.try_unwrap_index()?;
                let duration = match duration_secs {
                    None => ExtInt::Infinity,
                    Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
                };

                Ok(ScriptScaleAnimation::new(index, duration))
            },
        );

        methods.add_method(
            "create_subpos_anim",
            |_, entity, duration_secs: Option<f32>| {
                let index = entity.try_unwrap_index()?;
                let duration = match duration_secs {
                    None => ExtInt::Infinity,
                    Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
                };
                Ok(ScriptSubposAnimation::new(index, duration))
            },
        );

        methods.add_method(
            "create_color_anim",
            |_, entity, duration_secs: Option<f32>| {
                let index = entity.try_unwrap_index()?;
                let duration = match duration_secs {
                    None => ExtInt::Infinity,
                    Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
                };
                Ok(ScriptColorAnimation::new(index, duration))
            },
        );

        methods.add_method(
            "create_particle_generator",
            |_, entity, args: (String, Option<f32>)| {
                let sprite = args.0;
                let index = entity.try_unwrap_index()?;
                let duration = match args.1 {
                    None => ExtInt::Infinity,
                    Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
                };
                Ok(ScriptParticleGenerator::new(index, sprite, duration))
            },
        );

        methods.add_method("wait_anim", |_, entity, duration: f32| {
            let index = entity.try_unwrap_index()?;
            let image = ResourceSet::empty_image();
            let duration = ExtInt::Int((duration * 1000.0) as u32);
            Ok(ScriptParticleGenerator::new_anim(
                index,
                image.id(),
                duration,
            ))
        });

        methods.add_method(
            "create_anim",
            |_, entity, (image, duration): (String, Option<f32>)| {
                let duration = match duration {
                    None => ExtInt::Infinity,
                    Some(amount) => ExtInt::Int((amount * 1000.0) as u32),
                };
                let index = entity.try_unwrap_index()?;
                Ok(ScriptParticleGenerator::new_anim(index, image, duration))
            },
        );

        methods.add_method("create_targeter", |_, entity, ability: ScriptAbility| {
            let index = entity.try_unwrap_index()?;
            Ok(TargeterData::new_ability(index, &ability.id))
        });

        methods.add_method("create_targeter_for_item", |_, entity, item: ScriptItem| {
            let index = entity.try_unwrap_index()?;
            Ok(TargeterData::new_item(index, item.kind()))
        });

        methods.add_method(
            "move_towards_entity",
            |_, entity, (dest, dist, max_len): (ScriptEntity, Option<f32>, Option<u32>)| {
                let parent = entity.try_unwrap()?;
                let target = dest.try_unwrap()?;

                let mut dest = GameState::get_target_dest(&*parent.borrow(), &*target.borrow());
                if let Some(dist) = dist {
                    dest.dist = dist;
                }

                dest.max_path_len = max_len;

                move_towards_dest(parent, dest)
            },
        );

        methods.add_method(
            "move_towards_point",
            |_, entity, (x, y, dist): (f32, f32, Option<f32>)| {
                let parent = entity.try_unwrap()?;

                let mut dest = GameState::get_point_dest(&*parent.borrow(), x, y);
                dest.dist = dist.unwrap_or(MOVE_TO_THRESHOLD);

                move_towards_dest(parent, dest)
            },
        );

        methods.add_method("has_ap_to_attack", |_, entity, ()| {
            let parent = entity.try_unwrap()?;
            let result = parent.borrow().actor.has_ap_to_attack();
            if parent.borrow().actor.stats.attack_disabled {
                return Ok(false);
            }
            Ok(result)
        });

        methods.add_method(
            "is_within_attack_dist",
            |_, entity, target: ScriptEntity| {
                let parent = entity.try_unwrap()?;
                let target = target.try_unwrap()?;
                let result = is_within_attack_dist(&*parent.borrow(), &*target.borrow());
                Ok(result)
            },
        );

        methods.add_method("is_within_touch_dist", |_, entity, target: ScriptEntity| {
            let parent = entity.try_unwrap()?;
            let target = target.try_unwrap()?;
            let result = is_within_touch_dist(&*parent.borrow(), &*target.borrow());
            Ok(result)
        });

        methods.add_method("has_visibility", |_, entity, target: ScriptEntity| {
            let parent = entity.try_unwrap()?;
            let target = target.try_unwrap()?;
            let area_state = GameState::area_state();
            let area_state = area_state.borrow();
            let result = area_state.has_visibility(&parent.borrow(), &target.borrow());
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
            let entity_index = entity.borrow().index();
            let mgr = GameState::turn_manager();

            let area_state = GameState::area_state();
            if !entity.borrow().location.is_in(&area_state.borrow()) {
                let old_area_state =
                    GameState::get_area_state(&entity.borrow().location.area_id).unwrap();

                let surfaces = old_area_state
                    .borrow_mut()
                    .remove_entity(&entity, &mgr.borrow());
                for surface in surfaces {
                    mgr.borrow_mut().remove_from_surface(entity_index, surface);
                }

                let new_loc = Location::new(x, y, &area_state.borrow().area.area);
                if let Err(e) =
                    area_state
                        .borrow_mut()
                        .transition_entity_to(&entity, entity_index, new_loc)
                {
                    warn!("Unable to move entity using script function");
                    warn!("{}", e);
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
            let area_state = GameState::area_state();
            let result = entity_attack_handler::weapon_attack(&parent, &target);

            let mut total_hit_kind = HitKind::Miss;
            let mut total_damage = Vec::new();
            for entry in result {
                let (hit_kind, hit_flags, damage) = entry;

                if hit_kind > total_hit_kind {
                    total_hit_kind = hit_kind;
                }

                total_damage.append(&mut damage.clone());

                let feedback = AreaFeedbackText::with_damage(
                    &target.borrow(),
                    &area_state.borrow(),
                    hit_kind,
                    hit_flags,
                    &damage,
                );
                area_state.borrow_mut().add_feedback_text(feedback);
            }

            let hit_kind = ScriptHitKind::new(total_hit_kind, total_damage);
            Ok(hit_kind)
        });

        methods.add_method("anim_weapon_attack", |_, entity, (target, callback, use_ap):
                           (ScriptEntity, Option<CallbackData>, Option<bool>)| {
            entity.check_not_equal(&target)?;
            let parent = entity.try_unwrap()?;
            let target = target.try_unwrap()?;

            let cb: Option<Box<dyn ScriptCallback>> = match callback {
                None => None,
                Some(cb) => Some(Box::new(cb)),
            };

            let use_ap = use_ap.unwrap_or(false);

            EntityState::attack(&parent, &target, cb, use_ap);
            Ok(())
        });

        methods.add_method("anim_special_attack", |_, entity,
            (target, attack_kind, accuracy_kind, min_damage, max_damage, ap, damage_kind, cb):
            (ScriptEntity, String, String, f32, f32, f32, String, Option<CallbackData>)| {

            let min_damage = min_damage as u32;
            let max_damage = max_damage as u32;
            let ap = ap as u32;

            entity.check_not_equal(&target)?;
            let parent = entity.try_unwrap()?;
            let target = target.try_unwrap()?;
            let damage_kind = DamageKind::unwrap_from_str(&damage_kind);
            let attack_kind = AttackKind::from_str(&attack_kind, &accuracy_kind);
            let mut cbs: Vec<Box<dyn ScriptCallback>> = Vec::new();
            if let Some(cb) = cb {
                cbs.push(Box::new(cb));
            }
            let time = Config::animation_base_time_millis() * 5;
            let anim = animation::melee_attack_animation::new(&Rc::clone(&parent), &target,
                                                              time, cbs, Box::new(move |att, def| {
                let mut attack = Attack::special(&parent.borrow().actor.stats,
                    min_damage, max_damage, ap, damage_kind, attack_kind.clone());

                vec![entity_attack_handler::attack(att, def, &mut attack)]
            }));

            GameState::add_animation(anim);
            Ok(())
        });

        #[allow(clippy::type_complexity)]
        methods.add_method(
            "special_attack",
            |_,
             entity,
             (target, attack_kind, accuracy_kind, min_damage, max_damage, ap, damage_kind): (
                ScriptEntity,
                String,
                String,
                Option<f32>,
                Option<f32>,
                Option<f32>,
                Option<String>,
            )| {
                let target = target.try_unwrap()?;
                let parent = entity.try_unwrap()?;

                let damage_kind = match damage_kind {
                    None => DamageKind::Raw,
                    Some(ref kind) => DamageKind::unwrap_from_str(kind),
                };
                let attack_kind = AttackKind::from_str(&attack_kind, &accuracy_kind);

                let min_damage = min_damage.unwrap_or(0.0) as u32;
                let max_damage = max_damage.unwrap_or(0.0) as u32;
                let ap = ap.unwrap_or(0.0) as u32;

                let mut attack = Attack::special(
                    &parent.borrow().actor.stats,
                    min_damage,
                    max_damage,
                    ap,
                    damage_kind,
                    attack_kind,
                );

                let (hit_kind, hit_flags, damage) =
                    entity_attack_handler::attack(&parent, &target, &mut attack);

                let area_state = GameState::area_state();

                let feedback = AreaFeedbackText::with_damage(
                    &target.borrow(),
                    &area_state.borrow(),
                    hit_kind,
                    hit_flags,
                    &damage,
                );
                area_state.borrow_mut().add_feedback_text(feedback);
                let hit_kind = ScriptHitKind::new(hit_kind, damage);
                Ok(hit_kind)
            },
        );

        methods.add_method("remove", |_, entity, ()| {
            let parent = entity.try_unwrap()?;
            parent.borrow_mut().marked_for_removal = true;
            Ok(())
        });

        methods.add_method(
            "take_damage",
            |_,
             entity,
             (attacker, min_damage, max_damage, damage_kind, ap): (
                ScriptEntity,
                f32,
                f32,
                String,
                Option<u32>,
            )| {
                let rules = Module::rules();
                let parent = entity.try_unwrap()?;
                let attacker = attacker.try_unwrap()?;
                let damage_kind = DamageKind::unwrap_from_str(&damage_kind);

                let min_damage = min_damage as u32;
                let max_damage = max_damage as u32;
                let damage = {
                    let parent = &parent.borrow().actor.stats;
                    let attack = Attack::special(
                        parent,
                        min_damage,
                        max_damage,
                        ap.unwrap_or(0),
                        damage_kind,
                        AttackKind::Dummy,
                    );
                    let damage = &attack.damage;
                    rules.roll_damage(damage, &parent.armor, &parent.resistance, 1.0)
                };

                if !damage.is_empty() {
                    EntityState::remove_hp(&parent, &attacker, HitKind::Hit, damage.clone());
                }

                let area_state = GameState::area_state();

                let feedback = AreaFeedbackText::with_damage(
                    &parent.borrow(),
                    &area_state.borrow(),
                    HitKind::Auto,
                    HitFlags::default(),
                    &damage,
                );
                area_state.borrow_mut().add_feedback_text(feedback);
                Ok(())
            },
        );

        methods.add_method("heal_damage", |_, entity, amount: f32| {
            let amount = amount as u32;
            let parent = entity.try_unwrap()?;
            parent.borrow_mut().actor.add_hp(amount);
            let area_state = GameState::area_state();

            let mut feedback =
                AreaFeedbackText::with_target(&parent.borrow(), &area_state.borrow());
            feedback.add_entry(format!("{}", amount), ColorKind::Heal);
            area_state.borrow_mut().add_feedback_text(feedback);

            Ok(())
        });

        methods.add_method(
            "add_class_stat",
            |_, entity, (stat, amount): (String, f32)| {
                let amount = amount as u32;
                let parent = entity.try_unwrap()?;
                parent.borrow_mut().actor.add_class_stat(&stat, amount);
                let area = GameState::area_state();

                let mut feedback = AreaFeedbackText::with_target(&parent.borrow(), &area.borrow());
                feedback.add_entry(format!("{}", amount), ColorKind::Heal);
                area.borrow_mut().add_feedback_text(feedback);
                Ok(())
            },
        );

        methods.add_method(
            "remove_class_stat",
            |_, entity, (stat, amount): (String, f32)| {
                let amount = amount as u32;
                let parent = entity.try_unwrap()?;
                parent.borrow_mut().actor.remove_class_stat(&stat, amount);
                let area = GameState::area_state();

                let mut feedback = AreaFeedbackText::with_target(&parent.borrow(), &area.borrow());
                feedback.add_entry(format!("{}", amount), ColorKind::Hit);
                area.borrow_mut().add_feedback_text(feedback);
                Ok(())
            },
        );

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

        methods.add_method("add_ap", |_, entity, ap| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().actor.add_ap(ap);
            Ok(())
        });

        methods.add_method("remove_ap", |_, entity, ap| {
            let entity = entity.try_unwrap()?;
            entity.borrow_mut().actor.remove_ap(ap);
            Ok(())
        });

        methods.add_method("base_class", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.actor.actor.base_class().id.clone())
        });

        methods.add_method("id", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.unique_id().to_string())
        });

        methods.add_method("name", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.actor.actor.name.to_string())
        });

        methods.add_method("has_ability", |_, entity, id: String| {
            let entity = entity.try_unwrap()?;
            let has = entity.borrow().actor.actor.has_ability_with_id(&id);
            Ok(has)
        });

        methods.add_method("get_ability", |_, entity, id: String| {
            let ability = match Module::ability(&id) {
                None => {
                    return Err(rlua::Error::FromLuaConversionError {
                        from: "String",
                        to: "ScriptAbility",
                        message: Some(format!("Ability '{}' does not exist", id)),
                    });
                }
                Some(ability) => ability,
            };
            if ability.active.is_none() {
                return Err(rlua::Error::FromLuaConversionError {
                    from: "String",
                    to: "ScriptAbility",
                    message: Some(format!("Ability '{}' is not active", id)),
                });
            }
            let entity = entity.try_unwrap()?;
            if !entity.borrow().actor.actor.has_ability(&ability) {
                return Ok(None);
            }

            Ok(Some(ScriptAbility::from(&ability)))
        });

        methods.add_method("get_abilities_with_group", |_, entity, group: String| {
            let entity = entity.try_unwrap()?;
            let actor = &entity.borrow().actor;

            let mut table = Vec::new();
            for state in actor.ability_states.values() {
                if state.group != group {
                    continue;
                }
                table.push(ScriptAbility::from(&state.ability));
            }

            Ok(table)
        });

        methods.add_method("ability_level_from_id", |_, entity, ability_id: String| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();

            match entity.actor.actor.ability_level(&ability_id) {
                None => Ok(0),
                Some(level) => Ok(level + 1),
            }
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
            for (_, state) in entity.actor.ability_states.iter() {
                if state.is_active_mode() {
                    return Ok(true);
                }
            }
            Ok(false)
        });

        methods.add_method("get_active_mode", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            for (id, state) in entity.actor.ability_states.iter() {
                if state.is_active_mode() {
                    let ability = Module::ability(id).unwrap();
                    return Ok(Some(ScriptAbility::from(&ability)));
                }
            }

            Ok(None)
        });

        methods.add_method("stats", &create_stats_table);

        methods.add_method("race", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let race_id = entity.borrow().actor.actor.race.id.to_string();
            Ok(race_id)
        });

        methods.add_method("image_layer_offset", |_, entity, layer: String| {
            let layer = match ImageLayer::from_str(&layer) {
                Err(e) => {
                    return Err(rlua::Error::FromLuaConversionError {
                        from: "String",
                        to: "ImageLayer",
                        message: Some(e.to_string()),
                    });
                }
                Ok(layer) => layer,
            };

            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            let offset = entity
                .actor
                .actor
                .race
                .get_image_layer_offset(layer)
                .unwrap_or(&(0.0, 0.0));
            let mut table: HashMap<&str, f32> = HashMap::new();
            table.insert("x", offset.0);
            table.insert("y", offset.1);
            Ok(table)
        });

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
        methods.add_method("location", |lua, entity, ()| {
            let entity = entity.try_unwrap()?;
            let location = lua.create_table()?;
            {
                let entity = entity.borrow();
                location.set("x", entity.location.x)?;
                location.set("y", entity.location.y)?;
                location.set("area", entity.location.area_id.to_string())?;
            }
            Ok(location)
        });
        methods.add_method("area", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let id = entity.borrow().location.area_id.to_string();
            Ok(id)
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
            let x =
                entity.borrow().location.x as f32 + entity.borrow().size.width as f32 / 2.0 - 0.5;
            Ok(x)
        });

        methods.add_method("center_y", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let y =
                entity.borrow().location.y as f32 + entity.borrow().size.height as f32 / 2.0 - 0.5;
            Ok(y)
        });

        methods.add_method("dist_to_entity", |_, entity, target: ScriptEntity| {
            let entity = entity.try_unwrap()?;
            let target = target.try_unwrap()?;
            let entity = &*entity.borrow();
            let target = &*target.borrow();

            let result = dist(entity, target);
            Ok(result)
        });

        methods.add_method("dist_to_point", |_, entity, point: HashMap<String, i32>| {
            let (x, y) = unwrap_point(point)?;
            let entity = entity.try_unwrap()?;
            let entity = &*entity.borrow();

            let result = dist(entity, &Point::new(x, y));
            Ok(result)
        });

        methods.add_method("is_threatened", |_, entity, ()| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();
            Ok(entity.actor.is_threatened())
        });

        methods.add_method("is_threatened_by", |_, entity, target: ScriptEntity| {
            let entity = entity.try_unwrap()?;
            let entity = entity.borrow();

            let target = target.index.unwrap_or(std::usize::MAX);
            Ok(entity.actor.p_stats().is_threatened_by(target))
        });
    }
}

fn move_towards_dest(parent: Rc<RefCell<EntityState>>, dest: Destination) -> Result<bool> {
    let mgr = GameState::turn_manager();
    let area = GameState::get_area_state(&parent.borrow().location.area_id).unwrap();
    let mut to_ignore = Vec::new();
    for e in area.borrow().entity_iter() {
        let other = mgr.borrow().entity(*e);
        if parent.borrow().ai_group() != other.borrow().ai_group() { continue; }

        if parent.borrow().is_friendly(&other.borrow()) {
            to_ignore.push(*e);
        }
    }

    Ok(GameState::move_towards_dest(
            &parent,
            &to_ignore,
            dest,
            None,
    ))
}

pub fn unwrap_point(point: HashMap<String, i32>) -> Result<(i32, i32)> {
    let x = match point.get("x") {
        None => {
            return Err(rlua::Error::FromLuaConversionError {
                from: "ScriptPoint",
                to: "Point",
                message: Some("Point must have x and y coordinates".to_string()),
            });
        }
        Some(x) => *x,
    };

    let y = match point.get("y") {
        None => {
            return Err(rlua::Error::FromLuaConversionError {
                from: "ScriptPoint",
                to: "Point",
                message: Some("Point must have x and y coordinates".to_string()),
            });
        }
        Some(y) => *y,
    };

    Ok((x, y))
}

fn create_stats_table<'a>(
    lua: Context<'a>,
    parent: &ScriptEntity,
    _args: (),
) -> Result<rlua::Table<'a>> {
    let rules = Module::rules();

    let parent = parent.try_unwrap()?;
    let parent = parent.borrow();
    let src = &parent.actor.stats;

    let stats = lua.create_table()?;
    stats.set("current_hp", parent.actor.hp())?;
    stats.set("current_ap", parent.actor.ap())?;
    stats.set("current_xp", parent.actor.xp())?;

    stats.set("strength", src.attributes.strength)?;
    stats.set("dexterity", src.attributes.dexterity)?;
    stats.set("endurance", src.attributes.endurance)?;
    stats.set("perception", src.attributes.perception)?;
    stats.set("intellect", src.attributes.intellect)?;
    stats.set("wisdom", src.attributes.wisdom)?;

    {
        use self::Attribute::*;
        stats.set(
            "strength_bonus",
            src.attributes.bonus(Strength, rules.base_attribute),
        )?;
        stats.set(
            "dexterity_bonus",
            src.attributes.bonus(Dexterity, rules.base_attribute),
        )?;
        stats.set(
            "endurance_bonus",
            src.attributes.bonus(Endurance, rules.base_attribute),
        )?;
        stats.set(
            "perception_bonus",
            src.attributes.bonus(Perception, rules.base_attribute),
        )?;
        stats.set(
            "intellect_bonus",
            src.attributes.bonus(Intellect, rules.base_attribute),
        )?;
        stats.set(
            "wisdom_bonus",
            src.attributes.bonus(Wisdom, rules.base_attribute),
        )?;
    }

    stats.set("base_armor", src.armor.base())?;
    let armor = lua.create_table()?;
    for kind in DamageKind::iter() {
        armor.set(kind.to_str(), src.armor.amount(*kind))?;
    }
    stats.set("armor", armor)?;

    let resistance = lua.create_table()?;
    for kind in DamageKind::iter() {
        resistance.set(kind.to_str(), src.resistance.amount(*kind))?;
    }
    stats.set("resistance", resistance)?;

    stats.set("level", parent.actor.actor.total_level)?;
    stats.set("caster_level", src.caster_level)?;
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

    stats.set("touch_distance", src.touch_distance())?;
    stats.set("attack_distance", src.attack_distance())?;
    stats.set("attack_is_melee", src.attack_is_melee())?;
    stats.set("attack_is_ranged", src.attack_is_ranged())?;

    stats.set("concealment", src.concealment)?;
    stats.set("concealment_ignore", src.concealment_ignore)?;
    stats.set("crit_chance", src.crit_chance)?;
    stats.set("graze_threshold", src.graze_threshold)?;
    stats.set("hit_threshold", src.hit_threshold)?;
    stats.set("graze_multiplier", src.graze_multiplier)?;
    stats.set("hit_multiplier", src.hit_multiplier)?;
    stats.set("crit_multiplier", src.crit_multiplier)?;
    stats.set("movement_rate", src.movement_rate)?;
    stats.set("attack_cost", src.attack_cost)?;

    stats.set("is_hidden", src.hidden)?;
    stats.set("is_abilities_disabled", src.abilities_disabled)?;
    stats.set("is_attack_disabled", src.attack_disabled)?;
    stats.set("is_move_disabled", src.move_disabled)?;

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

fn targets(_lua: Context, parent: &ScriptEntity, _args: ()) -> Result<ScriptEntitySet> {
    let parent = parent.try_unwrap()?;
    let area_id = parent.borrow().location.area_id.to_string();

    let mgr = GameState::turn_manager();
    let mut indices = Vec::new();
    for entity in mgr.borrow().entity_iter() {
        let entity = entity.borrow();
        if parent.borrow().is_hostile(&entity) && entity.actor.stats.hidden {
            continue;
        }

        if entity.actor.is_dead() {
            continue;
        }
        if !entity.location.is_in_area_id(&area_id) {
            continue;
        }

        indices.push(Some(entity.index()));
    }

    let parent_index = parent.borrow().index();
    Ok(ScriptEntitySet {
        parent: parent_index,
        indices,
        selected_point: None,
        affected_points: Vec::new(),
        surface: None,
    })
}

fn get_on_activate_fn(is_party_member: bool, ai_data: &AIData) -> String {
    if is_party_member {
        "on_activate".to_string()
    } else if let Some(func) = &ai_data.on_activate_fn {
        func.to_string()
    } else {
        "on_activate".to_string()
    }
}
