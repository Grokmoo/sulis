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

use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::config::Config;
use sulis_core::ui::theme::SizeRelative;
use sulis_core::ui::{Widget, WidgetKind};
use sulis_core::io::{GraphicsRenderer, event::ClickKind};
use sulis_core::util::{Point, Size};
use crate::MarkupRenderer;

pub struct TextArea {
    pub text: Option<String>,
    pub(crate) limit_to_screen_edge: bool,
}

impl TextArea {
    pub fn empty() -> Rc<RefCell<TextArea>> {
        Rc::new(RefCell::new(TextArea {
            text: None,
            limit_to_screen_edge: true,
        }))
    }

    pub fn new(text: &str) -> Rc<RefCell<TextArea>> {
        Rc::new(RefCell::new(TextArea {
            text: Some(text.to_string()),
            limit_to_screen_edge: true,
        }))
    }

    fn render_to_cache(&self, widget: &mut Widget) {
        if let Some(ref font) = widget.state.font {
            let mut renderer = MarkupRenderer::new(font, widget.state.inner_size.width);
            renderer.render_to_cache(&widget.state);
            widget.state.text_renderer = Some(Box::new(renderer));
        }
    }
}

impl WidgetKind for TextArea {
    widget_kind!["text_area"];

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        false
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        false
    }

    fn on_mouse_drag(&mut self, _widget: &Rc<RefCell<Widget>>, _kind: ClickKind,
                     _delta_x: f32, _delta_y: f32) -> bool {
        false
    }

    fn on_mouse_move(&mut self, _widget: &Rc<RefCell<Widget>>,
                     _delta_x: f32, _delta_y: f32) -> bool {
        true
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        false
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        false
    }

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref text) = self.text {
            widget.state.add_text_arg("0", text);
        }

        widget.do_base_layout();

        let mut right = widget.state.inner_right();
        let mut bottom = widget.state.inner_top();
        if let Some(ref font) = widget.state.font {
            let mut renderer = MarkupRenderer::new(font, widget.state.inner_size.width);
            renderer.render_to_cache(&widget.state);
            bottom = renderer.text_bottom();
            right = renderer.text_right();
            widget.state.text_renderer = Some(Box::new(renderer));
        }

        if let Some(ref theme) = widget.theme {
            let mut height = widget.state.size.height;
            let mut width = widget.state.size.width;

            if theme.height_relative == SizeRelative::Custom {
                height = bottom - widget.state.position.y + widget.state.border.bottom;
            }

            if theme.width_relative == SizeRelative::Custom {
                width = right - widget.state.position.x + widget.state.border.right;
            }

            widget.state.set_size(Size::new(width, height));
        }

        let (ui_x, ui_y) = Config::ui_size();

        // limit position of text area to inside the screen
        // we need to double render in this case - first render finds
        // the text area size, second render to actually display the text
        if !self.limit_to_screen_edge { return; }

        if widget.state.position.y + widget.state.size.height > ui_y {
            let x = widget.state.position.x;
            let y = ui_y - widget.state.size.height;
            widget.state.set_position(x, y);
            self.render_to_cache(widget);
        }

        if widget.state.position.x + widget.state.size.width > ui_x {
            let x = ui_x - widget.state.size.width;
            let y = widget.state.position.y;
            widget.state.set_position(x, y);
            self.render_to_cache(widget);
        }
    }

    fn draw(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
            widget: &Widget, _millis: u32) {
        let font_rend = match &widget.state.text_renderer {
            &None => return,
            &Some(ref renderer) => renderer,
        };

        // let start_time = time::Instant::now();
        let x = widget.state.inner_left() as f32;
        let y = widget.state.inner_top() as f32;
        font_rend.render(renderer, x, y, &widget.state);
        // info!("Text Area render time: {}", util::format_elapsed_secs(start_time.elapsed()));
    }
}
