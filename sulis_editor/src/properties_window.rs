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
use sulis_widgets::{Button, InputField, Label};

use AreaEditor;

pub const NAME: &str = "properties_window";

pub struct PropertiesWindow {
    area_editor: Rc<RefCell<AreaEditor>>,
}

impl PropertiesWindow {
    pub fn new(area_editor: Rc<RefCell<AreaEditor>>) -> Rc<RefCell<PropertiesWindow>> {
        Rc::new(RefCell::new(PropertiesWindow {
            area_editor
        }))
    }
}

impl WidgetKind for PropertiesWindow {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let id_label = Widget::with_theme(Label::empty(), "id_label");
        let id_field = Widget::with_theme(InputField::new(), "id_field");

        let name_label = Widget::with_theme(Label::empty(), "name_label");
        let name_field = Widget::with_theme(InputField::new(), "name_field");

        vec![title, close, id_label, id_field, name_label, name_field]
    }
}
