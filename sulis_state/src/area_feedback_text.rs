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

use std::time::Instant;

use sulis_core::config::CONFIG;
use sulis_core::resource::ResourceSet;
use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::{Color, LineRenderer};
use sulis_core::util;

pub struct AreaFeedbackText {
    pos_x: f32,
    pos_y: f32,
    text: String,
    text_width: f32,
    start_time: Instant,
    duration: u32,
    font_renderer: LineRenderer,
    color: Color,
    move_rate: f32,

    hover_y: f32,
    alpha: f32,
}

impl AreaFeedbackText {
    pub fn new(text: String, pos_x: f32, pos_y: f32, color: Color, move_rate: f32) -> AreaFeedbackText {
        let font = ResourceSet::get_default_font();
        let text_width = font.get_width(&text) as f32 / font.line_height as f32;

        AreaFeedbackText {
            text,
            text_width,
            pos_x,
            pos_y,
            color,
            move_rate,
            start_time: Instant::now(),
            duration: CONFIG.display.animation_base_time_millis * 40,
            font_renderer: LineRenderer::new(&font),

            hover_y: 0.0,
            alpha: 1.0,
        }
    }

    pub fn update(&mut self) {
        let frac = util::get_elapsed_millis(self.start_time.elapsed()) as f32 / self.duration as f32;

        self.hover_y = frac * self.move_rate;

        if frac < 0.5 {
            self.alpha = 1.0;
        } else {
            self.alpha = (1.0 - frac) * 2.0;
        }
    }

    pub fn retain(&self) -> bool {
        self.alpha > 0.0
    }

    pub fn draw(&self, renderer: &mut GraphicsRenderer, text_scale: f32, offset_x: f32, offset_y: f32,
                scale_x: f32, scale_y: f32) {
        // TODO font, color at a minimum should be configurable via text configuration
        let pos_x = offset_x + self.pos_x - text_scale * self.text_width / 2.0;
        let pos_y = offset_y + self.pos_y - self.hover_y;

        let mut draw_list = self.font_renderer.get_draw_list(&self.text, pos_x, pos_y, text_scale);
        draw_list.set_scale(scale_x, scale_y);
        draw_list.set_color(Color::new(self.color.r, self.color.g, self.color.b, self.alpha));
        renderer.draw(draw_list);
    }
}
