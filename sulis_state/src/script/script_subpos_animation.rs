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

use rlua::{Lua, UserData, UserDataMethods};

use {GameState};
use animation::{Animation, EntitySubposAnimation};
use animation::particle_generator::{Param};
use script::{CallbackData, Result, script_particle_generator};

#[derive(Clone)]
pub struct ScriptSubposAnimation {
    parent: usize,
    completion_callback: Option<CallbackData>,
    callbacks: Vec<(f32, CallbackData)>,
    duration_secs: f32,
    position: (Param, Param),
}

impl ScriptSubposAnimation {
    pub fn new(parent: usize, duration_secs: f32) -> ScriptSubposAnimation {
        ScriptSubposAnimation {
            parent,
            completion_callback: None,
            callbacks: Vec::new(),
            duration_secs,
            position: (Param::fixed(0.0), Param::fixed(0.0)),
        }
    }
}

impl UserData for ScriptSubposAnimation {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("activate", &activate);
        methods.add_method("param", &script_particle_generator::param);
        methods.add_method_mut("set_position", |_, gen, (x, y): (Param, Param)| {
            gen.position = (x, y);
            Ok(())
        });
        methods.add_method_mut("set_completion_callback", |_, gen, cb: CallbackData| {
            gen.completion_callback = Some(cb);
            Ok(())
        });
        methods.add_method_mut("add_callback", |_, gen, (cb, time): (CallbackData, f32)| {
            gen.callbacks.push((time, cb));
            Ok(())
        });
    }
}

fn activate(_lua: &Lua, data: &ScriptSubposAnimation, _args: ()) -> Result<()> {
    let anim = create_anim(data)?;

    GameState::add_animation(Box::new(anim));

    Ok(())
}

pub fn create_anim(data: &ScriptSubposAnimation) -> Result<EntitySubposAnimation> {
    let area_state = GameState::area_state();
    let parent = area_state.borrow().get_entity(data.parent);

    let mut anim = EntitySubposAnimation::new(parent, data.position.clone(), data.duration_secs);

    if let Some(ref cb) = data.completion_callback {
        anim.set_callback(Some(Box::new(cb.clone())));
    }

    for &(time, ref cb) in data.callbacks.iter() {
        anim.add_callback(Box::new(cb.clone()), time);
    }

    Ok(anim)
}

