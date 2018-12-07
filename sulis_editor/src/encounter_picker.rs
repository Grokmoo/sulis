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
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_module::{Encounter, Module};
use sulis_widgets::{Button, Label, Spinner};

use crate::{AreaModel, EditorMode};

const NAME: &str = "encounter_picker";

pub struct EncounterPicker {
    cur_width: i32,
    cur_height: i32,
    cur_encounter: Option<Rc<Encounter>>,
    cursor_pos: Option<Point>,

    encounter_sprite: Option<Rc<Sprite>>,
}

impl EncounterPicker {
    pub fn new() -> Rc<RefCell<EncounterPicker>> {
        let enc_tile = Config::editor_config().area.encounter_tile;

        let sprite = match ResourceSet::get_sprite(&enc_tile) {
            Ok(sprite) => Some(sprite),
            Err(_) => {
                warn!("Encounter tile '{}' not found", enc_tile);
                None
            },
        };

        Rc::new(RefCell::new(EncounterPicker {
            cur_encounter: None,
            cursor_pos: None,
            encounter_sprite: sprite,
            cur_width: 10,
            cur_height: 10,
        }))
    }
}

impl EditorMode for EncounterPicker {
    fn draw_mode(&mut self, renderer: &mut GraphicsRenderer, _model: &AreaModel, x: f32, y: f32,
            scale_x: f32, scale_y: f32, _millis: u32) {

        match self.cur_encounter {
            None => return,
            Some(ref encounter) => encounter,
        };

        let pos = match self.cursor_pos {
            None => return,
            Some(pos) => pos,
        };

        if let Some(ref sprite) = self.encounter_sprite {
            let x = x + pos.x as f32;
            let y = y + pos.y as f32;
            let w = self.cur_width as f32;
            let h = self.cur_height as f32;
            let mut draw_list = DrawList::from_sprite_f32(sprite, x, y, w, h);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }
    }

    fn cursor_size(&self) -> (i32, i32) {
        (self.cur_width, self.cur_height)
    }

    fn mouse_move(&mut self, _model: &mut AreaModel, x: i32, y: i32) {
        self.cursor_pos = Some(Point::new(x, y));
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let encounter = match self.cur_encounter {
            None => return,
            Some(ref encounter) => encounter,
        };

        model.add_encounter(Rc::clone(encounter), x, y, self.cur_width, self.cur_height);
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        if self.cur_encounter.is_none() { return; }

        model.remove_encounters_within(x, y, self.cur_width, self.cur_height);
    }
}

impl WidgetKind for EncounterPicker {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let width = Widget::with_theme(Spinner::new(self.cur_width, 1, 50), "width");
        width.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<EncounterPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast to spinner"),
                Some(widget) => widget,
            };

            picker.cur_width = spinner.value();
        })));
        let height = Widget::with_theme(Spinner::new(self.cur_height, 1, 50), "height");
        height.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<EncounterPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast to spinner"),
                Some(widget) => widget,
            };

            picker.cur_height = spinner.value();
        })));

        let size_label = Widget::with_theme(Label::empty(), "size_label");

        let encounters = Widget::empty("encounters");
        {
            let mut all_encounters = Module::all_encounters();
            all_encounters.sort_by(|a, b| a.id.cmp(&b.id));

            for encounter in all_encounters {
                let button = Widget::with_theme(Button::empty(), "encounter_button");
                button.borrow_mut().state.add_text_arg("name", &encounter.id);
                button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                    let parent = Widget::go_up_tree(widget, 2);
                    let cur_state = widget.borrow_mut().state.is_active();
                    if !cur_state {
                        trace!("Set active encounter: {}", widget.borrow().state.text);
                        for child in parent.borrow().children.iter() {
                            child.borrow_mut().state.set_active(false);
                        }
                        widget.borrow_mut().state.set_active(true);
                    }

                    let encounter_picker = Widget::downcast_kind_mut::<EncounterPicker>(&parent);
                    encounter_picker.cur_encounter = Some(Rc::clone(&encounter));
                })));

                Widget::add_child_to(&encounters, button);
            }
        }

        vec![width, height, size_label, encounters]
    }
}
