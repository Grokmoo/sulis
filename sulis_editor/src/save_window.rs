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
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_widgets::{Button, InputField, Label};

use AreaEditor;

pub const NAME: &str = "save_window";

pub struct SaveWindow {
    area_editor: Rc<RefCell<AreaEditor>>,
}

impl SaveWindow {
    pub fn new(area_editor: Rc<RefCell<AreaEditor>>) -> Rc<RefCell<SaveWindow>> {
        Rc::new(RefCell::new(SaveWindow {
            area_editor
        }))
    }
}

impl WidgetKind for SaveWindow {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let id_label = Widget::with_theme(Label::empty(), "id_label");
        let id_field = Widget::with_theme(
            InputField::new(&self.area_editor.borrow().model.id()), "id_field");

        let area_editor_ref = Rc::clone(&self.area_editor);
        id_field.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_widget, kind| {
            let input_field = match kind.as_any_mut().downcast_mut::<InputField>() {
                Some(mut input_field) => input_field,
                None => panic!("Failed to downcast to InputField"),
            };
            area_editor_ref.borrow_mut().model.set_id(&input_field.text);
        })));

        let name_label = Widget::with_theme(Label::empty(), "name_label");
        let name_field = Widget::with_theme(
            InputField::new(&self.area_editor.borrow().model.name()), "name_field");

        let area_editor_ref = Rc::clone(&self.area_editor);
        name_field.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_widget, kind| {
            let input_field = match kind.as_any_mut().downcast_mut::<InputField>() {
                Some(mut input_field) => input_field,
                None => panic!("Failed to downcast to InputField"),
            };
            area_editor_ref.borrow_mut().model.set_name(&input_field.text);
        })));

        let filename_label = Widget::with_theme(Label::empty(), "filename_label");
        let filename_field = Widget::with_theme(
            InputField::new(&self.area_editor.borrow().model.filename()), "filename_field");

        let area_editor_ref = Rc::clone(&self.area_editor);
        filename_field.borrow_mut().state.add_callback(Callback::new(Rc::new(move |_widget, kind| {
            let input_field = match kind.as_any_mut().downcast_mut::<InputField>() {
                Some(mut input_field) => input_field,
                None => panic!("Failed to downcast to InputField"),
            };
            area_editor_ref.borrow_mut().model.set_filename(&input_field.text);
        })));

        let save_button = Widget::with_theme(Button::empty(), "save_button");

        let area_editor_kind_ref = Rc::clone(&self.area_editor);
        save_button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _kind| {
            let parent = Widget::get_parent(widget);
            let filename_prefix = format!("../{}/{}/areas/",
                                          Config::resources_config().campaigns_directory,
                                          Config::editor_config().module);
            area_editor_kind_ref.borrow().model.save(&filename_prefix);
            parent.borrow_mut().mark_for_removal();
        })));

        vec![title, close, id_label, id_field, name_label, name_field,
            filename_label, filename_field, save_button]
    }
}
