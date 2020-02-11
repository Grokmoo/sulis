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

/// An animation that changes the size of an entity.
/// Normally created via `ScriptEntity:create_scale_anim`.
/// Upon completion, the parent scale is set back to 1.0
///
/// # `activate()`
/// Activates and applies this animation to the parent.
///
/// # `param(value: Float, dt: Float (Optional), d2t: Float (Optional),
/// d3t: Float (Optional))`
/// Creates a param.  See `ScriptParticleGenerator`
///
/// # `set_scale(scale: Param)`
/// Sets the parent entity scale `scale`
///
/// # `set_completion_callback(callback: CallbackData)`
/// Sets the specified `callback` to be called when this animation completes.
///
/// # `add_callback(callback: CallbackData, time: Float)`
/// Sets the specified `callback` to be called after the specified `time` has elapsed,
/// in seconds.
#[derive(Clone)]
pub struct ScriptScaleAnimation {
    parent: usize,
    completion_callback: Option<CallbackData>,
    callbacks: Vec<(f32, CallbackData)>,
    duration_millis: ExtInt,
    scale: Param,
}

impl ScriptScaleAnimation {
    pub fn new(parent: usize, duration_millis: ExtInt) -> ScriptScaleAnimation {
        ScriptScaleAnimation {
            parent,
            completion_callback: None,
            callbacks: Vec::new(),
            duration_millis,
            scale: Param::fixed(1.0),
        }
    }
}

impl UserData for ScriptScaleAnimation {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("activate", &activate);
        methods.add_method("param", &script_particle_generator::param);
        methods.add_method_mut("set_scale", |_, gen, scale: Param| {
            gen.scale = scale;
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

fn activate(_lua: Context, data: &ScriptScaleAnimation, _args: ()) -> Result<()> {
    let anim = create_anim(data)?;

    GameState::add_animation(anim);

    Ok(())
}

pub fn create_anim(data: &ScriptScaleAnimation) -> Result<Anim> {
    let mgr = GameState::turn_manager();
    let parent = mgr.borrow().entity(data.parent);

    let scale = data.scale;

    let mut anim = Anim::new_entity_scale(&parent, data.duration_millis, scale);

    if let Some(ref cb) = data.completion_callback {
        anim.add_completion_callback(Box::new(cb.clone()));
    }

    for &(time, ref cb) in data.callbacks.iter() {
        anim.add_update_callback(Box::new(cb.clone()), (time * 1000.0) as u32);
    }

    Ok(anim)
}
