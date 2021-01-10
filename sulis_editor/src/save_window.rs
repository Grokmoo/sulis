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

use sulis_core::config::Config;
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::widgets::{Button, InputField, Label, Spinner};
use sulis_module::area::{LocationKind, OnRest};

use crate::AreaEditor;

pub const NAME: &str = "save_window";

pub struct SaveWindow {
    area_editor: Rc<RefCell<AreaEditor>>,
}

impl SaveWindow {
    pub fn new(area_editor: Rc<RefCell<AreaEditor>>) -> Rc<RefCell<SaveWindow>> {
        Rc::new(RefCell::new(SaveWindow { area_editor }))
    }
}

impl WidgetKind for SaveWindow {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let close = Widget::with_theme(Button::empty(), "close");
        close
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, _| {
                let (parent, _) = Widget::parent::<SaveWindow>(widget);
                parent.borrow_mut().mark_for_removal();
            })));

        let save = Widget::with_theme(Button::empty(), "save_button");
        let area_editor_kind_ref = Rc::clone(&self.area_editor);
        save.borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(move |widget, _kind| {
                let (parent, _) = Widget::parent::<SaveWindow>(widget);
                let filename_prefix = format!(
                    "../{}/{}/areas/",
                    Config::resources_config().campaigns_directory,
                    Config::editor_config().module
                );
                area_editor_kind_ref.borrow().model.save(&filename_prefix);
                parent.borrow_mut().mark_for_removal();
            })));

        let content = Widget::empty("content");

        let id_box = Widget::empty("id");
        {
            Widget::add_child_to(&id_box, Widget::with_defaults(Label::empty()));
            let field =
                Widget::with_defaults(InputField::new(&self.area_editor.borrow().model.id()));

            let area_editor_ref = Rc::clone(&self.area_editor);
            field
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |_, kind| {
                    let input_field = match kind.as_any_mut().downcast_mut::<InputField>() {
                        Some(input_field) => input_field,
                        None => panic!("Failed to downcast to InputField"),
                    };
                    area_editor_ref.borrow_mut().model.set_id(&input_field.text);
                })));

            Widget::add_child_to(&id_box, field);
        }
        Widget::add_child_to(&content, id_box);

        let name_box = Widget::empty("name");
        {
            Widget::add_child_to(&name_box, Widget::with_defaults(Label::empty()));
            let field =
                Widget::with_defaults(InputField::new(&self.area_editor.borrow().model.name()));

            let area_editor_ref = Rc::clone(&self.area_editor);
            field
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |_widget, kind| {
                    let input_field = match kind.as_any_mut().downcast_mut::<InputField>() {
                        Some(input_field) => input_field,
                        None => panic!("Failed to downcast to InputField"),
                    };
                    area_editor_ref
                        .borrow_mut()
                        .model
                        .set_name(&input_field.text);
                })));
            Widget::add_child_to(&name_box, field);
        }
        Widget::add_child_to(&content, name_box);

        let filename_box = Widget::empty("filename");
        {
            Widget::add_child_to(&filename_box, Widget::with_defaults(Label::empty()));
            let field =
                Widget::with_defaults(InputField::new(&self.area_editor.borrow().model.filename()));

            let area_editor_ref = Rc::clone(&self.area_editor);
            field
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |_widget, kind| {
                    let input_field = match kind.as_any_mut().downcast_mut::<InputField>() {
                        Some(input_field) => input_field,
                        None => panic!("Failed to downcast to InputField"),
                    };
                    area_editor_ref
                        .borrow_mut()
                        .model
                        .set_filename(&input_field.text);
                })));
            Widget::add_child_to(&filename_box, field);
        }
        Widget::add_child_to(&content, filename_box);

        let loc_box = Widget::empty("location_kind");
        {
            for kind in LocationKind::iter() {
                let button = Widget::with_defaults(Button::empty());
                button
                    .borrow_mut()
                    .state
                    .add_text_arg("name", &format!("{:?}", kind));

                if *kind == self.area_editor.borrow().model.location_kind() {
                    button.borrow_mut().state.set_active(true);
                }

                let area_editor_ref = Rc::clone(&self.area_editor);
                button
                    .borrow_mut()
                    .state
                    .add_callback(Callback::new(Rc::new(move |widget, _| {
                        let parent = Widget::direct_parent(widget);
                        for child in parent.borrow().children.iter() {
                            child.borrow_mut().state.set_active(false);
                        }
                        widget.borrow_mut().state.set_active(true);
                        area_editor_ref.borrow_mut().model.set_location_kind(*kind);
                    })));
                Widget::add_child_to(&loc_box, button);
            }
        }
        Widget::add_child_to(&content, loc_box);

        let on_rest_box = Widget::empty("on_rest");
        {
            let disabled = Widget::with_theme(Button::empty(), "disabled");
            let fire_script = Widget::with_theme(Button::empty(), "fire_script");

            match self.area_editor.borrow().model.on_rest {
                OnRest::Disabled { .. } => disabled.borrow_mut().state.set_active(true),
                OnRest::FireScript { .. } => fire_script.borrow_mut().state.set_active(true),
            }

            let area_editor_ref = Rc::clone(&self.area_editor);
            let fire_script_ref = Rc::clone(&fire_script);
            disabled
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    area_editor_ref.borrow_mut().model.on_rest = OnRest::Disabled {
                        message: "<<PLACEHOLDER>>".to_string(),
                    };
                    widget.borrow_mut().state.set_active(true);
                    fire_script_ref.borrow_mut().state.set_active(false);
                })));

            let area_editor_ref = Rc::clone(&self.area_editor);
            let disabled_ref = Rc::clone(&disabled);
            fire_script
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    area_editor_ref.borrow_mut().model.on_rest = OnRest::FireScript {
                        id: "<<PLACEHOLDER>>".to_string(),
                        func: "<<PLACEHOLDER>>".to_string(),
                    };
                    widget.borrow_mut().state.set_active(true);
                    disabled_ref.borrow_mut().state.set_active(false);
                })));

            Widget::add_child_to(&on_rest_box, disabled);
            Widget::add_child_to(&on_rest_box, fire_script);
        }
        Widget::add_child_to(&content, on_rest_box);

        let vis_dist_box = Widget::empty("vis_dist");
        {
            Widget::add_child_to(
                &vis_dist_box,
                Widget::with_theme(Label::empty(), "vis_dist_label"),
            );
            Widget::add_child_to(
                &vis_dist_box,
                Widget::with_theme(Label::empty(), "vis_dist_up_label"),
            );

            let value = self.area_editor.borrow().model.max_vis_distance;
            let vis_dist = Widget::with_theme(Spinner::new(value, 1, 30), "vis_dist_spinner");
            let area_editor_ref = Rc::clone(&self.area_editor);
            vis_dist
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |_, kind| {
                    let spinner = match kind.as_any_mut().downcast_mut::<Spinner>() {
                        Some(widget) => widget,
                        None => panic!("Failed to downcast to Spinner"),
                    };
                    area_editor_ref.borrow_mut().model.max_vis_distance = spinner.value();
                })));
            Widget::add_child_to(&vis_dist_box, vis_dist);

            let value = self.area_editor.borrow().model.max_vis_up_one_distance;
            let up_vis_dist = Widget::with_theme(Spinner::new(value, 1, 30), "vis_dist_up_spinner");
            let area_editor_ref = Rc::clone(&self.area_editor);
            up_vis_dist
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |_, kind| {
                    let spinner = match kind.as_any_mut().downcast_mut::<Spinner>() {
                        Some(widget) => widget,
                        None => panic!("Failed to downcast to Spinner"),
                    };
                    area_editor_ref.borrow_mut().model.max_vis_up_one_distance = spinner.value();
                })));
            Widget::add_child_to(&vis_dist_box, up_vis_dist);
        }
        Widget::add_child_to(&content, vis_dist_box);

        let world_map_box = Widget::empty("world_map_location");
        {
            Widget::add_child_to(&world_map_box, Widget::with_defaults(Label::empty()));
            let model = &self.area_editor.borrow().model;
            let loc = model.world_map_location.as_deref().unwrap_or_default();
            let field = Widget::with_defaults(InputField::new(loc));

            let area_editor_ref = Rc::clone(&self.area_editor);
            field
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |_widget, kind| {
                    let input_field = match kind.as_any_mut().downcast_mut::<InputField>() {
                        Some(input_field) => input_field,
                        None => panic!("Failed to downcast to InputField"),
                    };

                    let text = input_field.text.to_string();
                    if text.is_empty() {
                        area_editor_ref.borrow_mut().model.world_map_location = None;
                    } else {
                        area_editor_ref.borrow_mut().model.world_map_location = Some(text);
                    }
                })));
            Widget::add_child_to(&world_map_box, field);
        }
        Widget::add_child_to(&content, world_map_box);

        vec![close, save, content]
    }
}
