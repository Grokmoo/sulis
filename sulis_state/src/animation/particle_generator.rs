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

use std::cmp::Ordering;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use sulis_core::image::Image;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{animation_state, Color, Widget};
use sulis_core::util;

use {animation, ChangeListener, Effect, EntityState, ScriptCallback};

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
        if min == max {
            Dist::FixedDist { value: min }
        } else if max > min {
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
    d3t: Dist,
}

impl DistParam {
    pub fn new(value: Dist, dt: Dist, d2t: Dist, d3t: Dist) -> DistParam {
        DistParam { value, dt, d2t, d3t }
    }
}

impl UserData for DistParam { }

#[derive(Clone)]
pub struct Param {
    initial_value: f32,
    dt: f32,
    d2t: f32,
    d3t: f32,

    value: f32,
}

impl UserData for Param { }

impl Param {
    pub fn fixed(value: f32) -> Param {
        let initial_value = value;
        Param {
            initial_value, value, dt: 0.0, d2t: 0.0, d3t: 0.0,
        }
    }

    pub fn with_speed(value: f32, speed: f32) -> Param {
        let initial_value = value;
        Param {
            initial_value, value, dt: speed, d2t: 0.0, d3t: 0.0,
        }
    }

    pub fn with_accel(value: f32, speed: f32, accel: f32) -> Param {
        let initial_value = value;
        Param {
            initial_value, value, dt: speed, d2t: accel, d3t: 0.0,
        }
    }

    pub fn with_jerk(value: f32, speed: f32, accel: f32, jerk: f32) -> Param {
        let initial_value = value;
        Param {
           initial_value, value, dt: speed, d2t: accel, d3t: jerk,
        }
    }

    fn update(&mut self, v_term: f32, a_term: f32, j_term: f32) {
        self.value = self.initial_value + self.dt * v_term + self.d2t * a_term + self.d3t * j_term;
    }
}

#[derive(Clone)]
pub struct GeneratorModel {
    pub position: (Param, Param),
    pub red: Param,
    pub green: Param,
    pub blue: Param,
    pub alpha: Param,
    pub moves_with_parent: bool,
    pub duration_secs: f32,
    pub gen_rate: Param,
    pub initial_overflow: f32,
    pub particle_x_dist: Option<DistParam>,
    pub particle_y_dist: Option<DistParam>,
    pub particle_duration_dist: Option<Dist>,
    pub particle_size_dist: Option<(Dist, Dist)>,
}

impl UserData for GeneratorModel { }

impl GeneratorModel {
    pub fn new(duration_secs: f32, x: f32, y: f32) -> GeneratorModel {
        GeneratorModel {
            duration_secs,
            position: (Param::fixed(x), Param::fixed(y)),
            red: Param::fixed(1.0),
            green: Param::fixed(1.0),
            blue: Param::fixed(1.0),
            alpha: Param::fixed(1.0),
            moves_with_parent: false,
            gen_rate: Param::fixed(0.0),
            initial_overflow: 0.0,
            particle_x_dist: None,
            particle_y_dist: None,
            particle_duration_dist: None,
            particle_size_dist: None,
        }
    }
}

struct Particle {
    position: (Param, Param),
    total_duration: f32,
    current_duration: f32,
    width: f32,
    height: f32,
}

impl Particle {
    fn update(&mut self, frame_time: f32) -> bool {
        self.current_duration += frame_time;

        let v_term = self.current_duration;
        let a_term = 1.0 / 2.0 * v_term * v_term;
        let j_term = 1.0 / 3.0 * a_term * v_term;

        self.position.0.update(v_term, a_term, j_term);
        self.position.1.update(v_term, a_term, j_term);

        self.current_duration > self.total_duration
    }
}

pub struct ParticleGenerator {
    image: Rc<Image>,
    owner: Rc<RefCell<EntityState>>,
    start_time: Instant,
    previous_secs: f32,
    particles: Vec<Particle>,
    callbacks: Vec<(f32, Box<ScriptCallback>)>, //sorted by the first field which is time in seconds
    callback_index: usize,
    callback: Option<Box<ScriptCallback>>,
    gen_overflow: f32,
    marked_for_removal: Rc<RefCell<bool>>,

    model: GeneratorModel,
}

impl ParticleGenerator {
    pub fn new(owner: Rc<RefCell<EntityState>>, image: Rc<Image>,
               model: GeneratorModel) -> ParticleGenerator {
        trace!("Created new particle generator with particle '{}', duration {}",
               image.id(), model.duration_secs);
        let gen_overflow = model.initial_overflow;
        ParticleGenerator {
            owner,
            image,
            callback: None,
            callback_index: 1, // use lua 1 based indexing
            callbacks: Vec::new(),
            start_time: Instant::now(),
            previous_secs: 0.0,
            particles: Vec::new(),
            gen_overflow,
            model,
            marked_for_removal: Rc::new(RefCell::new(false)),
        }
    }

    pub fn add_callback(&mut self, callback: Box<ScriptCallback>, time_secs: f32) {
        self.callbacks.push((time_secs, callback));

        self.callbacks.sort_by(|a, b| {
            if a.0 < b.0 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });
    }

    pub fn add_removal_listener(&self, effect: &mut Effect) {
        let marked_for_removal = Rc::clone(&self.marked_for_removal);
        effect.removal_listeners.add(ChangeListener::new("particle_gen", Box::new(move |_| {
            *marked_for_removal.borrow_mut() = true;
        })));
    }

    fn generate_particle(&self) -> Particle {
        let mut position = self.model.position.clone(); // inherit position from generator
        position.0.initial_value = position.0.value;
        position.1.initial_value = position.1.value;

        if let Some(ref dist) = self.model.particle_x_dist {
            position.0.initial_value += dist.value.generate();
            position.0.dt += dist.dt.generate();
            position.0.d2t += dist.d2t.generate();
            position.0.d3t += dist.d3t.generate();
        }

        if let Some(ref dist) = self.model.particle_y_dist {
            position.1.initial_value += dist.value.generate();
            position.1.dt += dist.dt.generate();
            position.1.d2t += dist.d2t.generate();
            position.1.d3t += dist.d3t.generate();
        }

        let total_duration = match self.model.particle_duration_dist {
            None => self.model.duration_secs,
            Some(ref dist) => dist.generate(),
        };

        let (width, height) = match self.model.particle_size_dist.as_ref() {
            None => (1.0, 1.0),
            Some(&(ref width, ref height)) => (width.generate(), height.generate()),
        };

        Particle {
            position,
            total_duration,
            current_duration: 0.0,
            width,
            height,
        }
    }
}

impl animation::Animation for ParticleGenerator {
    fn update(&mut self, _root: &Rc<RefCell<Widget>>) -> bool {
        let secs = util::get_elapsed_millis(self.start_time.elapsed()) as f32 / 1000.0;
        let frame_time_secs = secs - self.previous_secs;

        let num_to_gen = self.model.gen_rate.value * frame_time_secs + self.gen_overflow;

        self.gen_overflow = num_to_gen.fract();

        for _ in 0..(num_to_gen.trunc() as i32) {
            let particle = self.generate_particle();
            self.particles.push(particle);
        }

        let v_term = secs;
        let a_term = 1.0 / 2.0 * secs * secs;
        let j_term = 1.0 / 6.0 * secs * secs * secs;

        self.model.gen_rate.update(v_term, a_term, j_term);
        self.model.position.0.update(v_term, a_term, j_term);
        self.model.position.1.update(v_term, a_term, j_term);
        self.model.red.update(v_term, a_term, j_term);
        self.model.green.update(v_term, a_term, j_term);
        self.model.blue.update(v_term, a_term, j_term);
        self.model.alpha.update(v_term, a_term, j_term);

        let mut i = self.particles.len();
        loop {
            if i == 0 { break; }

            i -= 1;

            let remove = self.particles[i].update(frame_time_secs);

            if remove {
                self.particles.remove(i);
            }
        }

        if !self.callbacks.is_empty() {
            if secs > self.callbacks[0].0 {
                self.callbacks[0].1.on_anim_update(self.callback_index);
                self.callbacks.remove(0);
                self.callback_index += 1;
            }
        }

        self.previous_secs = secs;
        if secs < self.model.duration_secs && !*self.marked_for_removal.borrow() {
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
        let (offset_x, offset_y) = if self.model.moves_with_parent {
            let parent = self.owner.borrow();
            let x = parent.location.x as f32 + parent.size.width as f32 / 2.0 + parent.sub_pos.0;
            let y = parent.location.y as f32 + parent.size.height as f32 / 2.0 + parent.sub_pos.1;
            (x + offset_x, y + offset_y)
        } else {
            (offset_x, offset_y)
        };

        let mut draw_list = DrawList::empty_sprite();
        for particle in self.particles.iter() {
            let x = particle.position.0.value + offset_x;
            let y = particle.position.1.value + offset_y;
            let w = particle.width;
            let h = particle.height;
            let millis = (particle.current_duration * 1000.0) as u32;
            self.image.append_to_draw_list(&mut draw_list, &animation_state::NORMAL,
                                           x, y, w, h, millis);
        }

        if !draw_list.is_empty() {
            draw_list.set_scale(scale_x, scale_y);
            draw_list.set_color(Color::new(self.model.red.value, self.model.green.value,
                                           self.model.blue.value, self.model.alpha.value));
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
