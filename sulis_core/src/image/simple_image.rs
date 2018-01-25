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
use std::io::{Error, ErrorKind};

use image::Image;
use resource::{ResourceBuilder, ResourceSet, Sprite};
use io::{DrawList, GraphicsRenderer};
use ui::AnimationState;
use util::Size;

use serde_json;
use serde_yaml;

#[derive(Debug)]
pub struct SimpleImage {
    id: String,
    image_display: Rc<Sprite>,
    size: Size,
}

impl SimpleImage {
    pub fn new(builder: SimpleImageBuilder, resources: &ResourceSet) -> Result<Rc<Image>, Error> {
        let sprite = resources.get_sprite_internal(&builder.image_display)?;

        Ok(Rc::new(SimpleImage {
            id: builder.id,
            size: builder.size,
            image_display: sprite,
        }))
    }
}

impl Image for SimpleImage {
    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, _state: &AnimationState,
                          x: f32, y: f32, w: f32, h: f32) {
        renderer.draw(DrawList::from_sprite_f32(&self.image_display, x, y, w, h));
    }

    fn get_width_f32(&self) -> f32 {
        self.size.width as f32
    }

    fn get_height_f32(&self) -> f32 {
        self.size.height as f32
    }

    fn get_size(&self) -> &Size {
        &self.size
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SimpleImageBuilder {
    id: String,
    image_display: String,
    size: Size,
}

impl ResourceBuilder for SimpleImageBuilder {
    fn owned_id(&self) -> String {
        self.id.to_string()
    }

    fn from_json(data: &str) -> Result<SimpleImageBuilder, Error> {
        let resource: SimpleImageBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<SimpleImageBuilder, Error> {
        let resource: Result<SimpleImageBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}

