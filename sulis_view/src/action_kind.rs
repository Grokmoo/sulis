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

use sulis_core::ui::{animation_state, Widget};
use sulis_core::util::Point;
use sulis_module::{Module, ObjectSize};
use sulis_state::{EntityState, GameState, ScriptCallback};
use RootView;

pub fn get_action(x: i32, y: i32) -> Box<ActionKind> {
    if let Some(action) = AttackAction::create_if_valid(x, y) { return action; }
    if let Some(action) = PropAction::create_if_valid(x, y) { return action; }
    if let Some(action) = TransitionAction::create_if_valid(x, y) { return action; }
    if let Some(action) = MoveAction::create_if_valid(x as f32, y as f32, None) { return action; }

    Box::new(InvalidAction {})
}

pub trait ActionKind {
    fn cursor_state(&self) -> animation_state::Kind;

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)>;

    fn fire_action(&mut self, widget: &Rc<RefCell<Widget>>);
}

struct PropAction {
    index: usize,
}

impl PropAction {
    fn create_if_valid(x: i32, y: i32) -> Option<Box<ActionKind>> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();

        let index = match area_state.prop_index_at(x, y) {
            None => return None,
            Some(index) => index,
        };

        let prop_state = area_state.get_prop(index);
        if !prop_state.prop.interactive { return None; }

        let max_dist = Module::rules().max_prop_distance;
        let pc = GameState::pc();
        if pc.borrow().dist_to_prop(prop_state) > max_dist {
            let cb_action = Box::new(PropAction { index });
            return MoveThenAction::create_if_valid(prop_state.location.to_point(),
                &prop_state.prop.size, max_dist, cb_action, animation_state::Kind::MouseActivate);
        }

        Some(Box::new(PropAction { index }))
    }
}

impl ActionKind for PropAction {
    fn cursor_state(&self) -> animation_state::Kind { animation_state::Kind::MouseActivate }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let prop = area_state.get_prop(self.index);
        let point = prop.location.to_point();
        Some((Rc::clone(&prop.prop.size), point.x, point.y))
    }

    fn fire_action(&mut self, widget: &Rc<RefCell<Widget>>) {
        let is_active = {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            let state = area_state.get_prop_mut(self.index);
            state.toggle_active();
            state.is_active()
        };

        let root = Widget::get_root(&widget);
        let view = Widget::downcast_kind_mut::<RootView>(&root);
        view.set_prop_window(&root, is_active, self.index);
    }
}

struct TransitionAction {
    x: i32,
    y: i32,
    to_x: i32,
    to_y: i32,
    area_id: Option<String>,
}

impl TransitionAction {
    fn create_if_valid(x: i32, y: i32) -> Option<Box<ActionKind>> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let transition = area_state.get_transition_at(x, y);
        let transition = match transition {
            None => return None,
            Some(ref transition) => transition,
        };

        let cb_action = Box::new(TransitionAction {
            area_id: transition.to_area.clone(),
            x, y,
            to_x: transition.to.x,
            to_y: transition.to.y,
        });

        let max_dist = Module::rules().max_transition_distance;
        let pc = GameState::pc();
        if pc.borrow().dist_to_transition(transition) > max_dist {
            return MoveThenAction::create_if_valid(transition.from,
                &transition.size, max_dist, cb_action, animation_state::Kind::MouseTravel);
        }

        Some(cb_action)
    }
}

impl ActionKind for TransitionAction {
    fn cursor_state(&self) -> animation_state::Kind { animation_state::Kind::MouseTravel }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let transition = area_state.get_transition_at(self.x, self.y);
        let transition = match transition {
            None => return None,
            Some(ref transition) => transition,
        };
        Some((Rc::clone(&transition.size), transition.from.x, transition.from.y))
    }

    fn fire_action(&mut self, widget: &Rc<RefCell<Widget>>) {
        trace!("Firing transition callback.");
        GameState::transition(&self.area_id, self.to_x, self.to_y);
        let root = Widget::get_root(widget);
        root.borrow_mut().invalidate_children();
    }
}

struct AttackAction {
    pc: Rc<RefCell<EntityState>>,
    target: Rc<RefCell<EntityState>>,
}

impl AttackAction {
    fn create_if_valid(x: i32, y: i32) -> Option<Box<ActionKind>> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let target = match area_state.get_entity_at(x, y) {
            None => return None,
            Some(ref entity) => {
                if entity.borrow().is_pc() { return None; }
                if !entity.borrow().is_hostile() { return None; }
                Rc::clone(entity)
            }
        };
        let pc = GameState::pc();
        if pc.borrow().can_attack(&target, &area_state.area) {
            Some(Box::new(AttackAction { pc, target }))
        } else {
            let cb_action = Box::new(AttackAction {
                pc: Rc::clone(&pc),
                target: Rc::clone(&target)
            });
            return MoveThenAction::create_if_valid(target.borrow().location.to_point(),
                &target.borrow().size, pc.borrow().actor.stats.attack_distance(), cb_action,
                animation_state::Kind::MouseAttack);
        }
    }
}

impl ActionKind for AttackAction {
    fn cursor_state(&self) -> animation_state::Kind { animation_state::Kind::MouseAttack }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        let size = Rc::clone(&self.target.borrow().size);
        let point = self.target.borrow().location.to_point();
        Some((size, point.x, point.y))
    }

    fn fire_action(&mut self, _widget: &Rc<RefCell<Widget>>) {
        trace!("Firing attack action.");
        let area_state = GameState::area_state();
        if !self.pc.borrow().can_attack(&self.target, &area_state.borrow().area) {
            return;
        }

        EntityState::attack(&self.pc, &self.target, None);
    }
}

struct ActionCallback {
    action: Box<ActionKind>,
    widget: Rc<RefCell<Widget>>,
}

impl ScriptCallback for ActionCallback {
    fn on_anim_complete(&mut self) {
        self.action.fire_action(&self.widget);
    }
}

struct MoveThenAction {
    move_action: MoveAction,
    cb_action: Option<Box<ActionKind>>,
    cursor_state: animation_state::Kind,
}

impl MoveThenAction {
    fn create_if_valid(pos: Point, size: &Rc<ObjectSize>, dist: f32, cb_action: Box<ActionKind>,
                       cursor_state: animation_state::Kind) -> Option<Box<ActionKind>> {
        let x = pos.x as f32 + size.width as f32 / 2.0 - 0.5;
        let y = pos.y as f32 + size.height as f32 / 2.0 - 0.5;

        let pc = GameState::pc();
        let dist = dist + pc.borrow().size.diagonal / 2.0 + size.diagonal / 2.0;
        let move_action = match MoveAction::new_if_valid(x, y, Some(dist)) {
            None => return None,
            Some(move_action) => move_action,
        };

        Some(Box::new(MoveThenAction {
            move_action,
            cb_action: Some(cb_action),
            cursor_state
        }))
    }
}

impl ActionKind for MoveThenAction {
    fn cursor_state(&self) -> animation_state::Kind {
        self.cursor_state
    }

    fn fire_action(&mut self, widget: &Rc<RefCell<Widget>>) {
        let action = match self.cb_action.take() {
            None => return,
            Some(action) => action,
        };

        let cb = ActionCallback {
            action,
            widget: Rc::clone(widget),
        };

        self.move_action.cb = Some(Box::new(cb));
        self.move_action.fire_action(widget);
    }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        match self.cb_action {
            None => None,
            Some(ref action) => action.get_hover_info()
        }
    }
}
struct MoveAction {
    pc: Rc<RefCell<EntityState>>,
    x: f32,
    y: f32,
    dist: f32,
    cb: Option<Box<ScriptCallback>>,
}

impl MoveAction {
    fn new_if_valid(x: f32, y: f32, dist: Option<f32>) -> Option<MoveAction> {
        let pc = GameState::pc();

        let dist = match dist {
            None => 0.6,
            Some(dist) => dist,
        };

        if !GameState::can_move_towards_point(&pc, x, y, dist) {
            return None;
        }

        Some(MoveAction { pc, x, y, dist, cb: None })
    }

    fn create_if_valid(x: f32, y: f32, dist: Option<f32>) -> Option<Box<ActionKind>> {
        match MoveAction::new_if_valid(x, y, dist) {
            None => None,
            Some(action) => Some(Box::new(action)),
        }
    }
}

impl ActionKind for MoveAction {
    fn cursor_state(&self) -> animation_state::Kind {
        animation_state::Kind::MouseMove
    }

    fn fire_action(&mut self, widget: &Rc<RefCell<Widget>>) {
        trace!("Firing move action");

        let cb = self.cb.take();
        GameState::move_towards_point(&self.pc, self.x, self.y, self.dist, cb);

        let root = Widget::get_root(widget);
        let view = Widget::downcast_kind_mut::<RootView>(&root);
        view.set_prop_window(&root, false, 0);
    }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        let size = Rc::clone(&self.pc.borrow().size);
        let x = self.x as i32 - size.width / 2;
        let y = self.y as i32 - size.height / 2;
        Some((size, x, y))
    }
}

struct InvalidAction { }

impl ActionKind for InvalidAction {
    fn cursor_state(&self) -> animation_state::Kind {
        animation_state::Kind::MouseInvalid
    }

    fn fire_action(&mut self, _widget: &Rc<RefCell<Widget>>) { }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> { None }
}
