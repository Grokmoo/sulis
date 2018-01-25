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

use sulis_core::ui::{Widget, WidgetKind};
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
        for tile in Module::all_tiles() {
            let button = Widget::with_theme(Button::with_text(&tile.name), "tile_button");
            widgets.push(button);
        }

        widgets
    }
}
