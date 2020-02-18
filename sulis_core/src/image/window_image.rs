//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use crate::config::Config;
use crate::image::Image;
use crate::io::{DrawList, GraphicsRenderer};
use crate::resource::{ResourceSet, Sprite};
use crate::ui::AnimationState;
use crate::util::{Rect, Size};

/// An animated image with a moving window that covers part of the
/// image and changes over time.

#[derive(Debug)]
pub struct WindowImage {
    id: String,
    image: Rc<Sprite>,
    size: Size,
    initial: Rect,
    delta: Rect,
}

impl WindowImage {
    pub fn generate(
        builder: WindowImageBuilder,
        resources: &ResourceSet,
    ) -> Result<Rc<dyn Image>, Error> {
        let sprite = resources.sprite_internal(&builder.image)?;

        Ok(Rc::new(WindowImage {
            id: builder.id,
            image: sprite,
            size: builder.size,
            initial: builder.initial,
            delta: builder.delta,
        }))
    }

    fn get_draw_list(&self, rect: Rect, millis: u32) -> DrawList {
        let time = millis as f32 / 1000.0;

        let rel_x = self.initial.x + self.delta.x * time;
        let rel_y = self.initial.y + self.delta.y * time;
        let rel_w = self.initial.w + self.delta.w * time;
        let rel_h = self.initial.h + self.delta.h * time;

        let abs_tc = &self.image.tex_coords;
        let abs_x = abs_tc[0];
        let abs_w = abs_tc[4] - abs_x;
        let abs_y = abs_tc[3];
        let abs_h = abs_tc[1] - abs_y;

        let x_min = abs_x + abs_w * rel_x;
        let x_max = abs_x + abs_w * rel_w;
        let y_min = abs_y + abs_h * rel_y;
        let y_max = abs_y + abs_h * rel_h;

        let tc = [x_min, y_max, x_min, y_min, x_max, y_max, x_max, y_min];
        let draw = Rect {
            x: rect.x,
            y: rect.y,
            w: rect.w * rel_w,
            h: rect.h * rel_h,
        };
        let mut list = DrawList::from_texture_id(&self.image.sheet_id, &tc, draw);

        let y_max = Config::ui_height() as f32 - rect.y;
        let y_min = y_max - rect.h;
        list.centroid = Some([rect.x + rect.w / 2.0, (y_max + y_min) / 2.0]);
        list
    }
}

impl Image for WindowImage {
    fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        _state: &AnimationState,
        rect: Rect,
        millis: u32,
    ) {
        renderer.draw(self.get_draw_list(rect, millis));
    }

    fn append_to_draw_list(
        &self,
        draw_list: &mut DrawList,
        _state: &AnimationState,
        rect: Rect,
        millis: u32,
    ) {
        draw_list.append(&mut self.get_draw_list(rect, millis));
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
pub struct WindowImageBuilder {
    id: String,
    image: String,
    size: Size,
    initial: Rect,
    delta: Rect,
}
