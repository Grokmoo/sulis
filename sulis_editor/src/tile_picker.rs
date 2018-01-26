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

use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_module::Module;
use sulis_widgets::Button;

const NAME: &str = "tile_picker";

pub struct TilePicker { }

impl TilePicker {
    pub fn new() -> Rc<TilePicker> {
        Rc::new(TilePicker {

        })
    }
}

impl WidgetKind for TilePicker {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_add(&self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut widgets: Vec<Rc<RefCell<Widget>>> = Vec::new();

        let cb: Callback = Callback::new(Rc::new(move |widget| {
            let parent = Widget::get_parent(widget);
            let cur_state = widget.borrow_mut().state.is_active();
            if !cur_state {
                trace!("Set active: {}", widget.borrow().state.text);
                for child in parent.borrow_mut().children.iter() {
                    child.borrow_mut().state.set_active(false);
                }
                widget.borrow_mut().state.set_active(true);
            } else {
                widget.borrow_mut().state.set_active(false);
            }
        }));

        for tile in Module::all_tiles() {
            let button = Widget::with_theme(Button::with_text(&tile.name), "tile_button");
            button.borrow_mut().state.add_callback(cb.clone());
            widgets.push(button);
        }

        widgets
    }
}
