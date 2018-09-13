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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Label, Button, TextArea};
use sulis_module::Module;

pub const NAME: &str = "world_map_window";

pub struct Entry {
    child: Rc<RefCell<Widget>>,
    position: (f32, f32),
}

pub struct WorldMapWindow {
    entries: Vec<Entry>,
    size: (f32, f32),
    offset: (f32, f32),
    content: Rc<RefCell<Widget>>,
}

impl WorldMapWindow {
    pub fn new() -> Rc<RefCell<WorldMapWindow>> {
        Rc::new(RefCell::new(WorldMapWindow {
            entries: Vec::new(),
            size: (0.0, 0.0),
            offset: (0.0, 0.0),
            content: Widget::empty("content"),
        }))
    }
}

impl WidgetKind for WorldMapWindow {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        widget.do_self_layout();
        widget.do_children_layout();

        {
            let state= &self.content.borrow().state;
            let start_x = state.inner_position.x;
            let start_y = state.inner_position.y;

            let w = state.inner_size.width;
            let h = state.inner_size.height;

            let grid_w = w as f32 / self.size.0 as f32;
            let grid_h = h as f32 / self.size.1 as f32;

            let offset_x = self.offset.0 * grid_w;
            let offset_y = self.offset.1 * grid_h;

            for entry in self.entries.iter() {
                let x = start_x + (grid_w * entry.position.0 + offset_x) as i32;
                let y = start_y + (grid_h * entry.position.1 + offset_y) as i32;
                entry.child.borrow_mut().state.set_position(x, y);
            }
        }

        widget.do_children_layout();
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let label = Widget::with_theme(Label::empty(), "title");
        let bg= Widget::empty("bg");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let labels = Widget::with_theme(TextArea::empty(), "labels");

        let campaign = Module::campaign();
        let map = &campaign.world_map;

        self.content = Widget::empty("content");
        self.entries.clear();
        self.size = map.size;
        self.offset = map.offset;

        for location in map.locations.iter() {
            let button = Widget::with_theme(Button::empty(), "location");

            {
                let mut state = &mut button.borrow_mut().state;
                state.add_text_arg("name", &location.name);
                state.add_text_arg("icon", &location.icon.id());
            }

            button.borrow_mut().state.set_enabled(location.initially_enabled);

            let entry = Entry {
                child: Rc::clone(&button),
                position: location.position,
            };

            self.entries.push(entry);
            Widget::add_child_to(&self.content, button);
        }

        vec![label, bg, close, labels, Rc::clone(&self.content)]
    }
}
