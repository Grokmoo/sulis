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
use animation::{Animation, EntityColorAnimation};
use animation::particle_generator::{Param};
use script::{CallbackData, Result, script_particle_generator};

#[derive(Clone)]
pub struct ScriptColorAnimation {
    parent: usize,
    completion_callback: Option<CallbackData>,
    callbacks: Vec<(f32, CallbackData)>,
    duration_secs: f32,
    color: [Param; 4],
    color_sec: [Param; 4],
}

impl ScriptColorAnimation {
    pub fn new(parent: usize, duration_secs: f32) -> ScriptColorAnimation {
        ScriptColorAnimation {
            parent,
            completion_callback: None,
            callbacks: Vec::new(),
            duration_secs,
            color: [Param::fixed(1.0), Param::fixed(1.0), Param::fixed(1.0), Param::fixed(1.0)],
            color_sec: [Param::fixed(0.0), Param::fixed(0.0), Param::fixed(0.0), Param::fixed(0.0)],
        }
    }
}

impl UserData for ScriptColorAnimation {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("activate", &activate);
        methods.add_method("param", &script_particle_generator::param);
        methods.add_method_mut("set_color", |_, gen, (r, g, b, a): (Param, Param, Param, Param)| {
            gen.color = [r, g, b, a];
            Ok(())
        });
        methods.add_method_mut("set_color_sec", |_, gen, (r, g, b, a): (Param, Param, Param, Param)| {
            gen.color_sec = [r, g, b, a];
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

fn activate(_lua: &Lua, data: &ScriptColorAnimation, _args: ()) -> Result<()> {
    let anim = create_anim(data)?;

    GameState::add_animation(Box::new(anim));

    Ok(())
}

pub fn create_anim(data: &ScriptColorAnimation) -> Result<EntityColorAnimation> {
    let mgr = GameState::turn_manager();
    let parent = mgr.borrow().entity(data.parent);

    let mut anim = EntityColorAnimation::new(parent, data.color.clone(), data.color_sec.clone(),
        data.duration_secs);

    if let Some(ref cb) = data.completion_callback {
        anim.set_callback(Some(Box::new(cb.clone())));
    }

    for &(time, ref cb) in data.callbacks.iter() {
        anim.add_callback(Box::new(cb.clone()), time);
    }

    Ok(anim)
}

