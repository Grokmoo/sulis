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

use std::rc::Rc;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use sulis_core::resource::ResourceSet;
use sulis_core::util::ExtInt;
use sulis_module::ImageLayer;
use crate::{EntityState};
use crate::animation::{Anim, AnimKind,
    particle_generator::{GeneratorModel, GeneratorState, Param, Particle}};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AnimSaveState {
    kind: Kind,
    elapsed: u32,
    duration_millis: ExtInt,
    owner: usize,
    removal_effect: Option<usize>,
}

impl AnimSaveState {
    pub fn new(anim: &Anim) -> AnimSaveState {
        let kind = match &anim.kind {
            AnimKind::EntityColor { color, color_sec } => {
                Kind::EntityColor {
                    color: color.clone(),
                    color_sec: color_sec.clone(),
                }
            },
            AnimKind::EntityScale { scale } => {
                Kind::EntityScale {
                    scale: scale.clone(),
                }
            },
            AnimKind::EntityImageLayer { images } => {
                let mut imgs = HashMap::new();
                for (layer, image) in images.iter() {
                    imgs.insert(*layer, image.id());
                }
                Kind::EntityImageLayer {
                    images: imgs
                }
            },
            AnimKind::ParticleGenerator { model, state } => {
                let particles = if model.gen_rate.is_non_zero() {
                    // particle generator is generating new particles
                    Vec::new()
                } else {
                    // particle generator has a fixed set of particles that must be saved
                    state.particles.clone()
                };

                let state = GeneratorSaveState {
                    image: state.image.id(),
                    particles,
                    gen_overflow: state.gen_overflow,
                    previous_secs: state.previous_secs,
                };

                Kind::ParticleGenerator {
                    model: model.clone(),
                    state
                }
            },
            _ => {
                warn!("Attempted to serialize invalid anim kind");
                Kind::Invalid
            }
        };

        AnimSaveState {
            kind,
            elapsed: anim.elapsed,
            duration_millis: anim.duration_millis,
            owner: anim.owner.borrow().index(),
            removal_effect: anim.removal_effect,
        }
    }

    pub fn load(self, entities: &HashMap<usize, Rc<RefCell<EntityState>>>,
                effects: &HashMap<usize, usize>,
                marked: &mut HashMap<usize, Vec<Rc<Cell<bool>>>>) -> Option<Anim> {
        let entity = match entities.get(&self.owner) {
            None => {
                warn!("Invalid owner for animation {}", self.owner);
                return None;
            }, Some(ref entity) => Rc::clone(entity),
        };

        let mut anim = match self.kind {
            Kind::Invalid => return None,
            Kind::EntityColor { color, color_sec } => {
                Anim::new_entity_color(&entity, self.duration_millis, color, color_sec)
            },
            Kind::EntityScale { scale } => {
                Anim::new_entity_scale(&entity, self.duration_millis, scale)
            },
            Kind::EntityImageLayer { images } => {
                let mut imgs = HashMap::new();
                for (layer, image) in images {
                    let img = match ResourceSet::get_image(&image) {
                        None => continue,
                        Some(image) => image,
                    };
                    imgs.insert(layer, img);
                }
                Anim::new_entity_image_layer(&entity, self.duration_millis, imgs)
            },
            Kind::ParticleGenerator { model, state } => {
                let image = match ResourceSet::get_image(&state.image) {
                    None => {
                        warn!("Invalid image for animation {}", state.image);
                        return None;
                    }, Some(image) => image,
                };

                let state = GeneratorState {
                    image,
                    particles: state.particles,
                    gen_overflow: state.gen_overflow,
                    previous_secs: state.previous_secs,
                };

                Anim::new_pgen(&entity, self.duration_millis, model, state)
            }
        };

        anim.elapsed = self.elapsed;

        if let Some(index) = self.removal_effect {
            let new_index = match effects.get(&index) {
                None => {
                    warn!("Invalid removal effect for animation {}", index);
                    return None;
                }, Some(index) => *index,
            };

            anim.removal_effect = Some(new_index);

            let vec = marked.entry(new_index).or_insert(Vec::new());
            vec.push(anim.get_marked_for_removal());
        }

        Some(anim)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
enum Kind {
    Invalid,
    EntityColor { color: [Param; 4], color_sec: [Param; 4] },
    ParticleGenerator { model: GeneratorModel, state: GeneratorSaveState },
    EntityScale { scale: Param },
    EntityImageLayer { images: HashMap<ImageLayer, String> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct GeneratorSaveState {
    image: String,
    particles: Vec<Particle>,
    gen_overflow: f32,
    previous_secs: f32,
}
