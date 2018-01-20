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

use std::rc::Rc;

use ui::theme::{HorizontalTextAlignment, VerticalTextAlignment};
use ui::{LineRenderer, Widget, WidgetKind};
use io::GraphicsRenderer;
use util::Point;

pub struct Label {
    pub text: Option<String>,
}

impl Label {
    pub fn empty() -> Rc<Label> {
        Rc::new(Label {
            text: None,
        })
    }

    pub fn new(text: &str) -> Rc<Label> {
        Rc::new(Label {
            text: Some(text.to_string()),
        })
    }

    fn get_draw_params(width: f32, height: f32, widget: &Widget) -> (f32, f32, &str) {
        let text = &widget.state.text;
        let x = widget.state.inner_left() as f32;
        let y = widget.state.inner_top() as f32;
        let w = widget.state.inner_size.width as f32;
        let h = widget.state.inner_size.height as f32;

        let len = if width > w {
            w
        } else {
            width
        };

        let x = match widget.state.text_params.horizontal_alignment {
            HorizontalTextAlignment::Left => x,
            HorizontalTextAlignment::Center => (x + (w - len) / 2.0),
            HorizontalTextAlignment::Right => x +w - len,
        };

        let y = match widget.state.text_params.vertical_alignment {
            VerticalTextAlignment::Top => y,
            VerticalTextAlignment::Center => y + (h - height) / 2.0,
            VerticalTextAlignment::Bottom => y + h - height,
        };

        (x, y, &text)
    }
}

impl WidgetKind for Label {
    fn get_name(&self) -> &str {
        "label"
    }

    fn layout(&self, widget: &mut Widget) {
        if let Some(ref text) = self.text {
            widget.state.add_text_arg("0", text);
        }

        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }
    }

    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
                          widget: &Widget, _millis: u32) {
        let font_rend = match &widget.state.text_renderer {
            &None => return,
            &Some(ref renderer) => renderer,
        };

        let font = font_rend.get_font();
        let scale = widget.state.text_params.scale;
        let width = font.get_width(&widget.state.text) as f32 * scale / font.line_height as f32;
        let (x, y, text) = Label::get_draw_params(width, scale, widget);
        font_rend.render(renderer, text, x, y, &widget.state.text_params);
    }
}
