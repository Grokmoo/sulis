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

use rlua::{Context, UserData, UserDataMethods};

use crate::animation::particle_generator::Param;
use crate::animation::Anim;
use crate::script::{script_particle_generator, CallbackData, Result};
use crate::GameState;
use sulis_core::util::ExtInt;

/// A color animation changing a parent entity's base or secondary
/// color.  Created normally by `ScriptEntity:create_color_anim`
///
/// # `activate()`
/// Creates and applies this animation directly.
///
/// # `set_color(r: Param, g: Param, b: Param, a: Param)`
/// Sets the color components of this animation to the specified `Param`s
///
/// # `set_color_sec(r: Param, g: Param, b: Param, a: Param)`
/// Sets the secondary color copmonents of this animation to the
/// specified `Param`s
///
/// # `set_completion_callback(callback: CallbackData)`
/// Adds the specified `callback` to be called when this animation is complete.
///
/// # `add_callback(callback: CallbackData, time: Float)`
/// Sets the specified `callback` to be called at the specified `time` elapsed,
/// in seconds.
///
/// # `param(value: Float, dt: Float (Optional), d2t: Float (Optional),
/// d3t: Float (Optional))`
/// Creates a param for use in this animation's setup.  See `ScriptParticleGenerator`
#[derive(Clone)]
pub struct ScriptColorAnimation {
    parent: usize,
    completion_callback: Option<CallbackData>,
    callbacks: Vec<(f32, CallbackData)>,
    duration_millis: ExtInt,
    color: [Param; 4],
    color_sec: [Param; 4],
}

impl ScriptColorAnimation {
    pub fn new(parent: usize, duration_millis: ExtInt) -> ScriptColorAnimation {
        ScriptColorAnimation {
            parent,
            completion_callback: None,
            callbacks: Vec::new(),
            duration_millis,
            color: [
                Param::fixed(1.0),
                Param::fixed(1.0),
                Param::fixed(1.0),
                Param::fixed(1.0),
            ],
            color_sec: [
                Param::fixed(0.0),
                Param::fixed(0.0),
                Param::fixed(0.0),
                Param::fixed(0.0),
            ],
        }
    }
}

impl UserData for ScriptColorAnimation {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("activate", activate);
        methods.add_method("param", script_particle_generator::param);
        methods.add_method_mut(
            "set_color",
            |_, gen, (r, g, b, a): (Param, Param, Param, Param)| {
                gen.color = [r, g, b, a];
                Ok(())
            },
        );
        methods.add_method_mut(
            "set_color_sec",
            |_, gen, (r, g, b, a): (Param, Param, Param, Param)| {
                gen.color_sec = [r, g, b, a];
                Ok(())
            },
        );
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

fn activate(_lua: Context, data: &ScriptColorAnimation, _args: ()) -> Result<()> {
    let anim = create_anim(data)?;

    GameState::add_animation(anim);

    Ok(())
}

pub fn create_anim(data: &ScriptColorAnimation) -> Result<Anim> {
    let mgr = GameState::turn_manager();
    let parent = mgr.borrow().entity(data.parent);

    let mut anim =
        Anim::new_entity_color(&parent, data.duration_millis, data.color, data.color_sec);

    if let Some(ref cb) = data.completion_callback {
        anim.add_completion_callback(Box::new(cb.clone()));
    }

    for &(time, ref cb) in data.callbacks.iter() {
        anim.add_update_callback(Box::new(cb.clone()), (time * 1000.0) as u32);
    }

    Ok(anim)
}
