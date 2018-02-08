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

use image::Image;
use io::{DrawList, GraphicsRenderer};
use ui::{AnimationState, animation_state};
use util::Size;

#[derive(Debug)]
pub struct LayeredImage {
    layers: Vec<(f32, f32, Rc<Image>)>,
    size: Size,
}

impl LayeredImage {
    pub fn new(images: Vec<(f32, f32, Rc<Image>)>) -> LayeredImage {
        let mut max_x = 0.0;
        let mut max_y = 0.0;

        for &(_x, _y, ref image) in images.iter() {
            if image.get_width_f32() > max_x {
                max_x = image.get_width_f32();
            }

            if image.get_height_f32() > max_y {
                max_y = image.get_height_f32();
            }
        }

        LayeredImage {
            layers: images,
            size: Size::new(max_x as i32, max_y as i32),
        }
    }

    pub fn append_to(&self, draw_list: &mut DrawList, x: f32, y: f32, millis: u32) {
        for &(offset_x, offset_y, ref image) in self.layers.iter() {
            image.append_to_draw_list(draw_list, &animation_state::NORMAL, x + offset_x, y + offset_y,
                                      image.get_width_f32(), image.get_height_f32(), millis);
        }
    }
}

impl Image for LayeredImage {
    fn append_to_draw_list(&self, draw_list: &mut DrawList, state: &AnimationState,
                           x: f32, y: f32, w: f32, h: f32, millis: u32) {
        for &(offset_x, offset_y, ref image) in self.layers.iter() {
            image.append_to_draw_list(draw_list, state, x + offset_x, y + offset_y, w, h, millis);
        }
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, state: &AnimationState,
                          x: f32, y: f32, w: f32, h: f32, millis: u32) {
        let mut draw_list = DrawList::empty_sprite();
        self.append_to_draw_list(&mut draw_list, state, x, y, w, h, millis);
        renderer.draw(draw_list);
    }

    fn get_size(&self) -> &Size {
        &self.size
    }

    fn get_width_f32(&self) -> f32 {
        self.size.width as f32
    }

    fn get_height_f32(&self) -> f32 {
        self.size.height as f32
    }
}
