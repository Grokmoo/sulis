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
use std::cell::RefCell;
use std::fmt::Display;
use std::slice::Iter;

use list_box::Entry;
use sulis_core::io::{event, GraphicsRenderer};
use sulis_core::ui::{LineRenderer, Widget, WidgetKind};
use sulis_core::util::Point;
use {Label, ListBox};

const NAME: &str = "drop_down";

pub struct DropDown<T: Display + Clone + 'static> {
    entries: Vec<Entry<T>>,
    label: Rc<RefCell<Label>>,
}

impl<T: Display + Clone + 'static> DropDown<T> {
    pub fn new(entries: Vec<Entry<T>>) -> Rc<RefCell<DropDown<T>>> {
        Rc::new(RefCell::new(DropDown {
            entries,
            label: Label::empty(),
        }))
    }

    pub fn iter(&self) -> Iter<Entry<T>> {
        self.entries.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Entry<T>> {
        self.entries.get(index)
    }
}

impl<T: Display + Clone + 'static> WidgetKind for DropDown<T> {
    fn get_name(&self) -> &str {
        NAME
    }

    fn layout(&self, widget: &mut Widget) {
        widget.do_base_layout();

        if let Some(ref font) = widget.state.font {
            widget.state.text_renderer = Some(Box::new(LineRenderer::new(font)));
        }
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: event::ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        let list_box = Widget::with_theme(ListBox::new(self.entries.clone()), "list");

        Widget::add_child_to(widget, list_box);
        true
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, pixel_size: Point,
                          widget: &Widget, millis: u32) {
        self.label.borrow_mut().draw_graphics_mode(renderer, pixel_size, widget, millis);
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        Vec::new()
    }
}
