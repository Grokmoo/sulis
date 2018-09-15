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
use sulis_core::util::ExtInt;

use {GameState};
use animation::{Anim, self};
use animation::particle_generator::{Dist, Param, DistParam, DistParam2D, GeneratorModel};
use script::{CallbackData, Result};

#[derive(Clone)]
pub struct ScriptParticleGenerator {
    parent: usize,
    image: String,
    completion_callback: Option<CallbackData>,
    callbacks: Vec<(f32, CallbackData)>,
    model: GeneratorModel,
}

impl ScriptParticleGenerator {
    pub fn new(parent: usize, image: String, duration_millis: ExtInt) -> ScriptParticleGenerator {
        let mgr = GameState::turn_manager();
        let owner = mgr.borrow().entity(parent);
        let x = owner.borrow().location.x as f32 + owner.borrow().size.width as f32 / 2.0;
        let y = owner.borrow().location.y as f32 + owner.borrow().size.height as f32 / 2.0;

        let model = GeneratorModel::new(duration_millis, x, y);

        ScriptParticleGenerator {
            parent,
            image,
            completion_callback: None,
            callbacks: Vec::new(),
            model,
        }
    }

    pub fn set_completion_callback(&mut self, cb: CallbackData) {
        self.completion_callback = Some(cb);
    }

    pub fn new_anim(parent: usize, image: String, duration_millis: ExtInt) -> ScriptParticleGenerator {
        let mut pgen = ScriptParticleGenerator::new(parent, image, duration_millis);
        pgen.model.initial_overflow = 1.0;
        pgen.model.gen_rate = Param::fixed(0.0);
        pgen
    }

    pub fn owned_model(&self) -> GeneratorModel {
        self.model.clone()
    }
}

impl UserData for ScriptParticleGenerator {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("activate", &activate);
        methods.add_method("param", &param);
        methods.add_method("dist_param", &dist_param);
        methods.add_method("zero_dist", |_, _, _: ()| Ok(Dist::create_fixed(0.0)));
        methods.add_method("fixed_dist", |_, _, value: f32| Ok(Dist::create_fixed(value)));
        methods.add_method("uniform_dist", |_, _, (min, max): (f32, f32)| Ok(Dist::create_uniform(min, max)));
        methods.add_method("angular_dist", |_, _, (min_a, max_a, min_s, max_s): (f32, f32, f32, f32)| {
            Ok(Dist::create_angular(min_a, max_a, min_s, max_s))
        });
        methods.add_method_mut("set_draw_below_entities", |_, gen, _: ()| {
            gen.model.draw_above_entities = false;
            Ok(())
        });
        methods.add_method_mut("set_draw_above_entities", |_, gen, _: ()| {
            gen.model.draw_above_entities = true;
            Ok(())
        });
        methods.add_method_mut("set_initial_gen", |_, gen, value: f32| {
            gen.model.initial_overflow = value;
            Ok(())
        });
        methods.add_method_mut("set_moves_with_parent", |_, gen, _args: ()| {
            gen.model.moves_with_parent = true;
            Ok(())
        });
        methods.add_method_mut("set_gen_rate", |_, gen, rate: Param| {
            gen.model.gen_rate = rate;
            Ok(())
        });
        methods.add_method_mut("set_position", |_, gen, (x, y): (Param, Param)| {
            gen.model.position = (x, y);
            Ok(())
        });
        methods.add_method_mut("set_rotation", |_, gen, rotation: Param| {
            gen.model.rotation = Some(rotation);
            Ok(())
        });
        methods.add_method_mut("set_color", |_, gen, (r, g, b, a): (Param, Param, Param, Option<Param>)| {
            gen.model.red = r;
            gen.model.green = g;
            gen.model.blue = b;
            if let Some(a) = a {
                gen.model.alpha = a;
            }
            Ok(())
        });
        methods.add_method_mut("set_alpha", |_, gen, a: Param| {
            gen.model.alpha = a;
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
        methods.add_method_mut("set_particle_position_dist", |_, gen, (x, y): (DistParam, Option<DistParam>)| {
            gen.model.particle_position_dist = Some(DistParam2D::new(x, y));
           Ok(())
        });
        methods.add_method_mut("set_particle_duration_dist", |_, gen, value: Dist| {
            gen.model.particle_duration_dist = Some(value);
            Ok(())
        });
        methods.add_method_mut("set_particle_size_dist", |_, gen, (width, height): (Dist, Dist)| {
            gen.model.particle_size_dist = Some((width, height));
            Ok(())
        });
        methods.add_method_mut("set_particle_frame_time_offset_dist", |_, gen, value: Dist| {
            gen.model.particle_frame_time_offset_dist = Some(value);
            Ok(())
        });
    }
}

fn dist_param(_lua: &Lua, _: &ScriptParticleGenerator,
              (value, dt, d2t, d3t) : (Dist, Option<Dist>, Option<Dist>, Option<Dist>)) -> Result<DistParam> {
    if dt.is_none() {
        Ok(DistParam::new(value, Dist::create_fixed(0.0), Dist::create_fixed(0.0), Dist::create_fixed(0.0)))
    } else if d2t.is_none() {
        Ok(DistParam::new(value, dt.unwrap(), Dist::create_fixed(0.0), Dist::create_fixed(0.0)))
    } else if d3t.is_none() {
        Ok(DistParam::new(value, dt.unwrap(), d2t.unwrap(), Dist::create_fixed(0.0)))
    } else {
        Ok(DistParam::new(value, dt.unwrap(), d2t.unwrap(), d3t.unwrap()))
    }
}

pub fn param<T>(_lua: &Lua, _: &T,
         (value, dt, d2t, d3t): (f32, Option<f32>, Option<f32>, Option<f32>)) -> Result<Param> {
    if dt.is_none() {
        Ok(Param::fixed(value))
    } else if d2t.is_none() {
        Ok(Param::with_speed(value, dt.unwrap()))
    } else if d3t.is_none() {
        Ok(Param::with_accel(value, dt.unwrap(), d2t.unwrap()))
    } else {
        Ok(Param::with_jerk(value, dt.unwrap(), d2t.unwrap(), d3t.unwrap()))
    }
}

fn activate(_lua: &Lua, gen: &ScriptParticleGenerator, _args: ()) -> Result<()> {
    let pgen = create_pgen(gen, gen.model.clone())?;

    GameState::add_animation(pgen);

    Ok(())
}

pub fn create_surface_pgen(gen: &ScriptParticleGenerator, x: i32, y: i32) -> Result<Anim> {
    let mut model = gen.model.clone();
    let x_param = model.position.0.offset(x as f32);
    let y_param = model.position.1.offset(y as f32);
    model.position = (x_param, y_param);

    create_pgen(gen, model)
}

pub fn create_pgen(gen: &ScriptParticleGenerator, model: GeneratorModel) -> Result<Anim> {
    let mgr = GameState::turn_manager();
    let parent = mgr.borrow().entity(gen.parent);

    let image = match ResourceSet::get_image(&gen.image) {
        Some(image) => image,
        None => {
            warn!("Unable to locate image '{}' for particle generator", gen.image);
            return Err(rlua::Error::FromLuaConversionError {
                from: "ScriptParticleGenerator",
                to: "ParticleGenerator",
                message: Some("Image not found".to_string()),
            });
        }
    };

    let mut pgen = animation::particle_generator::new(&parent, image, model);

    if let Some(ref cb) = gen.completion_callback {
        pgen.add_completion_callback(Box::new(cb.clone()));
    }

    for &(time, ref cb) in gen.callbacks.iter() {
        pgen.add_update_callback(Box::new(cb.clone()), (time * 1000.0) as u32);
    }

    Ok(pgen)
}
