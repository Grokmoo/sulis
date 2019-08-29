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

pub mod animated_image;
pub use self::animated_image::AnimatedImage;

pub mod composed_image;
pub use self::composed_image::ComposedImage;

pub mod layered_image;
pub use self::layered_image::LayeredImage;

pub mod simple_image;
pub use self::simple_image::SimpleImage;

pub mod timer_image;
pub use self::timer_image::TimerImage;

use std::fmt::Debug;

use crate::io::{DrawList, GraphicsRenderer};
use crate::ui::AnimationState;
use crate::util::{size, Size};

pub trait Image: Debug {
    fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        state: &AnimationState,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        millis: u32,
    );

    fn append_to_draw_list(
        &self,
        draw_list: &mut DrawList,
        state: &AnimationState,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        millis: u32,
    );

    fn get_width_f32(&self) -> f32;

    fn get_height_f32(&self) -> f32;

    fn get_size(&self) -> &Size;

    fn id(&self) -> String;
}

#[derive(Debug)]
pub struct EmptyImage {}

impl Image for EmptyImage {
    fn draw(
        &self,
        _renderer: &mut dyn GraphicsRenderer,
        _state: &AnimationState,
        _x: f32,
        _y: f32,
        _w: f32,
        _h: f32,
        _millis: u32,
    ) {
    }

    fn append_to_draw_list(
        &self,
        _draw_list: &mut DrawList,
        _state: &AnimationState,
        _x: f32,
        _y: f32,
        _w: f32,
        _h: f32,
        _millis: u32,
    ) {
    }

    fn get_width_f32(&self) -> f32 {
        0.0
    }
    fn get_height_f32(&self) -> f32 {
        0.0
    }
    fn get_size(&self) -> &Size {
        &size::ZERO_SIZE
    }
    fn id(&self) -> String {
        "empty".to_string()
    }
}
