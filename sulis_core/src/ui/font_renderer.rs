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

use crate::io::{DrawList, GraphicsRenderer, Vertex};
use crate::resource::Font;
use crate::ui::WidgetState;
use std::rc::Rc;

pub trait FontRenderer {
    fn render(
        &self,
        renderer: &mut GraphicsRenderer,
        pos_x: f32,
        pos_y: f32,
        widget_state: &WidgetState,
    );

    fn get_font(&self) -> &Rc<Font>;
}

pub struct LineRenderer {
    font: Rc<Font>,
}

impl LineRenderer {
    pub fn new(font: &Rc<Font>) -> LineRenderer {
        LineRenderer {
            font: Rc::clone(font),
        }
    }

    pub fn get_draw_list(&self, text: &str, pos_x: f32, pos_y: f32, scale: f32) -> (DrawList, f32) {
        let mut quads: Vec<Vertex> = Vec::new();
        let mut x = pos_x;
        for c in text.chars() {
            x = self.font.get_quad(&mut quads, c, x, pos_y, scale);
        }
        (DrawList::from_font(&self.font.id, quads), x)
    }
}

impl FontRenderer for LineRenderer {
    fn render(
        &self,
        renderer: &mut GraphicsRenderer,
        pos_x: f32,
        pos_y: f32,
        widget_state: &WidgetState,
    ) {
        let text = &widget_state.text;
        let defaults = &widget_state.text_params;

        let mut quads: Vec<Vertex> = Vec::new();
        let mut x = pos_x;
        for c in text.chars() {
            x = self.font.get_quad(&mut quads, c, x, pos_y, defaults.scale);
        }

        let mut draw_list = DrawList::from_font(&self.font.id, quads);
        draw_list.set_color(defaults.color);
        renderer.draw(draw_list);
    }

    fn get_font(&self) -> &Rc<Font> {
        &self.font
    }
}
