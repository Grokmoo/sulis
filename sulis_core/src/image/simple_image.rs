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

use std::io::Error;
use std::rc::Rc;

use crate::image::Image;
use crate::io::{DrawList, GraphicsRenderer};
use crate::resource::{ResourceSet, Sprite};
use crate::ui::AnimationState;
use crate::util::Size;

#[derive(Debug)]
pub struct SimpleImage {
    pub id: String,
    pub image_display: Rc<Sprite>,
    pub size: Size,
}

impl SimpleImage {
    pub fn new(
        builder: SimpleImageBuilder,
        resources: &ResourceSet,
    ) -> Result<Rc<dyn Image>, Error> {
        let sprite = resources.sprite_internal(&builder.image_display)?;

        Ok(Rc::new(SimpleImage {
            id: builder.id,
            size: builder.size,
            image_display: sprite,
        }))
    }
}

impl Image for SimpleImage {
    fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        _state: &AnimationState,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        _millis: u32,
    ) {
        renderer.draw(DrawList::from_sprite_f32(&self.image_display, x, y, w, h));
    }

    fn append_to_draw_list(
        &self,
        draw_list: &mut DrawList,
        _state: &AnimationState,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        _millis: u32,
    ) {
        draw_list.append(&mut DrawList::from_sprite_f32(
            &self.image_display,
            x,
            y,
            w,
            h,
        ));
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

    fn id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SimpleImageBuilder {
    pub(crate) id: String,
    pub(crate) image_display: String,
    pub(crate) size: Size,
}
