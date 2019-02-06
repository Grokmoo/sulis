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
use std::cell::RefCell;
use std::rc::Rc;

use crate::io::event::ClickKind;
use crate::io::GraphicsRenderer;
use crate::ui::theme::{self, HorizontalAlignment, VerticalAlignment};
use crate::ui::{LineRenderer, Widget, WidgetKind};
use crate::util::Point;
use crate::widget_kind;
use crate::widgets::TextArea;

pub struct Label {
    pub text: Option<String>,
    pub text_draw_end_x: f32,
    pub text_draw_end_y: f32,
    tooltip: String,
}

impl Label {
    pub fn empty() -> Rc<RefCell<Label>> {
        Rc::new(RefCell::new(Label {
            text: None,
            text_draw_end_x: 0.0,
            text_draw_end_y: 0.0,
            tooltip: String::new(),
        }))
    }

    pub fn new(text: &str) -> Rc<RefCell<Label>> {
        Rc::new(RefCell::new(Label {
            text: Some(text.to_string()),
            text_draw_end_x: 0.0,
            text_draw_end_y: 0.0,
            tooltip: String::new(),
        }))
    }

    fn get_draw_params(width: f32, height: f32, widget: &Widget) -> (f32, f32) {
        let x = widget.state.inner_left() as f32;
        let y = widget.state.inner_top() as f32;
        let w = widget.state.inner_width() as f32;
        let h = widget.state.inner_height() as f32;

        let len = if width > w { w } else { width };

        let x = match widget.state.text_params.horizontal_alignment {
            HorizontalAlignment::Left => x,
            HorizontalAlignment::Center => (x + (w - len) / 2.0),
            HorizontalAlignment::Right => x + w - len,
        };

        let y = match widget.state.text_params.vertical_alignment {
            VerticalAlignment::Top => y,
            VerticalAlignment::Center => y + (h - height) / 2.0,
            VerticalAlignment::Bottom => y + h - height,
        };

        (x, y)
    }
}

impl WidgetKind for Label {
    widget_kind!["label"];

    fn on_mouse_press(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);
        false
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        false
    }

    fn on_mouse_drag(
        &mut self,
        _widget: &Rc<RefCell<Widget>>,
        _kind: ClickKind,
        _delta_x: f32,
        _delta_y: f32,
    ) -> bool {
        false
    }

    fn on_mouse_enter(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        false
    }

    fn on_mouse_exit(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        self.super_on_mouse_exit(widget);
        false
    }

    // TODO refactor tooltip code into a common case somewhere
    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>, _dx: f32, _dy: f32) -> bool {
        if self.tooltip.is_empty() {
            return false;
        }

        let tooltip = Widget::with_theme(TextArea::empty(), "tooltip");
        tooltip.borrow_mut().state.add_text_arg("0", &self.tooltip);

        let (x, y) = {
            let state = &widget.borrow().state;
            (state.position().x + state.size().width, state.position().y)
        };

        Widget::set_mouse_over_widget(widget, tooltip, x, y);

        true
    }

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref text) = self.text {
            widget.state.add_text_arg("0", text);
        }

        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }

        if let Some(tooltip) = widget.theme.custom.get("tooltip") {
            self.tooltip = theme::expand_text_args(tooltip, &widget.state);
        }
    }

    fn draw(
        &mut self,
        renderer: &mut GraphicsRenderer,
        _pixel_size: Point,
        widget: &Widget,
        _millis: u32,
    ) {
        let font_rend = match &widget.state.text_renderer {
            &None => return,
            &Some(ref renderer) => renderer,
        };

        let font = font_rend.get_font();
        let scale = widget.state.text_params.scale;
        let width = font.get_width(&widget.state.text) as f32 * scale / font.line_height as f32;
        let (x, y) = Label::get_draw_params(width, scale, widget);
        font_rend.render(renderer, x, y, &widget.state);

        self.text_draw_end_x = x + width;
        self.text_draw_end_y = y;
    }
}
