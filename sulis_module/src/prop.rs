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

use sulis_core::ui::{AnimationState};
use sulis_core::io::DrawList;
use sulis_core::image::Image;
use sulis_core::serde_json;
use sulis_core::serde_yaml;
use sulis_core::resource::{ResourceBuilder, ResourceSet};
use sulis_core::util::unable_to_create_error;

use Module;

#[derive(Debug)]
pub struct Prop {
    pub id: String,
    pub name: String,
    pub icon: Rc<Image>,
    pub image: Rc<Image>,
    pub width: u32,
    pub height: u32,
    pub passable: bool,
    pub visible: bool,
    pub interactive: bool,
}

impl Prop {
    pub fn new(builder: PropBuilder, _module: &Module) -> Result<Prop, Error> {
        let icon = match ResourceSet::get_image(&builder.icon) {
            None => {
                    warn!("No image found for icon '{}'", builder.icon);
                    return unable_to_create_error("prop", &builder.id);
            }, Some(icon) => icon,
        };

        let image = match ResourceSet::get_image(&builder.image) {
            None => {
                warn!("No image found for image '{}'", builder.image);
                return unable_to_create_error("prop", &builder.id);
            }, Some(image) => image,
        };

        // TODO props need per square passability

        Ok(Prop {
            id: builder.id,
            name: builder.name,
            icon,
            image,
            width: builder.width,
            height: builder.height,
            passable: builder.passable,
            visible: builder.visible,
            interactive: builder.interactive,
        })
    }

    pub fn append_to_draw_list(&self, draw_list: &mut DrawList, state: &AnimationState,
                               x: f32, y: f32, millis: u32) {
        let w = self.width as f32;
        let h = self.height as f32;

        self.image.append_to_draw_list(draw_list, state, x, y, w, h, millis);
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PropBuilder {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub image: String,
    pub width: u32,
    pub height: u32,
    pub passable: bool,
    pub visible: bool,
    pub interactive: bool,
}

impl ResourceBuilder for PropBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<PropBuilder, Error> {
        let resource: PropBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<PropBuilder, Error> {
        let resource: Result<PropBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
