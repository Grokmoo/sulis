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
use std::time::Instant;

use crate::io::{event, GraphicsRenderer};
use crate::ui::{theme, LineRenderer, Widget, WidgetKind, RcRfc};
use crate::util::{self, Point};
use crate::widget_kind;
use crate::widgets::{Label, TextArea};

pub struct Button {
    label: RcRfc<Label>,
    mouse_pressed_time: Option<Instant>,
    last_repeat_time: Option<Instant>,
    repeat_init_time: u32,
    repeat_time: u32,
    tooltip: String,
}

impl Button {
    pub fn empty() -> RcRfc<Button> {
        Rc::new(RefCell::new(Button {
            label: Label::empty(),
            mouse_pressed_time: None,
            last_repeat_time: None,
            repeat_init_time: 0,
            repeat_time: 0,
            tooltip: String::new(),
        }))
    }

    pub fn with_text(text: &str) -> RcRfc<Button> {
        Rc::new(RefCell::new(Button {
            label: Label::new(text),
            mouse_pressed_time: None,
            last_repeat_time: None,
            repeat_init_time: 0,
            repeat_time: 0,
            tooltip: String::new(),
        }))
    }
}

impl WidgetKind for Button {
    widget_kind!["button"];

    fn update(&mut self, widget: &RcRfc<Widget>, _millis: u32) {
        if self.repeat_time == 0 {
            return;
        }

        let start_time = match self.mouse_pressed_time {
            None => return,
            Some(time) => time,
        };

        if util::get_elapsed_millis(start_time.elapsed()) < self.repeat_init_time {
            return;
        }

        let millis = match self.last_repeat_time {
            None => self.repeat_time,
            Some(time) => util::get_elapsed_millis(time.elapsed()),
        };

        // this is going to be very frame rate dependant for small values of
        // repeat_time, but it is fine for our purposes
        if millis >= self.repeat_time {
            self.last_repeat_time = Some(Instant::now());
            Widget::fire_callback(widget, self);
        }
    }

    fn layout(&mut self, widget: &mut Widget) {
        if let Some(ref text) = self.label.borrow().text {
            widget.state.add_text_arg("0", text);
        }
        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }

        let theme = &widget.theme;
        self.repeat_time = theme.get_custom_or_default("repeat_time", 0);
        self.repeat_init_time = theme.get_custom_or_default("repeat_init_time", 0);
        if let Some(tooltip) = theme.custom.get("tooltip") {
            self.tooltip = theme::expand_text_args(tooltip, &widget.state);
        }
    }

    fn draw(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        pixel_size: Point,
        widget: &Widget,
        millis: u32,
    ) {
        self.label
            .borrow_mut()
            .draw(renderer, pixel_size, widget, millis);
    }

    // TODO refactor tooltip code into a common case somewhere - can probably
    // also include item button, ability button
    fn on_mouse_move(&mut self, widget: &RcRfc<Widget>, _dx: f32, _dy: f32) -> bool {
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

    fn on_mouse_press(&mut self, widget: &RcRfc<Widget>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_press(widget, kind);

        self.mouse_pressed_time = Some(Instant::now());
        true
    }

    fn on_mouse_release(&mut self, widget: &RcRfc<Widget>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        Widget::fire_callback(widget, self);
        self.mouse_pressed_time = None;
        self.last_repeat_time = None;
        true
    }
}
