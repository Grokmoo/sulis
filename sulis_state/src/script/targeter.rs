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
    fn add_methods(methods: &mut UserDataMethods<Self>) {
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
        let cb = OnTrigger {
            say_line: Some("No valid targets.".to_string()),
            ..Default::default()
        };

        GameState::add_ui_callback(cb, &parent, &parent);
        return Ok(());
    }

    let targeter = AreaTargeter::from(data);

    let area_state = GameState::area_state();
    area_state.borrow_mut().set_targeter(targeter);

    Ok(())
}
