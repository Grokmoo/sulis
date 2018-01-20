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
//  You should have received a copy of the GNU General Public License//
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

pub mod simple_image;
pub use self::simple_image::SimpleImage;

pub mod composed_image;
pub use self::composed_image::ComposedImage;

pub mod animated_image;
pub use self::animated_image::AnimatedImage;

use std::fmt::Debug;

use io::{GraphicsRenderer, TextRenderer};
use ui::AnimationState;
use util::{Point, Size};

pub trait Image: Debug {
    fn draw_text_mode(&self, renderer: &mut TextRenderer, state: &AnimationState,
                      position: &Point);

    fn fill_text_mode(&self, renderer: &mut TextRenderer, state: &AnimationState,
                      position: &Point, size: &Size) {

        let mut rel_y = 0;
        while rel_y < size.height {
            let mut rel_x = 0;
            while rel_x < size.width {
                self.draw_text_mode(renderer, state, &position.add(rel_x, rel_y));
                rel_x += self.get_size().width;
            }
            rel_y += self.get_size().height;
        }
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, state: &AnimationState,
                          x: f32, y: f32, w: f32, h: f32);

    fn get_width_f32(&self) -> f32;
    fn get_height_f32(&self) -> f32;

    fn get_size(&self) -> &Size;
}
