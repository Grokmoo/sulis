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

use rlua::UserData;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::{animation::Anim, EntityState};
use sulis_core::config::Config;
use sulis_core::image::Image;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{animation_state, Color};
use sulis_core::util::{approx_eq, gen_rand, ExtInt, Offset, Rect, Scale};

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(val: &f32) -> bool {
    *val == 0.0
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum Coord {
    X,
    Y,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum Dist {
    Fixed {
        value: f32,
    },
    Uniform {
        min: f32,
        max: f32,
    },
    FixedAngleUniformSpeed {
        angle: f32,
        min_speed: f32,
        max_speed: f32,
    },
    UniformAngleFixedSpeed {
        min_angle: f32,
        max_angle: f32,
        speed: f32,
    },
    UniformAngleUniformSpeed {
        min_angle: f32,
        max_angle: f32,
        min_speed: f32,
        max_speed: f32,
    },
}

impl UserData for Dist {}

fn swap_if_backwards(min: f32, max: f32) -> (f32, f32) {
    if min >= max {
        (max, min)
    } else {
        (min, max)
    }
}

fn force_normal(val: f32) -> f32 {
    if val.is_normal() {
        val
    } else {
        0.0
    }
}

impl Dist {
    pub fn create_fixed(value: f32) -> Dist {
        let value = force_normal(value);
        Dist::Fixed { value }
    }

    pub fn create_uniform(min: f32, max: f32) -> Dist {
        let min = force_normal(min);
        let max = force_normal(max);
        if approx_eq(min, max) {
            Dist::Fixed { value: min }
        } else {
            let (min, max) = swap_if_backwards(min, max);
            Dist::Uniform { min, max }
        }
    }

    pub fn create_angular(min_angle: f32, max_angle: f32, min_speed: f32, max_speed: f32) -> Dist {
        let (min_speed, max_speed) = (force_normal(min_speed), force_normal(max_speed));
        let (min_angle, max_angle) = (force_normal(min_angle), force_normal(max_angle));
        let (min_speed, max_speed) = swap_if_backwards(min_speed, max_speed);
        let (min_angle, max_angle) = swap_if_backwards(min_angle, max_angle);

        if approx_eq(min_angle, max_angle) && approx_eq(min_speed, max_speed) {
            warn!("Creating invalid angular distribution with both speed and angle fixed");
            Dist::Fixed { value: min_speed }
        } else if approx_eq(min_angle, max_angle) {
            Dist::FixedAngleUniformSpeed {
                angle: min_angle,
                min_speed,
                max_speed,
            }
        } else if approx_eq(min_speed, max_speed) {
            Dist::UniformAngleFixedSpeed {
                min_angle,
                max_angle,
                speed: min_speed,
            }
        } else {
            Dist::UniformAngleUniformSpeed {
                min_angle,
                max_angle,
                min_speed,
                max_speed,
            }
        }
    }

    fn generate_pair(&self) -> (f32, f32) {
        match self {
            Dist::Fixed { value } => (*value, *value),
            Dist::Uniform { min, max } => (gen_rand(*min, *max), gen_rand(*min, *max)),
            Dist::FixedAngleUniformSpeed {
                angle,
                min_speed,
                max_speed,
            } => {
                let speed = gen_rand(*min_speed, *max_speed);
                radial_to_cart(*angle, speed)
            }
            Dist::UniformAngleFixedSpeed {
                min_angle,
                max_angle,
                speed,
            } => {
                let angle = gen_rand(*min_angle, *max_angle);
                radial_to_cart(angle, *speed)
            }
            Dist::UniformAngleUniformSpeed {
                min_angle,
                max_angle,
                min_speed,
                max_speed,
            } => {
                let speed = gen_rand(*min_speed, *max_speed);
                let angle = gen_rand(*min_angle, *max_angle);
                radial_to_cart(angle, speed)
            }
        }
    }

    fn generate(&self) -> f32 {
        match self {
            Dist::Fixed { value } => *value,
            Dist::Uniform { min, max } => gen_rand(*min, *max),
            _ => {
                warn!("2D dists should only be used as the sole dist in a position component");
                0.0
            }
        }
    }
}

fn radial_to_cart(angle: f32, mag: f32) -> (f32, f32) {
    let (y, x) = angle.sin_cos();
    (x * mag, y * mag)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DistParam {
    value: Dist,
    dt: Dist,
    d2t: Dist,
    d3t: Dist,
}

impl DistParam {
    pub fn new(value: Dist, dt: Dist, d2t: Dist, d3t: Dist) -> DistParam {
        DistParam {
            value,
            dt,
            d2t,
            d3t,
        }
    }
}

impl UserData for DistParam {}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct DistParam2D {
    x: DistParam,
    y: Option<DistParam>,
}

impl DistParam2D {
    pub fn new(x: DistParam, y: Option<DistParam>) -> DistParam2D {
        DistParam2D { x, y }
    }
}

impl UserData for DistParam2D {}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Param {
    #[serde(default, skip_serializing_if = "is_zero")]
    initial_value: f32,

    #[serde(default, skip_serializing_if = "is_zero")]
    dt: f32,

    #[serde(default, skip_serializing_if = "is_zero")]
    d2t: f32,

    #[serde(default, skip_serializing_if = "is_zero")]
    d3t: f32,

    #[serde(default, skip_serializing_if = "is_zero")]
    pub value: f32,
}

impl UserData for Param {}

impl Param {
    pub fn is_non_zero(&self) -> bool {
        self.initial_value != 0.0 || self.dt != 0.0 || self.d2t != 0.0 || self.d3t != 0.0
    }

    pub fn offset(&self, offset: f32) -> Param {
        let value = self.value + offset;
        Param {
            initial_value: value,
            value,
            dt: self.dt,
            d2t: self.d2t,
            d3t: self.d3t,
        }
    }

    pub fn fixed(value: f32) -> Param {
        let initial_value = value;
        Param {
            initial_value,
            value,
            dt: 0.0,
            d2t: 0.0,
            d3t: 0.0,
        }
    }

    pub fn with_speed(value: f32, speed: f32) -> Param {
        let initial_value = value;
        Param {
            initial_value,
            value,
            dt: speed,
            d2t: 0.0,
            d3t: 0.0,
        }
    }

    pub fn with_accel(value: f32, speed: f32, accel: f32) -> Param {
        let initial_value = value;
        Param {
            initial_value,
            value,
            dt: speed,
            d2t: accel,
            d3t: 0.0,
        }
    }

    pub fn with_jerk(value: f32, speed: f32, accel: f32, jerk: f32) -> Param {
        let initial_value = value;
        Param {
            initial_value,
            value,
            dt: speed,
            d2t: accel,
            d3t: jerk,
        }
    }

    pub fn update(&mut self, v_term: f32, a_term: f32, j_term: f32) {
        self.value = self.initial_value + self.dt * v_term + self.d2t * a_term + self.d3t * j_term;
    }
}

pub(in crate::animation) fn update(
    owner: &Rc<RefCell<EntityState>>,
    model: &mut GeneratorModel,
    state: &mut GeneratorState,
    marked_for_removal: &Rc<Cell<bool>>,
    millis: u32,
) {
    if model.moves_with_parent && owner.borrow().actor.is_dead() {
        marked_for_removal.set(true);
        return;
    }

    let secs = millis as f32 / 1000.0;
    let frame_time_secs = secs - state.previous_secs;

    let num_to_gen = model.gen_rate.value * frame_time_secs + state.gen_overflow;

    state.gen_overflow = num_to_gen.fract();

    for _ in 0..(num_to_gen.trunc() as i32) {
        let particle = model.generate_particle();
        state.particles.push(particle);
    }

    let v_term = secs;
    let a_term = secs * secs;
    let j_term = secs * secs * secs;

    model.gen_rate.update(v_term, a_term, j_term);
    model.position.0.update(v_term, a_term, j_term);
    model.position.1.update(v_term, a_term, j_term);
    model.red.update(v_term, a_term, j_term);
    model.green.update(v_term, a_term, j_term);
    model.blue.update(v_term, a_term, j_term);
    model.alpha.update(v_term, a_term, j_term);

    if let Some(ref mut rotation) = model.rotation {
        rotation.update(v_term, a_term, j_term);
    }

    if let Some(ref mut centroid) = model.centroid {
        centroid.0.update(v_term, a_term, j_term);
        centroid.1.update(v_term, a_term, j_term);
    }

    let mut i = state.particles.len();
    loop {
        if i == 0 {
            break;
        }

        i -= 1;

        let remove = state.particles[i].update(frame_time_secs);

        if remove {
            state.particles.remove(i);
        }
    }

    state.previous_secs = secs;
}

pub(in crate::animation) fn draw(
    state: &GeneratorState,
    model: &GeneratorModel,
    owner: &Rc<RefCell<EntityState>>,
    renderer: &mut dyn GraphicsRenderer,
    offset: Offset,
    scale: Scale,
    _millis: u32,
) {
    let (offset_x, offset_y) = if model.moves_with_parent {
        let parent = owner.borrow();
        let x = parent.location.x as f32 + parent.size.width as f32 / 2.0 + parent.sub_pos.0;
        let y = parent.location.y as f32 + parent.size.height as f32 / 2.0 + parent.sub_pos.1;
        (x + offset.x, y + offset.y)
    } else {
        (offset.x, offset.y)
    };

    let mut draw_list = DrawList::empty_sprite();
    for particle in state.particles.iter() {
        let rect = Rect {
            x: particle.position.0.value + offset_x,
            y: particle.position.1.value + offset_y,
            w: particle.width,
            h: particle.height,
        };
        let millis = (particle.current_duration * 1000.0) as u32;
        state
            .image
            .append_to_draw_list(&mut draw_list, &animation_state::NORMAL, rect, millis);
    }

    if !draw_list.is_empty() {
        draw_list.set_scale(scale);
        draw_list.set_color(Color::new(
            model.red.value,
            model.green.value,
            model.blue.value,
            model.alpha.value,
        ));
        if let Some(ref rotation) = model.rotation {
            if let Some(ref centroid) = model.centroid {
                let x = centroid.0.value + offset_x;
                let y = centroid.1.value + offset_y;
                let y = Config::ui_height() as f32 - y;
                draw_list.centroid = Some([x, y]);
            }

            draw_list.rotate(rotation.value);
        }
        renderer.draw(draw_list);
    }
}

pub fn new(owner: &Rc<RefCell<EntityState>>, image: Rc<dyn Image>, model: GeneratorModel) -> Anim {
    let state = GeneratorState {
        image,
        particles: Vec::new(),
        gen_overflow: model.initial_overflow,
        previous_secs: 0.0,
    };

    Anim::new_pgen(owner, model.duration_millis, model, state)
}
pub(in crate::animation) struct GeneratorState {
    pub(in crate::animation) image: Rc<dyn Image>,
    pub(in crate::animation) particles: Vec<Particle>,
    pub(in crate::animation) gen_overflow: f32,
    pub(in crate::animation) previous_secs: f32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct GeneratorModel {
    pub position: (Param, Param),

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Param>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub centroid: Option<(Param, Param)>,

    pub red: Param,
    pub green: Param,
    pub blue: Param,
    pub alpha: Param,
    pub moves_with_parent: bool,
    pub duration_millis: ExtInt,
    pub gen_rate: Param,
    pub initial_overflow: f32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_position_dist: Option<DistParam2D>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_duration_dist: Option<Dist>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_frame_time_offset_dist: Option<Dist>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_size_dist: Option<(Dist, Dist)>,

    pub draw_above_entities: bool,

    pub is_blocking: bool,
}

impl UserData for GeneratorModel {}

impl GeneratorModel {
    pub fn new(duration_millis: ExtInt, x: f32, y: f32) -> GeneratorModel {
        let blocking = !duration_millis.is_infinite();

        GeneratorModel {
            duration_millis,
            position: (Param::fixed(x), Param::fixed(y)),
            rotation: None,
            centroid: None,
            red: Param::fixed(1.0),
            green: Param::fixed(1.0),
            blue: Param::fixed(1.0),
            alpha: Param::fixed(1.0),
            moves_with_parent: false,
            gen_rate: Param::fixed(0.0),
            initial_overflow: 0.0,
            particle_position_dist: None,
            particle_duration_dist: None,
            particle_frame_time_offset_dist: None,
            particle_size_dist: None,
            draw_above_entities: true,
            is_blocking: blocking,
        }
    }

    fn generate_particle(&self) -> Particle {
        let mut position = self.position; // inherit position from generator
        position.0.initial_value = position.0.value;
        position.1.initial_value = position.1.value;

        if let Some(ref dist2d) = self.particle_position_dist {
            match dist2d.y {
                None => {
                    let value = dist2d.x.value.generate_pair();
                    let dt = dist2d.x.dt.generate_pair();
                    let d2t = dist2d.x.d2t.generate_pair();
                    let d3t = dist2d.x.d3t.generate_pair();
                    position.0.initial_value += value.0;
                    position.0.dt += dt.0;
                    position.0.d2t += d2t.0;
                    position.0.d3t += d3t.0;
                    position.1.initial_value += value.1;
                    position.1.dt += dt.1;
                    position.1.d2t += d2t.1;
                    position.1.d3t += d3t.1;
                }
                Some(ref y) => {
                    let x = &dist2d.x;
                    position.0.initial_value += x.value.generate();
                    position.0.dt += x.dt.generate();
                    position.0.d2t += x.d2t.generate();
                    position.0.d3t += x.d3t.generate();

                    position.1.initial_value += y.value.generate();
                    position.1.dt += y.dt.generate();
                    position.1.d2t += y.d2t.generate();
                    position.1.d3t += y.d3t.generate();
                }
            }
        }

        let total_duration = match self.particle_duration_dist {
            None => self.duration_millis.to_f32() * 1000.0,
            Some(ref dist) => dist.generate(),
        };

        let (width, height) = match self.particle_size_dist.as_ref() {
            None => (1.0, 1.0),
            Some(&(ref width, ref height)) => (width.generate(), height.generate()),
        };

        let initial_duration = match self.particle_frame_time_offset_dist.as_ref() {
            None => 0.0,
            Some(ref dist) => dist.generate(),
        };

        Particle {
            position,
            total_duration,
            current_duration: initial_duration,
            width,
            height,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(in crate::animation) struct Particle {
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
        let a_term = v_term * v_term;
        let j_term = a_term * v_term;

        self.position.0.update(v_term, a_term, j_term);
        self.position.1.update(v_term, a_term, j_term);

        self.current_duration > self.total_duration
    }
}
