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

use rlua::{self, Lua, UserData, UserDataMethods};

use sulis_module::{Module, OnTrigger};

use script::area_targeter::Shape;
use script::{AreaTargeter, Result, ScriptEntity, ScriptEntitySet, ScriptItemKind};
use {GameState};

#[derive(Clone)]
pub enum Kind {
    Ability(String),
    Item(ScriptItemKind),
}

/// Created by calling `create_targeter` on a `ScriptEntity`.  A targeter
/// allows the player (or ai script) to select a specific target from a list
/// of available targets, or choose a location for an area of effect.
///
/// The targeter is configured, and then finally activated with `activate()`
///
/// # `activate()`
/// Once configuration of this targeter is complete, activate it.  The player (or ai
/// script) will then be able to interact with it.
///
/// # `set_callback_fn(func: String)`
/// Sets the callback function that is called when the target is selected.  By
/// default, this is `on_target_select(parent, ability/item, targets)`.  However,
/// if more than one targeter is used in a given script then this method can
/// be used to make those targeters use different functions.
///
/// # `set_callback_custom_target(target: ScriptEntity)`
/// Causes the specified `target` to always be added to the list of targets returned
/// by the targeter.  This is occasionally useful for targeters that select a location
/// rather than an entity.
///
/// # `add_all_selectable(selectable: ScriptEntitySet)`
/// Adds all entities in `selectable` as possible selections for this targeter.  They must
/// still meet all other constraints added in order to be valid selections.
///
/// # `add_selectable(selectable: ScriptEntity)`
/// Adds a single entity `selectable` as a possible selection for this targeter.  It must
/// still meet all other constraints in order to be a valid selection.
///
/// # `set_show_mouseover(show: Bool)`
/// Sets whether to `show` the entity mouseover for this targeter.  By default, it is shown.
/// For some abilities, it may be a better user experience to not show it.
///
/// # `set_free_select(range: Float)`
/// Sets free select mode, allowing any point within the specified range of the parent entity
/// to be selected.  By default, the targeter is in entity select mode.
///
/// # `set_free_select_must_be_passable(size_id: String)`
/// Only applies when the targeter is in free select mode.  Requires any point that is selected
/// in free select to be passable for the size specified by `size_id`
///
/// # `impass_blocks_affected_points(blocks: bool)`
/// Sets whether or not an impassable tile blocks further affected points in a line extending
/// from the targeter center outwards.  Defaults to false
///
/// # `invis_blocks_affected_points(blocks: bool)`
/// Sets whether a visibility blocking tile blocks further affected points in a line extending
/// from the targeter center outwards.  Defaults to false
///
/// # `allow_affected_points_impass(allow: bool)`
/// Sets whether or not to allow affected points to be terrain impassable.  defaults to true
///
/// # `allow_affected_points_invis(allow: bool)`
/// Sets whether or not to allow affected points to prevent visibility.  defaults to false
///
/// # `add_all_effectable(targets: ScriptEntitySet)`
/// Adds all entities in the set as targets that can potentially be affected by this targeter.
/// Only affectable entities within the area of effect of the chosen shape will end up as
/// targets.
///
/// # `add_effectable(target: ScriptEntity)`
/// Adds the specified entity as a target to potentially be affected by this targeter.  See
/// `add_all_effectable`
///
/// # `set_max_effectable(max: Int)`
/// Sets the maximum number of targets that this targeter may affect and return.
///
/// # `set_shape_circle(radius: Float)`
/// Sets the shape of this targeter to a circle with the specified `radius`, in tiles
///
/// # `set_shape_line(size: String, x: Int, y: Int, length: Int)`
/// Sets the shape of this targeter to a line that extends from `x`, `y` as its origin for the
/// specified `length`.  The width of the line is determined by the `size`.  Effectively,
/// only the angle that the line is pointing in is determined by the user.
///
/// # `set_shape_line_segment(size: String, x: Int, y: Int)`
/// Sets the shape of this targeter to a line segment from `x`, `y` to the user selected point.
/// The width of the segment is based on `size`.
///
/// # `set_shape_object_size(size: String)`
/// Sets this targeter to affect a set of points with the shape specified by the specified
/// `size`.
///
/// # `set_shape_cone(x: Float, y: Float, radius: Float, angle: Float)`
/// Sets this targeter to a cone shape, with center / origin `x`, `y`, a specified `radius`,
/// and subtending the specified `angle`.  The angle is in radians.
#[derive(Clone)]
pub struct TargeterData {
    pub kind: Kind,
    pub parent: usize,
    pub selectable: Vec<Option<usize>>,
    pub effectable: Vec<Option<usize>>,
    pub max_effectable: Option<usize>,
    pub shape: Shape,
    pub show_mouseover: bool,
    pub free_select: Option<f32>,
    pub free_select_must_be_passable: Option<String>,
    pub allow_affected_points_impass: bool,
    pub impass_blocks_affected_points: bool,
    pub invis_blocks_affected_points: bool,
    pub allow_affected_points_invis: bool,
    pub on_target_select_func: String,
    pub on_target_select_custom_target: Option<usize>,
}

impl TargeterData {
    fn new(parent: usize, kind: Kind) -> TargeterData {
        TargeterData {
            parent,
            kind,
            selectable: Vec::new(),
            effectable: Vec::new(),
            max_effectable: None,
            shape: Shape::Single,
            show_mouseover: true,
            free_select: None,
            free_select_must_be_passable: None,
            on_target_select_func: "on_target_select".to_string(),
            on_target_select_custom_target: None,
            allow_affected_points_impass: true,
            impass_blocks_affected_points: false,
            invis_blocks_affected_points: false,
            allow_affected_points_invis: false,
        }
    }

    pub fn new_item(parent: usize, kind: ScriptItemKind) -> TargeterData {
        TargeterData::new(parent, Kind::Item(kind))
    }

    pub fn new_ability(parent: usize, ability_id: &str) -> TargeterData {
        TargeterData::new(parent, Kind::Ability(ability_id.to_string()))
    }
}

impl UserData for TargeterData {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("activate", &activate);
        methods.add_method_mut("set_callback_fn", |_, targeter, func: String| {
            targeter.on_target_select_func = func;
            Ok(())
        });
        methods.add_method_mut("set_callback_custom_target", |_, targeter, target: ScriptEntity| {
            let index = target.try_unwrap_index()?;
            targeter.on_target_select_custom_target = Some(index);
            Ok(())
        });
        methods.add_method_mut("add_all_selectable", |_, targeter, selectable: ScriptEntitySet| {
            targeter.selectable.append(&mut selectable.indices.clone());
            Ok(())
        });
        methods.add_method_mut("add_selectable", |_, targeter, target: ScriptEntity| {
            let index = target.try_unwrap_index()?;
            targeter.selectable.push(Some(index));
            Ok(())
        });
        methods.add_method_mut("set_show_mouseover", |_, targeter, val: bool| {
            targeter.show_mouseover = val;
            Ok(())
        });
        methods.add_method_mut("set_free_select", |_, targeter, val: f32| {
            targeter.free_select = Some(val);
            Ok(())
        });

        methods.add_method_mut("impass_blocks_affected_points", |_, targeter, blocks: bool| {
            targeter.impass_blocks_affected_points = blocks;
            Ok(())
        });

        methods.add_method_mut("invis_blocks_affected_points", |_, targeter, blocks: bool| {
            targeter.invis_blocks_affected_points = blocks;
            Ok(())
        });

        methods.add_method_mut("allow_affected_points_impass", |_, targeter, allow: bool| {
            targeter.allow_affected_points_impass = allow;
            Ok(())
        });

        methods.add_method_mut("allow_affected_points_invis", |_, targeter, allow: bool| {
            targeter.allow_affected_points_invis = allow;
            Ok(())
        });

        methods.add_method_mut("set_free_select_must_be_passable", |_, targeter, val: String| {
            match Module::object_size(&val) {
                None => {
                    warn!("No object size '{}' found", val);
                    return Err(rlua::Error::FromLuaConversionError {
                        from: "String",
                        to: "ObjectSize",
                        message: Some("Size must be the ID of a valid object size".to_string())
                    });
                },
                Some(_) => (),
            }
            targeter.free_select_must_be_passable = Some(val);
            Ok(())
        });
        methods.add_method_mut("add_all_effectable", |_, targeter, targets: ScriptEntitySet| {
            targeter.effectable.append(&mut targets.indices.clone());
            Ok(())
        });
        methods.add_method_mut("add_effectable", |_, targeter, target: ScriptEntity| {
            let index = target.try_unwrap_index()?;
            targeter.effectable.push(Some(index));
            Ok(())
        });
        methods.add_method_mut("set_max_effectable", |_, targeter, max: usize| {
            targeter.max_effectable = Some(max);
            Ok(())
        });
        methods.add_method_mut("set_shape_circle", |_, targeter, radius: f32| {
            targeter.shape = Shape::Circle { radius };
            Ok(())
        });
        methods.add_method_mut("set_shape_line", |_, targeter, (size, origin_x, origin_y, length):
                               (String, i32, i32, i32)| {
            match Module::object_size(&size) {
                None => {
                    warn!("No object size '{}' found", size);
                    return Err(rlua::Error::FromLuaConversionError {
                        from: "String",
                        to: "ObjectSize",
                        message: Some("Size must be the ID of a valid object size".to_string())
                    });
                }, Some(_) => (),
            }
            targeter.shape = Shape::Line { size, origin_x, origin_y, length };
            Ok(())
        });
        methods.add_method_mut("set_shape_line_segment",
                               |_, targeter, (size, origin_x, origin_y): (String, i32, i32)| {
            match Module::object_size(&size) {
                None => {
                    warn!("No object size '{}' found", size);
                    return Err(rlua::Error::FromLuaConversionError {
                        from: "String",
                        to: "ObjectSize",
                        message: Some("Size must be the ID of a valid object size".to_string())
                    });
                },
                Some(_) => (),
            }
            targeter.shape = Shape::LineSegment { size, origin_x, origin_y };
            Ok(())
        });
        methods.add_method_mut("set_shape_object_size", |_, targeter, size: String| {
            match Module::object_size(&size) {
                None => {
                    warn!("No object size '{}' found", size);
                    return Err(rlua::Error::FromLuaConversionError {
                        from: "String",
                        to: "ObjectSize",
                        message: Some("Size must be the ID of a valid object size".to_string())
                    });
                },
                Some(_) => (),
            }
            targeter.shape = Shape::ObjectSize { size };
            Ok(())
        });
        methods.add_method_mut("set_shape_cone", |_, targeter,
                               (origin_x, origin_y, radius, angle): (f32, f32, f32, f32)| {
            let origin_x = origin_x.floor() as i32;
            let origin_y = origin_y.floor() as i32;
            targeter.shape = Shape::Cone { origin_x, origin_y, radius, angle };
            Ok(())
        });
    }
}

fn activate(_lua: &Lua, data: &TargeterData, _args: ()) -> Result<()> {
    info!("Activating targeter");

    let parent = ScriptEntity::new(data.parent).try_unwrap()?;
    if parent.borrow().is_party_member() && data.free_select.is_none() && data.selectable.is_empty() {
        let cb = OnTrigger::SayLine("No valid targets".to_string());

        GameState::add_ui_callback(vec![cb], &parent, &parent);
        return Ok(());
    }

    let targeter = AreaTargeter::from(data);

    let area_state = GameState::area_state();
    area_state.borrow_mut().set_targeter(targeter);

    Ok(())
}
