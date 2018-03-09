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

use sulis_core::ui::{animation_state, Callback, Cursor, Widget, WidgetKind};
use sulis_core::io::event::ClickKind;
use sulis_core::util::Point;
use sulis_module::{Area, Module, ObjectSize};
use sulis_state::{ChangeListener, GameState, EntityState};
use sulis_widgets::{Label, list_box, ListBox};

use {RootView};

const NAME: &'static str = "action_menu";

pub struct ActionMenu {
    hovered_entity: Option<Rc<RefCell<EntityState>>>,
    hovered_prop: Option<usize>,
    is_hover_pc: bool,
    area: Rc<Area>,
    area_pos: Point,
}

impl ActionMenu {
    pub fn new(x: i32, y: i32) -> Rc<RefCell<ActionMenu>> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let (hovered_entity, is_hover_pc) =
            if let Some(ref entity) = area_state.get_entity_at(x, y) {
                (Some(Rc::clone(entity)), entity.borrow().is_pc())
            } else {
                (None, false)
            };

        let hovered_prop = match area_state.prop_index_at(x, y) {
            None => None,
            Some(index) => {
                let prop_state = &area_state.props[index];
                if prop_state.prop.interactive {
                    Some(index)
                } else {
                    None
                }
            }
        };

        Rc::new(RefCell::new(ActionMenu {
            area_pos: Point::new(x, y),
            area: Rc::clone(&area_state.area),
            hovered_entity,
            is_hover_pc,
            hovered_prop,
        }))
    }

    pub fn is_transition_valid(&self) -> bool {
        let max_dist = Module::rules().max_transition_distance;

        let area_state = GameState::area_state();
        if let Some(transition) = area_state.borrow()
            .get_transition_at(self.area_pos.x, self.area_pos.y) {
                let pc = GameState::pc();
                return pc.borrow().dist_to_transition(transition) < max_dist;
            }
        false
    }

    pub fn transition_callback(&self) -> Callback {
        let area_state = GameState::area_state();
        let (area_id, x, y) = match area_state.borrow()
            .get_transition_at(self.area_pos.x, self.area_pos.y) {
                None => return Callback::empty(),
                Some(transition) => (transition.to_area.clone(), transition.to.x, transition.to.y)
            };

        Callback::new(Rc::new( move |widget, _kind| {
            trace!("Firing transition callback.");
            GameState::transition(&area_id, x, y);
            let root = Widget::get_root(&widget);
            root.borrow_mut().invalidate_children();
        }))
    }

    pub fn is_attack_valid(&self) -> bool {
        if self.is_hover_pc { return false; }
        let pc = GameState::pc();

        match self.hovered_entity {
            None => false,
            Some(ref entity) => pc.borrow().can_attack(entity, &self.area),
        }
    }

    pub fn attack_callback(&self) -> Callback {
        if let Some(ref entity) = self.hovered_entity {
            let entity_ref = Rc::clone(entity);
            Callback::new(Rc::new(move |_, _| {
                trace!("Firing attack callback.");
                let pc = GameState::pc();
                EntityState::attack(&pc, &entity_ref);
            }))
        } else {
            Callback::empty()
        }
    }

    pub fn is_move_valid(&self) -> bool {
        let pc = GameState::pc();
        if pc.borrow().actor.ap() < Module::rules().movement_ap {
            return false;
        }

        GameState::can_move_to(&pc, self.area_pos.x, self.area_pos.y)
    }

    pub fn move_callback(&self) -> Callback {
        let pc = GameState::pc();
        let x = self.area_pos.x;
        let y = self.area_pos.y;
        Callback::new(Rc::new(move |widget, _| {
            trace!("Firing move callback.");
            GameState::move_to(&pc, x, y);

            let root = Widget::get_root(&widget);
            let view = Widget::downcast_kind_mut::<RootView>(&root);
            view.set_prop_window(&root, false, 0);
        }))
    }

    pub fn is_prop_valid(&self) -> bool {
        let prop_index = match self.hovered_prop {
            None => return false,
            Some(index) => index,
        };

        let max_dist = Module::rules().max_prop_distance;
        let area_state = GameState::area_state();
        let prop_state = &area_state.borrow().props[prop_index];
        let pc = GameState::pc();
        let pc = pc.borrow();

        pc.dist_to_prop(prop_state) < max_dist
    }

    pub fn prop_callback(&self) -> Callback {
        if self.hovered_prop.is_none() {
            return Callback::empty();
        }

        let index = self.hovered_prop.unwrap();
        Callback::new(Rc::new(move |widget, _| {
            let active = {
                let area_state = GameState::area_state();
                let mut area_state = area_state.borrow_mut();
                area_state.props[index].toggle_active();
                area_state.props[index].is_active()
            };

            let root = Widget::get_root(&widget);
            let view = Widget::downcast_kind_mut::<RootView>(&root);
            view.set_prop_window(&root, active, index);
        }))
    }

    fn callback_add_removal(cb: Callback) -> Option<Callback> {
        Some(Callback::new(Rc::new(move |widget, kind| {
            cb.call(widget, kind);
            Widget::mark_removal_up_tree(&widget, 2);
        })))
    }

    pub fn fire_default_callback(&self, widget: &Rc<RefCell<Widget>>, kind: &mut WidgetKind) {
        if self.is_attack_valid() {
            self.attack_callback().call(widget, kind);
        } else if self.is_transition_valid() {
            self.transition_callback().call(widget, kind);
        } else if self.is_prop_valid() {
            self.prop_callback().call(widget, kind);
        } else if self.is_move_valid() {
            self.move_callback().call(widget, kind);
        }
    }

    pub fn is_default_callback_valid(&self) -> bool {
        self.is_attack_valid() || self.is_transition_valid() ||
            self.is_prop_valid() || self.is_move_valid()
    }

    pub fn get_cursor(&self) -> (Rc<ObjectSize>, Point) {
        let x = self.area_pos.x;
        let y = self.area_pos.y;

        let state = GameState::area_state();
        let state = state.borrow();

        if let Some(ref entity) = self.hovered_entity {
            if entity.borrow().is_pc() {
                Cursor::set_cursor_state(animation_state::Kind::MouseSelect);
            } else {
                Cursor::set_cursor_state(animation_state::Kind::MouseAttack);
            }
            let size = Rc::clone(&entity.borrow().size);
            (size, entity.borrow().location.to_point())
        } else if let Some(ref transition) = state.get_transition_at(x, y) {
            Cursor::set_cursor_state(animation_state::Kind::MouseTravel);
            (Rc::clone(&transition.size), transition.from)
        } else if let Some(index) = self.hovered_prop {
            Cursor::set_cursor_state(animation_state::Kind::MouseActivate);
            let prop = &state.props[index];
            (Rc::clone(&prop.prop.size), prop.location.to_point())
        } else {
            Cursor::set_cursor_state(animation_state::Kind::MouseMove);
            let pc = GameState::pc();
            let size = Rc::clone(&pc.borrow().size);
            let pos = Point::new(self.area_pos.x - size.width / 2,
                                 self.area_pos.y - size.height / 2);
            (size, pos)
        }
    }
}

impl WidgetKind for ActionMenu {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_remove(&mut self) {
        let area_state = GameState::area_state();
        area_state.borrow_mut().listeners.remove(NAME);
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let area_state = GameState::area_state();
        area_state.borrow_mut().listeners.add(ChangeListener::remove_widget(NAME, widget));

        widget.borrow_mut().state.set_modal(true);

        let title = Widget::with_theme(Label::empty(), "title");

        let mut entries: Vec<list_box::Entry<String>> = Vec::new();
        if self.is_move_valid() {
            entries.push(list_box::Entry::new(
                    "Move".to_string(), ActionMenu::callback_add_removal(self.move_callback())));

        }

        if self.is_attack_valid() {
            entries.push(list_box::Entry::new(
                    "Attack".to_string(), ActionMenu::callback_add_removal(self.attack_callback())));
        }

        if self.is_transition_valid() {
            entries.push(list_box::Entry::new(
                    "Transition".to_string(), ActionMenu::callback_add_removal(self.transition_callback())));
        }

        if self.is_prop_valid() {
            entries.push(list_box::Entry::new(
                    "Activate".to_string(), ActionMenu::callback_add_removal(self.prop_callback())));
        }

        if entries.is_empty() {
            entries.push(list_box::Entry::new(
                    "None".to_string(), ActionMenu::callback_add_removal(Callback::empty())));
        }
        let actions = Widget::with_theme(ListBox::new(entries), "actions");

        vec![title, actions]
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        self.super_on_mouse_release(widget, kind);
        if !widget.borrow().state.in_bounds(Cursor::get_x(), Cursor::get_y()) {
            widget.borrow_mut().mark_for_removal();
        }

        true
    }
}
