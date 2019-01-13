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

use sulis_core::ui::{Callback, Widget, WidgetKind, animation_state};
use sulis_widgets::{Label, Button, TextArea};
use sulis_rules::Time;
use sulis_module::Module;
use sulis_state::GameState;

pub const NAME: &str = "world_map_window";

pub struct Entry {
    child: Rc<RefCell<Widget>>,
    label: Rc<RefCell<Widget>>,
    position: (f32, f32),
}

pub struct WorldMapWindow {
    entries: Vec<Entry>,
    size: (f32, f32),
    offset: (f32, f32),
    content: Rc<RefCell<Widget>>,
    transition_enabled: bool,
}

impl WorldMapWindow {
    pub fn new(transition_enabled: bool) -> Rc<RefCell<WorldMapWindow>> {
        Rc::new(RefCell::new(WorldMapWindow {
            entries: Vec::new(),
            size: (0.0, 0.0),
            offset: (0.0, 0.0),
            content: Widget::empty("content"),
            transition_enabled,
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
                entry.label.borrow_mut().state.set_position(x, y);
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
        let map_state = GameState::world_map();

        self.content = Widget::empty("content");
        self.entries.clear();
        self.size = map.size;
        self.offset = map.offset;

        let area_state = GameState::area_state();
        let cur_location_id = area_state.borrow().area.world_map_location.clone();

        for location in map.locations.iter() {
            let button = Widget::with_theme(Button::empty(), "location");

            let (add_callback, label) = {
                let state = &mut button.borrow_mut().state;
                state.add_text_arg("name", &location.name);
                state.add_text_arg("icon", &location.icon.id());

                let is_active = if let Some(ref location_id) = &cur_location_id {
                    &location.id == location_id
                } else {
                    false
                };

                state.set_active(is_active);

                let (is_enabled, is_visible) =
                    (map_state.is_enabled(&location.id), map_state.is_visible(&location.id));
                state.set_enabled(is_enabled);
                state.set_visible(is_visible);

                if !self.transition_enabled {
                    state.animation_state.add(animation_state::Kind::Custom1);
                }

                let label = Widget::with_theme(TextArea::empty(), "label");
                label.borrow_mut().state.add_text_arg("name", &location.name);
                label.borrow_mut().state.set_visible(is_visible);

                (self.transition_enabled && is_enabled && is_visible && !is_active, label)
            };

            if add_callback {
                let mut travel_time = Time { day: 0, hour: 0, round: 0, millis: 0 };
                if let Some(cur_location) = &cur_location_id {
                    if let Some(hours) = location.travel_times.get(cur_location) {
                        travel_time.hour = *hours;
                        Module::rules().canonicalize_time(&mut travel_time);

                        label.borrow_mut().state.add_text_arg("travel_time",
                                                              &travel_time.to_string());
                    }
                }

                let (x, y) = (location.linked_area_pos.x, location.linked_area_pos.y);
                let area_id = location.linked_area.clone();
                button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                    let area_id = match area_id {
                        None => return,
                        Some(ref id) => id.clone(),
                    };

                    GameState::transition(&Some(area_id), x, y, travel_time);
                    let root = Widget::get_root(&widget);
                    root.borrow_mut().invalidate_children();
                })));
            }

            let entry = Entry {
                child: Rc::clone(&button),
                label: Rc::clone(&label),
                position: location.position,
            };

            self.entries.push(entry);
            Widget::add_child_to(&self.content, button);
        }

        // add labels after buttons so they show up on top
        for entry in self.entries.iter() {
            Widget::add_child_to(&self.content, Rc::clone(&entry.label));
        }

        vec![label, bg, close, labels, Rc::clone(&self.content)]
    }
}
