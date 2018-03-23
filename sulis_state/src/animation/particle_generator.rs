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

use rlua::{UserData};
use rand::{self, Rng};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use sulis_core::resource::Sprite;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::Widget;
use sulis_core::util;

use {animation, EntityState, ScriptCallback};

#[derive(Clone)]
pub enum Dist {
    FixedDist { value: f32 },
    UniformDist { min: f32, max: f32 },
}

impl UserData for Dist { }

impl Dist {
    pub fn create_fixed(value: f32) -> Dist {
        Dist::FixedDist { value }
    }

    pub fn create_uniform(min: f32, max: f32) -> Dist {
        if max > min {
            Dist::UniformDist { min, max }
        } else {
            Dist::UniformDist { min: max, max: min }
        }
    }

    fn generate(&self) -> f32 {
        match self {
            &Dist::FixedDist { value } => value,
            &Dist::UniformDist { min, max } => rand::thread_rng().gen_range(min, max),
        }
    }
}

#[derive(Clone)]
pub struct DistParam {
    value: Dist,
    dt: Dist,
    d2t: Dist,
}

impl DistParam {
    pub fn new(value: Dist, dt: Dist, d2t: Dist) -> DistParam {
        DistParam { value, dt, d2t }
    }
}

impl UserData for DistParam { }

#[derive(Clone)]
pub struct Param {
    value: f32,
    dt: f32,
    d2t: f32,
}

impl UserData for Param { }

impl Param {
    pub fn fixed(value: f32) -> Param {
        Param {
            value, dt: 0.0, d2t: 0.0,
        }
    }

    pub fn with_speed(value: f32, speed: f32) -> Param {
        Param {
            value, dt: speed, d2t: 0.0,
        }
    }

    pub fn with_accel(value: f32, speed: f32, accel: f32) -> Param {
        Param {
            value, dt: speed, d2t: accel,
        }
    }

    fn update(&mut self, elapsed_secs: f32) {
        self.value += self.dt * elapsed_secs + self.d2t * elapsed_secs * elapsed_secs;
    }
}

struct Particle {
    position: (Param, Param),
    remaining_duration: f32,
    width: f32,
    height: f32,
}

impl Particle {
    fn update(&mut self, elapsed_secs: f32) -> bool {
        self.position.0.update(elapsed_secs);
        self.position.1.update(elapsed_secs);

        self.remaining_duration -= elapsed_secs;

        self.remaining_duration < 0.0
    }
}

pub struct ParticleGenerator {
    sprite: Rc<Sprite>,
    owner: Rc<RefCell<EntityState>>,
    callback: Option<Box<ScriptCallback>>,
    start_time: Instant,
    duration_secs: f32,
    previous_secs: f32,
    position: (Param, Param),
    gen_rate: Param,
    gen_overflow: f32,

    particles: Vec<Particle>,

    particle_x_dist: Option<DistParam>,
    particle_y_dist: Option<DistParam>,
    particle_duration_dist: Option<Dist>,
    particle_size_dist: Option<(Dist, Dist)>
}

impl ParticleGenerator {
    pub fn new(owner: Rc<RefCell<EntityState>>, duration_millis: u32, sprite: Rc<Sprite>) -> ParticleGenerator {
        let x = owner.borrow().location.x as f32 + owner.borrow().size.width as f32 / 2.0;
        let y = owner.borrow().location.y as f32 + owner.borrow().size.height as f32 / 2.0;
        ParticleGenerator {
            owner,
            sprite,
            callback: None,
            start_time: Instant::now(),
            duration_secs: duration_millis as f32 / 1000.0,
            previous_secs: 0.0,

            position: (Param::fixed(x), Param::fixed(y)),
            gen_rate: Param::fixed(1.0),
            gen_overflow: 0.0,
            particles: Vec::new(),
            particle_x_dist: None,
            particle_y_dist: None,
            particle_duration_dist: None,
            particle_size_dist: None,
        }
    }

    pub fn set_gen_rate(&mut self, rate: Param) {
        self.gen_rate = rate;
    }

    pub fn set_position(&mut self, x: Param, y: Param) {
        self.position = (x, y);
    }

    pub fn set_particle_size_dist(&mut self, width: Dist, height: Dist) {
        self.particle_size_dist = Some((width, height));
    }

    pub fn set_particle_x_dist(&mut self, dist: DistParam) {
        self.particle_x_dist = Some(dist);
    }

    pub fn set_particle_y_dist(&mut self, dist: DistParam) {
        self.particle_y_dist = Some(dist);
    }

    pub fn set_particle_duration_dist(&mut self, dist: Dist) {
        self.particle_duration_dist = Some(dist);
    }

    fn generate_particle(&self) -> Particle {
        let mut position = self.position.clone(); // inherit position from generator

        if let Some(ref dist) = self.particle_x_dist {
            position.0.value += dist.value.generate();
            position.0.dt += dist.dt.generate();
            position.0.d2t += dist.d2t.generate();
        }

        if let Some(ref dist) = self.particle_y_dist {
            position.1.value += dist.value.generate();
            position.1.dt += dist.dt.generate();
            position.1.d2t += dist.d2t.generate();
        }

        let remaining_duration = match self.particle_duration_dist {
            None => self.duration_secs,
            Some(ref dist) => dist.generate(),
        };

        let (width, height) = match self.particle_size_dist.as_ref() {
            None => (1.0, 1.0),
            Some(&(ref width, ref height)) => (width.generate(), height.generate()),
        };

        Particle {
            position,
            remaining_duration,
            width,
            height,
        }
    }
}

impl animation::Animation for ParticleGenerator {
    fn update(&mut self, _root: &Rc<RefCell<Widget>>) -> bool {
        let millis = util::get_elapsed_millis(self.start_time.elapsed());
        let secs = millis as f32 / 1000.0;

        let frame_time_secs = secs - self.previous_secs;

        let num_to_gen = self.gen_rate.value * frame_time_secs + self.gen_overflow;

        self.gen_overflow = num_to_gen.fract();

        for _ in 0..(num_to_gen.trunc() as i32) {
            let particle = self.generate_particle();
            self.particles.push(particle);
        }

        self.gen_rate.update(frame_time_secs);
        self.position.0.update(frame_time_secs);
        self.position.1.update(frame_time_secs);

        let mut i = self.particles.len();
        loop {
            if i == 0 { break; }

            i -= 1;

            let remove = self.particles[i].update(frame_time_secs);

            if remove {
                self.particles.remove(i);
            }
        }

        self.previous_secs = secs;
        if secs < self.duration_secs {
            true
        } else {
            if let Some(ref cb) = self.callback {
                cb.on_anim_complete();
            }
            false
        }
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, offset_x: f32, offset_y: f32,
                          scale_x: f32, scale_y: f32, _millis: u32) {
        let mut draw_list = DrawList::empty_sprite();
        for particle in self.particles.iter() {
            let x = particle.position.0.value + offset_x;
            let y = particle.position.1.value + offset_y;
            let w = particle.width;
            let h = particle.height;
            draw_list.append(&mut DrawList::from_sprite_f32(&self.sprite, x, y, w, h));
        }

        if !draw_list.is_empty() {
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }
    }

    fn set_callback(&mut self, callback: Option<Box<ScriptCallback>>) {
        self.callback = callback;
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.owner
    }
}
