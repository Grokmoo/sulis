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

use sulis_core::resource::ResourceSet;

use {GameState};
use animation::{Animation, ParticleGenerator};
use animation::particle_generator::{Dist, Param, DistParam};
use script::{CallbackData, Result};


#[derive(Clone)]
pub struct ScriptParticleGenerator {
    parent: usize,
    sprite: String,
    duration_secs: f32,

    gen_rate: Option<Param>,
    position: Option<(Param, Param)>,
    callback: Option<CallbackData>,

    particle_x_dist: Option<DistParam>,
    particle_y_dist: Option<DistParam>,
    particle_duration_dist: Option<Dist>,
    particle_size_dist: Option<(Dist, Dist)>,
}

impl ScriptParticleGenerator {
    pub fn new(parent: usize, sprite: String, duration_secs: f32) -> ScriptParticleGenerator {
        ScriptParticleGenerator {
            parent,
            sprite,
            duration_secs,
            gen_rate: None,
            position: None,
            callback: None,
            particle_x_dist: None,
            particle_y_dist: None,
            particle_duration_dist: None,
            particle_size_dist: None,
        }
    }
}

impl UserData for ScriptParticleGenerator {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("activate", &activate);
        methods.add_method("fixed_param", |_, _, value: f32| Ok(Param::fixed(value)));
        methods.add_method("speed_param", |_, _, (value, dt): (f32, f32)| Ok(Param::with_speed(value, dt)));
        methods.add_method("accel_param", |_, _, (value, dt, d2t): (f32, f32, f32)| {
            Ok(Param::with_accel(value, dt, d2t))
        });
        methods.add_method("zero_dist", |_, _, _: ()| Ok(Dist::create_fixed(0.0)));
        methods.add_method("fixed_dist", |_, _, value: f32| Ok(Dist::create_fixed(value)));
        methods.add_method("uniform_dist", |_, _, (min, max): (f32, f32)| Ok(Dist::create_uniform(min, max)));
        methods.add_method_mut("set_gen_rate", |_, gen, rate: Param| {
            gen.gen_rate = Some(rate);
            Ok(())
        });
        methods.add_method_mut("set_position", |_, gen, (x, y): (Param, Param)| {
            gen.position = Some((x, y));
            Ok(())
        });
        methods.add_method_mut("set_callback", |_, gen, cb: CallbackData| {
            gen.callback = Some(cb);
            Ok(())
        });
        methods.add_method_mut("set_particle_x_dist", |_, gen, (value, dt, d2t): (Dist, Dist, Dist)| {
            gen.particle_x_dist = Some(DistParam::new(value, dt, d2t));
            Ok(())
        });
        methods.add_method_mut("set_particle_y_dist", |_, gen, (value, dt, d2t): (Dist, Dist, Dist)| {
            gen.particle_y_dist = Some(DistParam::new(value, dt, d2t));
            Ok(())
        });
        methods.add_method_mut("set_particle_duration_dist", |_, gen, value: Dist| {
            gen.particle_duration_dist = Some(value);
            Ok(())
        });
        methods.add_method_mut("set_particle_size_dist", |_, gen, (width, height): (Dist, Dist)| {
            gen.particle_size_dist = Some((width, height));
            Ok(())
        });
    }
}

fn activate(_lua: &Lua, gen: &ScriptParticleGenerator, _args: ()) -> Result<()> {
    let area_state = GameState::area_state();

    let parent = area_state.borrow().get_entity(gen.parent);
    let sprite = match ResourceSet::get_sprite(&gen.sprite) {
        Ok(sprite) => sprite,
        Err(_) => {
            warn!("Unable to locate sprite '{}' for particle generator", gen.sprite);
            return Err(rlua::Error::FromLuaConversionError {
                from: "ScriptParticleGenerator",
                to: "ParticleGenerator",
                message: Some("Sprite not found".to_string()),
            });
        }
    };
    let duration = (gen.duration_secs * 1000.0) as u32;

    let mut pgen = ParticleGenerator::new(parent, duration, sprite);

    if let Some(ref rate) = gen.gen_rate {
        pgen.set_gen_rate(rate.clone());
    }

    if let Some(&(ref x, ref y)) = gen.position.as_ref() {
        pgen.set_position(x.clone(), y.clone());
    }

    if let Some(ref cb) = gen.callback {
        pgen.set_callback(Some(Box::new(cb.clone())));
    }

    if let Some(ref dist) = gen.particle_x_dist {
        pgen.set_particle_x_dist(dist.clone());
    }

    if let Some(ref dist) = gen.particle_y_dist {
        pgen.set_particle_y_dist(dist.clone());
    }

    if let Some(ref dist) = gen.particle_duration_dist {
        pgen.set_particle_duration_dist(dist.clone());
    }

    if let Some(&(ref width, ref height)) = gen.particle_size_dist.as_ref() {
        pgen.set_particle_size_dist(width.clone(), height.clone());
    }

    GameState::add_animation(Box::new(pgen));

    Ok(())
}

