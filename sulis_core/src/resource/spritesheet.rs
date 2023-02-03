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

use std::collections::HashMap;
use std::io::Error;
use std::path::PathBuf;
use std::rc::Rc;

use crate::image::SimpleImage;
use crate::resource::ResourceSet;
use crate::util::{unable_to_create_error, Point, Size};

use crate::extern_image::{self, ImageBuffer, Rgba};

#[derive(Debug)]
pub struct Spritesheet {
    pub id: String,
    pub image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pub sprites: HashMap<String, Rc<Sprite>>,
}

#[derive(Debug)]
pub struct Sprite {
    pub sheet_id: String,
    pub sprite_id: String,
    pub position: Point,
    pub size: Size,
    pub tex_coords: [f32; 8],
}

impl Sprite {
    fn new(
        sheet_id: &str,
        sprite_id: &str,
        image_size: Size,
        position: Point,
        size: Size,
    ) -> Sprite {
        let image_width = image_size.width as f32;
        let image_height = image_size.height as f32;
        let x_min = (position.x as f32) / image_width;
        let y_min = (image_height - (position.y + size.height) as f32) / image_height;
        let x_max = (position.x + size.width) as f32 / image_width;
        let y_max = (image_height - position.y as f32) / image_height;

        Sprite {
            sheet_id: sheet_id.to_string(),
            sprite_id: sprite_id.to_string(),
            position,
            size,
            tex_coords: [x_min, y_max, x_min, y_min, x_max, y_max, x_max, y_min],
        }
    }

    pub fn full_id(&self) -> String {
        format!("{}/{}", self.sheet_id, self.sprite_id)
    }

    pub fn spritesheet(&self) -> Rc<Spritesheet> {
        ResourceSet::spritesheet(&self.sheet_id).unwrap()
    }
}

impl Spritesheet {
    pub fn new(
        builder: SpritesheetBuilder,
        resources: &mut ResourceSet,
    ) -> Result<Rc<Spritesheet>, Error> {
        let mut image = None;
        for dir in builder.source_dirs.iter().rev() {
            let mut filepath = PathBuf::from(dir);
            filepath.push(&builder.src);

            if let Ok(read_image) = extern_image::open(&filepath) {
                image = Some(read_image);
                break;
            }
        }

        let image = match image {
            None => {
                warn!(
                    "Unable to read spritesheet source '{}' from any of '{:?}'",
                    builder.src, builder.source_dirs
                );
                return unable_to_create_error("spritesheet", &builder.id);
            }
            Some(img) => img,
        };

        let image = image.to_rgba8();
        let (image_width, image_height) = image.dimensions();
        let image_size = Size::new(image_width as i32, image_height as i32);
        let multiplier = builder.grid_multiplier.unwrap_or(1) as i32;

        let mut sprites: HashMap<String, Rc<Sprite>> = HashMap::new();
        for (_, group) in builder.groups {
            let mut template: Option<SpritesheetGroupTemplate> = match group.from_template {
                None => None,
                Some(ref id) => {
                    let templates = match builder.templates {
                        None => {
                            warn!("Template '{}' not found", id);
                            continue;
                        }
                        Some(ref templates) => templates,
                    };

                    match templates.get(id) {
                        None => {
                            warn!("Template '{}' not found", id);
                            continue;
                        }
                        Some(template) => Some(template.clone()),
                    }
                }
            };

            let multiplier = group.grid_multiplier.unwrap_or(multiplier as u32) as i32;

            let base_size = match template {
                None => group.size,
                Some(ref template) => template.size,
            };

            let base_pos = group.position;

            let mut areas: HashMap<String, Vec<i32>> = group.areas.into_iter().collect();

            if let Some(template) = template.as_mut() {
                template.areas.drain().for_each(|(k, v)| {
                    areas.insert(k, v);
                });
            }

            for (base_id, area_pos) in areas {
                let id = match group.prefix {
                    None => base_id,
                    Some(ref prefix) => format!("{prefix}{base_id}"),
                };

                let (mut pos, mut size) = match area_pos.len() {
                    2 => (base_pos.add(area_pos[0], area_pos[1]), base_size),
                    4 => (
                        base_pos.add(area_pos[0], area_pos[1]),
                        base_size.add(area_pos[2], area_pos[3]),
                    ),
                    _ => {
                        warn!(
                            "Error in definition for sprite '{}' in sheet '{}'",
                            id, builder.id
                        );
                        warn!("Coordinates must either by [x, y] or [x, y, w, h]");
                        continue;
                    }
                };

                pos.mult_mut(multiplier);
                size.mult_mut(multiplier);
                trace!(
                    "Creating sprite with id '{}' in '{}', {:?}",
                    id,
                    builder.id,
                    size
                );
                let sprite = Sprite::new(&builder.id, &id, image_size, pos, size);

                if sprites.contains_key(&id) {
                    warn!("Duplicate sprite ID in sheet '{}': '{}'", builder.id, id);
                    continue;
                }

                let upper_bound_pos = sprite.position.add(sprite.size.width, sprite.size.height);

                if !sprite
                    .position
                    .in_bounds(image_width as i32 + 1, image_height as i32 + 1)
                    || !upper_bound_pos.in_bounds(image_width as i32 + 1, image_height as i32 + 1)
                {
                    warn!(
                        "Sprite '{}' in sheet '{}' coordinates fall outside image bounds",
                        id, builder.id
                    );
                    continue;
                }

                let sprite = Rc::new(sprite);

                // default to group scale, then fallback to overall sheet scale
                // if neither are defined, don't gen images
                let mut scale = group.simple_image_gen_scale;
                if scale.is_none() {
                    scale = builder.simple_image_gen_scale;
                }
                if let Some(scale) = scale {
                    let scale = scale as i32;
                    let full_id = format!("{}/{}", builder.id, id);
                    let simple_image = SimpleImage {
                        id: full_id.clone(),
                        size: Size::new(sprite.size.width / scale, sprite.size.height / scale),
                        image_display: Rc::clone(&sprite),
                    };
                    trace!("Generated image with ID '{}' for sprite.", full_id);
                    resources.images.insert(full_id, Rc::new(simple_image));
                }

                sprites.insert(id, sprite);
            }
        }

        Ok(Rc::new(Spritesheet {
            id: builder.id,
            image,
            sprites,
        }))
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SpritesheetBuilder {
    pub source_dirs: Vec<String>,
    pub id: String,
    pub src: String,
    pub size: Size,
    pub simple_image_gen_scale: Option<u32>,
    pub grid_multiplier: Option<u32>,
    groups: HashMap<String, SpritesheetGroup>,
    templates: Option<HashMap<String, SpritesheetGroupTemplate>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct SpritesheetGroupTemplate {
    pub size: Size,
    pub areas: HashMap<String, Vec<i32>>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct SpritesheetGroup {
    #[serde(default)]
    pub size: Size,

    #[serde(default)]
    pub position: Point,
    pub prefix: Option<String>,

    #[serde(default)]
    pub areas: HashMap<String, Vec<i32>>,
    pub from_template: Option<String>,
    pub simple_image_gen_scale: Option<u32>,
    pub grid_multiplier: Option<u32>,
}
