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

use sulis_engine::ui::{Callback, Cursor, Widget, WidgetKind};
use sulis_engine::io::event::ClickKind;
use sulis_engine::util::Point;
use sulis_module::Module;
use sulis_state::{ChangeListener, GameState, EntityState};
use sulis_widgets::{Label, list_box, ListBox};

const NAME: &'static str = "action_menu";

pub struct ActionMenu {
    hovered_entity: Option<Rc<RefCell<EntityState>>>,
    is_hover_pc: bool,
    area_pos: Point,
}

impl ActionMenu {
    pub fn new(x: i32, y: i32) -> Rc<ActionMenu> {
        let area_state = GameState::area_state();
        let (hovered_entity, is_hover_pc) =
            if let Some(ref entity) = area_state.borrow().get_entity_at(x, y) {
                (Some(Rc::clone(entity)), entity.borrow().is_pc())
            } else {
                (None, false)
            };
        Rc::new(ActionMenu {
            area_pos: Point::new(x, y),
            hovered_entity,
            is_hover_pc,
        })
    }

    pub fn is_transition_valid(&self) -> bool {
        let area_state = GameState::area_state();
        if let Some(transition) = area_state.borrow()
            .get_transition_at(self.area_pos.x, self.area_pos.y) {
                let pc = GameState::pc();
                return pc.borrow().dist_to_transition(transition) < 2.5;
            }
        false
    }

    pub fn transition_callback(&self) -> Option<Callback> {
        let area_state = GameState::area_state();
        let (area_id, x, y) = match area_state.borrow()
            .get_transition_at(self.area_pos.x, self.area_pos.y) {
                None => return None,
                Some(transition) => (transition.to_area.clone(), transition.to.x, transition.to.y)
            };

        Some(Callback::new(Rc::new( move |widget| {
            trace!("Firing transition callback.");
            GameState::transition(&area_id, x, y);
            Widget::mark_removal_up_tree(&widget, 2);
            let root = Widget::get_root(&widget);
            root.borrow_mut().invalidate_children();
        })))
    }

    pub fn is_attack_valid(&self) -> bool {
        if self.is_hover_pc { return false; }
        let pc = GameState::pc();

        match self.hovered_entity {
            None => false,
            Some(ref entity) => pc.borrow().can_attack(entity),
        }
    }

    pub fn attack_callback(&self) -> Box<Fn()> {
        if let Some(ref entity) = self.hovered_entity {
            let entity_ref = Rc::clone(entity);
            Box::new(move || {
                trace!("Firing attack callback.");
                let pc = GameState::pc();
                EntityState::attack(&pc, &entity_ref);
            })
        } else {
            Box::new(|| { })
        }
    }

    pub fn is_move_valid(&self) -> bool {
        let pc = GameState::pc();
        if pc.borrow().actor.ap() < Module::rules().movement_ap {
            return false;
        }

        GameState::can_move_to(&pc, self.area_pos.x, self.area_pos.y)
    }

    pub fn move_callback(&self) -> Box<Fn()> {
        let pc = GameState::pc();
        let x = self.area_pos.x;
        let y = self.area_pos.y;
        Box::new(move || {
            trace!("Firing move callback.");
            GameState::move_to(&pc, x, y);
        })
    }

    fn callback_with_removal(f: Box<Fn()>) -> Option<Callback> {
        Some(Callback::new(Rc::new(move |widget| {
            f();
            Widget::mark_removal_up_tree(&widget, 2);
        })))
    }

    pub fn fire_default_callback(&self) {
        if self.is_attack_valid() {
            (self.attack_callback())();
        } else if self.is_move_valid() {
            (self.move_callback())();
        }
    }

    pub fn is_default_callback_valid(&self) -> bool {
        self.is_attack_valid() || self.is_move_valid()
    }
}

impl WidgetKind for ActionMenu {
    fn get_name(&self) -> &str {
        NAME
    }

    fn on_remove(&self) {
        let area_state = GameState::area_state();
        area_state.borrow_mut().listeners.remove(NAME);
    }

    fn on_add(&self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let area_state = GameState::area_state();
        area_state.borrow_mut().listeners.add(ChangeListener::remove_widget(NAME, widget));

        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        if self.is_move_valid() {
            entries.push(list_box::Entry::new(
                    "Move".to_string(), ActionMenu::callback_with_removal(self.move_callback())));

        }

        if self.is_attack_valid() {
            entries.push(list_box::Entry::new(
                    "Attack".to_string(), ActionMenu::callback_with_removal(self.attack_callback())));
        }

        if self.is_transition_valid() {
            entries.push(list_box::Entry::new(
                    "Transition".to_string(), self.transition_callback()));
        }

        if entries.is_empty() {
            entries.push(list_box::Entry::new(
                    "None".to_string(), ActionMenu::callback_with_removal(Box::new(|| { }))));
        }
        let actions = Widget::with_theme(ListBox::new(entries), "actions");

        vec![title, actions]
    }

    fn on_mouse_release(&self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        if !widget.borrow().state.in_bounds(Cursor::get_x(), Cursor::get_y()) {
            widget.borrow_mut().mark_for_removal();
        }

        true
    }
}
