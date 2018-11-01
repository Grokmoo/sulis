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

use sulis_core::util::ExtInt;
use {GameState};
use animation::{Anim};
use animation::particle_generator::{Param};
use script::{CallbackData, Result, script_particle_generator};

/// An animation that moves the pixel precise coordinates of
/// an entity.  Normally created via `ScriptEntity:create_subps_anim`.
/// When this animation is complete, the parent subpos is set back to
/// the default, (0, 0)
///
/// # `activate()`
/// Activates and applies this animation to the parent.
///
/// # `param(value: Float, dt: Float (Optional), d2t: Float (Optional),
/// d3t: Float (Optional))`
/// Creates a param.  See `ScriptParticleGenerator`
///
/// # `set_position(x: Param, y: Param)`
/// Sets the parent entity position `x` and `y` coordinates, as Params.
///
/// # `set_completion_callback(callback: CallbackData)`
/// Sets the specified `callback` to be called when this animation completes.
///
/// # `add_callback(callback: CallbackData, time: Float)`
/// Sets the specified `callback` to be called after the specified `time` has elapsed,
/// in seconds.
#[derive(Clone)]
pub struct ScriptSubposAnimation {
    parent: usize,
    completion_callback: Option<CallbackData>,
    callbacks: Vec<(f32, CallbackData)>,
    duration_millis: ExtInt,
    position: (Param, Param),
}

impl ScriptSubposAnimation {
    pub fn new(parent: usize, duration_millis: ExtInt) -> ScriptSubposAnimation {
        ScriptSubposAnimation {
            parent,
            completion_callback: None,
            callbacks: Vec::new(),
            duration_millis,
            position: (Param::fixed(0.0), Param::fixed(0.0)),
        }
    }
}

impl UserData for ScriptSubposAnimation {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
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

    GameState::add_animation(anim);

    Ok(())
}

pub fn create_anim(data: &ScriptSubposAnimation) -> Result<Anim> {
    let mgr = GameState::turn_manager();
    let parent = mgr.borrow().entity(data.parent);

    let x = data.position.0.clone();
    let y = data.position.1.clone();

    let mut anim = Anim::new_entity_subpos(&parent, data.duration_millis, x, y);

    if let Some(ref cb) = data.completion_callback {
        anim.add_completion_callback(Box::new(cb.clone()));
    }

    for &(time, ref cb) in data.callbacks.iter() {
        anim.add_update_callback(Box::new(cb.clone()), (time * 1000.0) as u32);
    }

    Ok(anim)
}

