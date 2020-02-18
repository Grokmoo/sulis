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

use crate::image::Image;
use crate::io::{DrawList, GraphicsRenderer};
use crate::ui::{animation_state, AnimationState, Color};
use crate::util::{Offset, Rect, Scale, Size};

#[derive(Debug)]
pub struct Layer {
    x: f32,
    y: f32,
    color: Option<Color>,
    image: Rc<dyn Image>,
}

impl Layer {
    pub fn new(x: f32, y: f32, color: Option<Color>, image: Rc<dyn Image>) -> Layer {
        Layer { x, y, color, image }
    }
}

#[derive(Debug)]
pub struct LayeredImage {
    layers: Vec<Layer>,
    hue: Option<f32>,
    size: Size,
}

impl LayeredImage {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(images: Vec<Layer>, swap_hue: Option<f32>) -> LayeredImage {
        let mut max_x = 0.0;
        let mut max_y = 0.0;

        for layer in images.iter() {
            if layer.image.get_width_f32() > max_x {
                max_x = layer.image.get_width_f32();
            }

            if layer.image.get_height_f32() > max_y {
                max_y = layer.image.get_height_f32();
            }
        }

        LayeredImage {
            layers: images,
            hue: swap_hue,
            size: Size::new(max_x as i32, max_y as i32),
        }
    }

    #[inline]
    fn draw_layer(&self, layer: &Layer, offset: Offset, scale: Scale, millis: u32) -> DrawList {
        let mut draw_list = DrawList::empty_sprite();

        let draw = Rect {
            x: layer.x + offset.x,
            y: layer.y + offset.y,
            w: layer.image.get_width_f32(),
            h: layer.image.get_height_f32(),
        };

        layer
            .image
            .append_to_draw_list(&mut draw_list, &animation_state::NORMAL, draw, millis);
        if let Some(color) = &layer.color {
            draw_list.set_color(*color);
        }
        if let Some(hue) = self.hue {
            draw_list.set_swap_hue(hue);
        }
        draw_list.set_scale(scale);

        draw_list
    }

    pub fn draw_to_texture(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        texture_id: &str,
        offset: Offset,
        scale: Scale,
    ) {
        for layer in self.layers.iter() {
            let draw_list = self.draw_layer(layer, offset, scale, 0);
            renderer.draw_to_texture(texture_id, draw_list);
        }
    }

    pub fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        offset: Offset,
        scale: Scale,
        millis: u32,
    ) {
        for layer in self.layers.iter() {
            let draw_list = self.draw_layer(layer, offset, scale, millis);
            renderer.draw(draw_list);
        }
    }
}

impl Image for LayeredImage {
    fn append_to_draw_list(
        &self,
        _draw_list: &mut DrawList,
        _state: &AnimationState,
        _rect: Rect,
        _millis: u32,
    ) {
        panic!("LayeredImage must be drawn directly");
    }

    fn draw(
        &self,
        _renderer: &mut dyn GraphicsRenderer,
        _state: &AnimationState,
        _rect: Rect,
        _millis: u32,
    ) {
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
