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
use std::cell::{RefCell};

use sulis_core::util::Size;
use sulis_core::ui::{Callback, Cursor, Widget, WidgetKind};
use sulis_core::widgets::{Button, Label};
use sulis_state::{ChangeListener, GameState};

pub const NAME: &str = "formation_window";

struct Entry {
    child: Rc<RefCell<Widget>>,
    position: (f32, f32),
}

pub struct FormationWindow {
    entries: Vec<Entry>,
    grid_half_width: f32,
    grid_height: f32,
    button_size: f32,
    active_entry: Option<usize>,

    grid_size: (f32, f32),
    grid_offset: (f32, f32),
}

impl FormationWindow {
    pub fn new() -> Rc<RefCell<FormationWindow>> {
        Rc::new(RefCell::new(FormationWindow {
            entries: Vec::new(),
            grid_half_width: 10.0,
            grid_height: 10.0,
            button_size: 2.0,
            active_entry: None,
            grid_size: (0.0, 0.0),
            grid_offset: (0.0, 0.0),
        }))
    }

    fn set_active_entry(&mut self, index: Option<usize>) {
        if let Some(index) = self.active_entry {
            let formation = GameState::party_formation();
            formation.borrow_mut().set_position(index, self.entries[index].position);
        }

        self.active_entry = index;
        if let Some(index) = self.active_entry {
            for (entry_index, entry) in self.entries.iter().enumerate() {
                entry.child.borrow_mut().state.set_enabled(index  == entry_index);
            }
        } else {
            for entry in self.entries.iter() {
                entry.child.borrow_mut().state.set_enabled(true);
            }
        }
    }
}

impl WidgetKind for FormationWindow {
    widget_kind!(NAME);

    fn layout(&mut self, widget: &mut Widget) {
        let theme = &widget.theme;
        self.grid_half_width = theme.get_custom_or_default("grid_half_width", 10.0);
        self.grid_height = theme.get_custom_or_default("grid_height", 10.0);
        self.button_size = theme.get_custom_or_default("button_size", 2.0);

        widget.do_self_layout();

        let w = widget.state.inner_width() as f32;
        let h = widget.state.inner_height() as f32;

        self.grid_size = (w / (1.0 + self.grid_half_width * 2.0), h / self.grid_height);
        self.grid_offset = (self.grid_size.0 * self.grid_half_width, 0.0);

        self.grid_size = (self.grid_size.0.floor(), self.grid_size.1.floor());
        self.grid_offset = (self.grid_offset.0.floor(), self.grid_offset.1.floor());

        for entry in self.entries.iter() {
            let x = entry.position.0 * self.grid_size.0 + self.grid_offset.0;
            let y = entry.position.1 * self.grid_size.1 + self.grid_offset.1;
            entry.child.borrow_mut().state.set_position(widget.state.inner_left() + x as i32,
                                                        widget.state.inner_top() + y as i32);
            entry.child.borrow_mut().state.set_size(Size::new((self.grid_size.0 * self.button_size) as i32,
                (self.grid_size.1 * self.button_size) as i32));
        }
        widget.do_children_layout();
    }

    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>, _dx: f32, _dy: f32) -> bool {
        let index = match self.active_entry {
            None => return true,
            Some(index) => index,
        };
        let cur_grid_pos = self.entries[index].position;

        // compute the new grid position
        let x = Cursor::get_x_f32() - widget.borrow().state.inner_left() as f32;
        let y = Cursor::get_y_f32() - widget.borrow().state.inner_top() as f32;

        let mut grid_pos = (
            ((x - self.grid_offset.0) / self.grid_size.0 - self.button_size / 4.0).floor(),
            ((y - self.grid_offset.1) / self.grid_size.1 - self.button_size / 4.0).floor()
        );
        if grid_pos.0 < -self.grid_half_width {
            grid_pos.0 = -self.grid_half_width;
        }
        if grid_pos.0 > self.grid_half_width - self.button_size + 1.0 {
            grid_pos.0 = self.grid_half_width - self.button_size + 1.0;
        }
        if grid_pos.1 < 0.0 {
            grid_pos.1 = 0.0;
        }
        if grid_pos.1 > self.grid_height - self.button_size + 1.0 {
            grid_pos.1 = self.grid_height - self.button_size + 1.0;
        }

        if cur_grid_pos != grid_pos {
            //verify the new grid position is not blocked
            let r1 = (grid_pos.0, grid_pos.1, self.button_size, self.button_size);

            for (i, entry) in self.entries.iter().enumerate() {
                if i == index { continue; }

                let r2 = (entry.position.0, entry.position.1, self.button_size, self.button_size);

                // if one rectangle is on the left side of the other
                if r1.0 >= r2.0 + r2.2 || r2.0 >= r1.0 + r1.2 { continue; }

                // if one rectangle is above the other
                if r1.1 >= r2.1 + r2.3 || r2.1 >= r1.1 + r1.3 { continue; }

                // the rectangles overlap
                return true;
            }

            self.entries[index].position = (grid_pos.0, grid_pos.1);
            widget.borrow_mut().invalidate_layout();
        }
        true
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let widget_ref = Rc::clone(widget);
        GameState::add_party_listener(ChangeListener::new(NAME, Box::new(move |_| {
            widget_ref.borrow_mut().invalidate_children();
        })));

        let title = Widget::with_theme(Label::empty(), "title");

        let close = Widget::with_theme(Button::empty(), "close");
        close.borrow_mut().state.add_callback(Callback::remove_parent());

        let mut children = vec![title, close];

        self.entries.clear();
        let party = GameState::party();
        let formation = GameState::party_formation();
        let formation = formation.borrow();
        for (index, (x, y)) in formation.positions_iter().enumerate() {
            let button = Widget::with_theme(Button::empty(), "position");
            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::get_parent(widget);
                let window = Widget::downcast_kind_mut::<FormationWindow>(&parent);
                if !window.active_entry.is_some() {
                    window.set_active_entry(Some(index));
                } else {
                    window.set_active_entry(None);
                }
            })));

            let label = Widget::with_theme(Label::empty(), "portrait");
            if let Some(entity) = party.get(index) {
                if let Some(ref image) = entity.borrow().actor.actor.portrait {
                    label.borrow_mut().state.add_text_arg("portrait", &image.id());
                }
            }
            Widget::add_child_to(&button, label);

            let entry = Entry {
                child: Rc::clone(&button),
                position: (*x, *y),
            };
            self.entries.push(entry);
            children.push(button);
        }

        children
    }
}
