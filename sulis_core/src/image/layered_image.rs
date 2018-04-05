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
use ui::{animation_state, AnimationState, Color};
use util::Size;

#[derive(Debug)]
pub struct LayeredImage {
    layers: Vec<(f32, f32, Option<Color>, Rc<Image>)>,
    hue: Option<f32>,
    size: Size,
}

impl LayeredImage {
    pub fn new(images: Vec<(f32, f32, Option<Color>, Rc<Image>)>, swap_hue: Option<f32>) -> LayeredImage {
        let mut max_x = 0.0;
        let mut max_y = 0.0;

        for &(_x, _y, _color, ref image) in images.iter() {
            if image.get_width_f32() > max_x {
                max_x = image.get_width_f32();
            }

            if image.get_height_f32() > max_y {
                max_y = image.get_height_f32();
            }
        }

        LayeredImage {
            layers: images,
            hue: swap_hue,
            size: Size::new(max_x as i32, max_y as i32),
        }
    }

    pub fn draw_to_texture(&self, renderer: &mut GraphicsRenderer, texture_id: &str, scale_x: f32,
                           scale_y: f32, x: f32, y: f32) {
        let state = &animation_state::NORMAL;
        for &(offset_x, offset_y, color, ref image) in self.layers.iter() {
            let mut draw_list = DrawList::empty_sprite();
            let w = image.get_width_f32();
            let h = image.get_height_f32();
            image.append_to_draw_list(&mut draw_list, state, x + offset_x, y + offset_y, w, h, 0);
            if let Some(color) = color {
                draw_list.set_color(color);
            }
            if let Some(hue) = self.hue {
                draw_list.set_swap_hue(hue);
            }
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw_to_texture(texture_id, draw_list);
        }
    }

    // TODO refactor these two almost identical functions
    pub fn draw(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32,
            x: f32, y: f32, millis: u32) {
        let state = &animation_state::NORMAL;
        for &(offset_x, offset_y, color, ref image) in self.layers.iter() {
            let mut draw_list = DrawList::empty_sprite();
            let w = image.get_width_f32();
            let h = image.get_height_f32();
            image.append_to_draw_list(&mut draw_list, state, x + offset_x, y + offset_y, w, h, millis);
            if let Some(color) = color {
                draw_list.set_color(color);
            }
            if let Some(hue) = self.hue {
                draw_list.set_swap_hue(hue);
            }
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }
    }
}

impl Image for LayeredImage {
    fn append_to_draw_list(&self, _draw_list: &mut DrawList, _state: &AnimationState,
                           _x: f32, _y: f32, _w: f32, _h: f32, _millis: u32) {
        panic!("LayeredImage must be drawn directly");
    }

    fn draw_graphics_mode(&self, _renderer: &mut GraphicsRenderer, _state: &AnimationState,
                          _x: f32, _y: f32, _w: f32, _h: f32, _millis: u32) {
        panic!("LayeredImage must be drawn directly");
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

    fn id(&self) -> String {
        String::new()
    }
}
