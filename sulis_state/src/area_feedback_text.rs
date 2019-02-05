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
use std::time::Instant;

use sulis_core::config::Config;
use sulis_core::resource::{ResourceSet, Font};
use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::{Color, LineRenderer};
use sulis_core::util::{self, Point};
use sulis_module::DamageKind;

pub struct Params {
    pub font: Rc<Font>,
    pub scale: f32,
    pub info_color: Color,
    pub miss_color: Color,
    pub hit_color: Color,
    pub heal_color: Color,
    pub damage_colors: [Color; 8],
}

impl Default for Params {
    fn default() -> Params {
        use sulis_core::ui::color::*;
        Params {
            font: ResourceSet::default_font(),
            scale: 1.0,
            info_color: LIGHT_GRAY,
            miss_color: LIGHT_GRAY,
            hit_color: RED,
            heal_color: BLUE,
            damage_colors: [LIGHT_GRAY, LIGHT_GRAY, LIGHT_GRAY, GREEN,
                CYAN, BLUE, YELLOW, PURPLE],
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ColorKind {
    Info,
    Miss,
    Hit,
    Heal,
    Damage { kind: DamageKind },
}

pub struct AreaFeedbackText {
    pos_x: f32,
    pos_y: f32,
    text: String,
    text_width: f32,
    start_time: Instant,
    duration: u32,
    color_kind: ColorKind,
    move_rate: f32,

    hover_y: f32,
    alpha: f32,

    area_pos: Point,
}

impl AreaFeedbackText {
    pub fn new(area_pos: Point, text: String,
               pos_x: f32, pos_y: f32, color_kind: ColorKind, move_rate: f32) -> AreaFeedbackText {
        let duration = Config::animation_base_time_millis() * (50 + text.len() as u32 / 2);

        AreaFeedbackText {
            area_pos,
            text,
            text_width: 0.0,
            pos_x,
            pos_y,
            color_kind,
            move_rate,
            start_time: Instant::now(),
            duration,
            hover_y: 0.0,
            alpha: 1.0,
        }
    }

    pub fn area_pos(&self) -> Point { self.area_pos }

    pub fn cur_hover_y(&self) -> f32 { self.hover_y }

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

    // it is assumed that the params being passed in here do not change
    pub fn draw(&mut self, renderer: &mut GraphicsRenderer, params: &Params,
                offset_x: f32, offset_y: f32,
                scale_x: f32, scale_y: f32) {
        // creating the line renderer here is not ideal but is a low cost operation
        let font_renderer = LineRenderer::new(&params.font);
        if self.text_width == 0.0 {
            self.text_width = params.font.get_width(&self.text) as f32
                / params.font.line_height as f32;
        }

        let mut color = match self.color_kind {
            ColorKind::Info => params.info_color,
            ColorKind::Miss => params.miss_color,
            ColorKind::Hit => params.hit_color,
            ColorKind::Heal => params.heal_color,
            ColorKind::Damage { kind } => {
                let index = kind.index();
                params.damage_colors[index]
            }
        };
        color.a = self.alpha;

        let mut pos_x = offset_x + self.pos_x - params.scale * self.text_width / 2.0;
        if pos_x < 0.0 { pos_x = 0.0; }
        let pos_y = offset_y + self.pos_y - self.hover_y;

        let mut draw_list = font_renderer.get_draw_list(&self.text, pos_x, pos_y, params.scale);
        draw_list.set_scale(scale_x, scale_y);
        draw_list.set_color(color);
        renderer.draw(draw_list);
    }
}
