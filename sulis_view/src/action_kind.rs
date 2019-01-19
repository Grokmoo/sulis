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
use sulis_rules::Time;
use sulis_module::{Faction, Module, ObjectSize, area::ToKind};
use sulis_state::{MOVE_TO_THRESHOLD, EntityState, GameState, ScriptCallback, AreaState, PropState};
use crate::{dialog_window, RootView};

pub fn get_action(x: i32, y: i32) -> Box<ActionKind> {
    let area_state = GameState::area_state();
    if !area_state.borrow().area.coords_valid(x, y) { return Box::new(InvalidAction {}); }
    if !area_state.borrow().is_pc_explored(x, y) { return Box::new(InvalidAction {}); }

    if let Some(action) = SelectAction::create_if_valid(x, y) { return action; }
    if let Some(action) = AttackAction::create_if_valid(x, y) { return action; }
    if let Some(action) = DialogAction::create_if_valid(x, y) { return action; }

    if let Some(action) = get_prop_or_transition_action(x, y) { return action; }

    // if let Some(action) = LootPropAction::create_if_valid(x, y) { return action; }
    //
    // // open door
    // if let Some(action) = DoorPropAction::create_if_valid(x, y, true) { return action; }
    //
    // if let Some(action) = TransitionAction::create_if_valid(x, y) { return action; }
    //
    // // close door
    // if let Some(action) = DoorPropAction::create_if_valid(x, y, false) { return action; }

    if let Some(action) = MoveAction::create_if_valid(x as f32, y as f32, None) { return action; }

    Box::new(InvalidAction {})
}

fn get_prop_or_transition_action(x: i32, y: i32) -> Option<Box<ActionKind>> {
    let area_state = GameState::area_state();
    let area_state = area_state.borrow();

    let index = match area_state.prop_index_at(x, y) {
        None => return TransitionAction::create_if_valid(x, y, &area_state),
        Some(index) => index,
    };
    let prop = area_state.get_prop(index);

    // an enabled container or a closed door (regardless of enabled) blocks a transition.
    // an open door (regardless of enabled) does not block a transition

    if prop.is_container() && prop.is_enabled() {
        return LootPropAction::create_if_valid(index, &prop);
    }

    if prop.is_door() {
        if !prop.is_active() {
            // open door action (if enabled)
            return DoorPropAction::create_if_valid(index, &prop);
        }

        if let Some(action) = TransitionAction::create_if_valid(x, y, &area_state) {
            return Some(action);
        }

        // close door action (if enabled)
        return DoorPropAction::create_if_valid(index, &prop);
    }

    TransitionAction::create_if_valid(x, y, &area_state)
}

pub trait ActionKind {
    fn cursor_state(&self) -> animation_state::Kind;

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)>;

    fn fire_action(&mut self, widget: &Rc<RefCell<Widget>>);
}

struct SelectAction {
    target: Rc<RefCell<EntityState>>,
}

impl SelectAction {
    fn create_if_valid(x: i32, y: i32) -> Option<Box<ActionKind>> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let target = match area_state.get_entity_at(x, y) {
            None => return None,
            Some(ref entity) => {
                if !entity.borrow().is_party_member() { return None; }
                Rc::clone(entity)
            }
        };

        Some(Box::new(SelectAction {
            target,
        }))
    }
}

impl ActionKind for SelectAction{
    fn cursor_state(&self) -> animation_state::Kind {
        animation_state::Kind::MouseSelect
    }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        let size = Rc::clone(&self.target.borrow().size);
        let point = self.target.borrow().location.to_point();
        Some((size, point.x, point.y))
    }

    fn fire_action(&mut self, _widget: &Rc<RefCell<Widget>>) {
        trace!("Firing select action.");
        GameState::set_selected_party_member(Rc::clone(&self.target));
    }
}

struct DialogAction {
    target: Rc<RefCell<EntityState>>,
    pc: Rc<RefCell<EntityState>>,
}

impl DialogAction {
    fn create_if_valid(x: i32, y: i32) -> Option<Box<ActionKind>> {
        if GameState::is_combat_active() { return None; }

        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let target = match area_state.get_entity_at(x, y) {
            None => return None,
            Some(ref entity) => {
                if entity.borrow().is_party_member() { return None; }
                if entity.borrow().actor.actor.conversation.is_none() { return None; }
                Rc::clone(entity)
            }
        };
        let max_dist = Module::rules().max_dialog_distance;
        let pc = match GameState::selected().first() {
            None => return None,
            Some(pc) => Rc::clone(pc),
        };

        let dist = pc.borrow().dist_to_entity(&target);
        if dist <= max_dist {
            Some(Box::new(DialogAction { target, pc }))
        } else {
            let cb_action = Box::new(DialogAction {
                target: Rc::clone(&target),
                pc: Rc::clone(&pc),
            });
            return MoveThenAction::create_if_valid(&pc, target.borrow().location.to_point(),
                &target.borrow().size, max_dist, cb_action,
                animation_state::Kind::MouseDialog);
        }
    }
}

impl ActionKind for DialogAction {
    fn cursor_state(&self) -> animation_state::Kind {
        animation_state::Kind::MouseDialog
    }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        let size = Rc::clone(&self.target.borrow().size);
        let point = self.target.borrow().location.to_point();
        Some((size, point.x, point.y))
    }

    fn fire_action(&mut self, widget: &Rc<RefCell<Widget>>) {
        trace!("Firing dialog action.");

        let convo = match self.target.borrow().actor.actor.conversation {
            None => {
                warn!("Attempted to fire conversation action with entity with no convo");
                return;
            }, Some(ref convo) => Rc::clone(convo),
        };

        dialog_window::show_convo(convo, &self.pc, &self.target, widget);
    }
}

struct DoorPropAction {
    index: usize,
}

impl DoorPropAction {
    fn create_if_valid(index: usize, prop_state: &PropState) -> Option<Box<ActionKind>> {
        if !prop_state.is_door() || !prop_state.is_enabled() { return None; }

        let max_dist = Module::rules().max_prop_distance;
        let pc = match GameState::selected().first() {
            None => return None,
            Some(pc) => Rc::clone(pc),
        };
        if pc.borrow().dist_to_prop(prop_state) > max_dist {
            let cb_action = Box::new(DoorPropAction { index });
            return MoveThenAction::create_if_valid(&pc, prop_state.location.to_point(),
                &prop_state.prop.size, max_dist, cb_action, animation_state::Kind::MouseInteract);
        }

        Some(Box::new(DoorPropAction { index }))
    }
}

impl ActionKind for DoorPropAction {
    fn cursor_state(&self) -> animation_state::Kind { animation_state::Kind::MouseInteract }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let prop = area_state.get_prop(self.index);
        let point = prop.location.to_point();
        Some((Rc::clone(&prop.prop.size), point.x, point.y))
    }

    fn fire_action(&mut self, _widget: &Rc<RefCell<Widget>>) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();
        area_state.toggle_prop_active(self.index);
    }
}

struct LootPropAction {
    index: usize,
}

impl LootPropAction {
    fn create_if_valid(index: usize, prop_state: &PropState) -> Option<Box<ActionKind>> {
        if GameState::is_combat_active() { return None; }

        if !prop_state.is_container() || !prop_state.is_enabled() { return None; }

        let max_dist = Module::rules().max_prop_distance;
        let pc = match GameState::selected().first() {
            None => return None,
            Some(pc) => Rc::clone(pc),
        };
        if pc.borrow().dist_to_prop(prop_state) > max_dist {
            let cb_action = Box::new(LootPropAction { index });
            return MoveThenAction::create_if_valid(&pc, prop_state.location.to_point(),
                &prop_state.prop.size, max_dist, cb_action, animation_state::Kind::MouseLoot);
        }

        Some(Box::new(LootPropAction { index }))
    }
}

impl ActionKind for LootPropAction {
    fn cursor_state(&self) -> animation_state::Kind { animation_state::Kind::MouseLoot }

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
    to: ToKind,
}

impl TransitionAction {
    fn create_if_valid(x: i32, y: i32, area_state: &AreaState) -> Option<Box<ActionKind>> {
        if GameState::is_combat_active() { return None; }

        let transition = area_state.get_transition_at(x, y);
        let transition = match transition {
            None => return None,
            Some(ref transition) => transition,
        };

        let cb_action = Box::new(TransitionAction {
            x,
            y,
            to: transition.to.clone(),
        });

        let max_dist = Module::rules().max_transition_distance;
        let pc = match GameState::selected().first() {
            None => return None,
            Some(pc) => Rc::clone(pc),
        };
        if pc.borrow().dist_to_transition(transition) > max_dist {
            return MoveThenAction::create_if_valid(&pc, transition.from,
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
        let time = Time { day: 0, hour: 0, round: 0, millis: 0 };
        match self.to {
            ToKind::Area { ref id, x, y } => {
                GameState::transition(&Some(id.to_string()), x, y, time);
                let root = Widget::get_root(widget);
                root.borrow_mut().invalidate_children();
            },
            ToKind::CurArea { x, y } => {
                GameState::transition(&None, x, y, time);
            },
            ToKind::WorldMap => {
                let root = Widget::get_root(widget);
                let view = Widget::downcast_kind_mut::<RootView>(&root);
                view.set_map_window(&root, true, true);
            }
        }
    }
}

struct AttackAction {
    pc: Rc<RefCell<EntityState>>,
    target: Rc<RefCell<EntityState>>,
}

fn get_attack_target(area_state: &AreaState, x: i32, y: i32) -> Option<Rc<RefCell<EntityState>>> {
    match area_state.get_entity_at(x, y) {
        None => None,
        Some(ref entity) => {
            if entity.borrow().actor.hp() <= 0 { return None; }
            if entity.borrow().is_party_member() { return None; }
            if entity.borrow().actor.faction() != Faction::Hostile { return None; }
            Some(Rc::clone(entity))
        }
    }
}

impl AttackAction {
    fn create_if_valid(x: i32, y: i32) -> Option<Box<ActionKind>> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let target = match get_attack_target(&area_state, x, y) {
            None => return None,
            Some(target) => target,
        };
        let pc = match GameState::selected().first() {
            None => return None,
            Some(pc) => Rc::clone(pc),
        };

        if !pc.borrow().actor.has_ap_to_attack() { return None; }

        if pc.borrow().can_attack(&target, &area_state) {
            Some(Box::new(AttackAction { pc, target }))
        } else {
            let cb_action = Box::new(AttackAction {
                pc: Rc::clone(&pc),
                target: Rc::clone(&target)
            });
            return MoveThenAction::create_if_valid(&pc, target.borrow().location.to_point(),
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
        if !self.pc.borrow().can_attack(&self.target, &area_state.borrow()) {
            return;
        }

        EntityState::attack(&self.pc, &self.target, None, true);
    }
}

struct ActionCallback {
    action: Rc<RefCell<Box<ActionKind>>>,
    widget: Rc<RefCell<Widget>>,
}

impl ScriptCallback for ActionCallback {
    fn on_anim_complete(&self) {
        self.action.borrow_mut().fire_action(&self.widget);
    }
}

struct MoveThenAction {
    move_action: MoveAction,
    cb_action: Option<Box<ActionKind>>,
    cursor_state: animation_state::Kind,
}

impl MoveThenAction {
    fn create_if_valid(pc: &Rc<RefCell<EntityState>>,
                       pos: Point,
                       size: &Rc<ObjectSize>,
                       dist: f32,
                       cb_action: Box<ActionKind>,
                       cursor_state: animation_state::Kind) -> Option<Box<ActionKind>> {
        let (px, py) = (pos.x as f32, pos.y as f32);

        let x = px + (size.width / 2) as f32;
        let y = py + (size.height / 2) as f32;

        let dist = dist + (size.diagonal + pc.borrow().size.diagonal) / 2.0;
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
            Some(action) => Rc::new(RefCell::new(action)),
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
    selected: Vec<Rc<RefCell<EntityState>>>,
    x: f32,
    y: f32,
    dist: f32,
    cb: Option<Box<ScriptCallback>>,
}

fn entities_to_ignore() -> Vec<usize> {
    if GameState::is_combat_active() {
        Vec::new()
    } else {
        GameState::party().iter().map(|e| e.borrow().index()).collect()
    }
}

impl MoveAction {
    fn new_if_valid(x: f32, y: f32, dist: Option<f32>) -> Option<MoveAction> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();

        let pc = match GameState::selected().first() {
            None => return None,
            Some(pc) => Rc::clone(pc),
        };

        let dist = match dist {
            None => MOVE_TO_THRESHOLD,
            Some(dist) => dist,
        };

        if let Some(target) = get_attack_target(&area_state, x as i32, y as i32) {
            if pc.borrow().can_attack(&target, &area_state) {
                // if we can already reach the target with our weapon, don't
                // move further towards them
                return None;
            }
        }

        let selected = GameState::selected();

        if !GameState::can_move_towards_point(&selected[0], entities_to_ignore(), x, y, dist) {
            return None;
        }

        Some(MoveAction { selected, x, y, dist, cb: None })
    }

    fn create_if_valid(x: f32, y: f32, dist: Option<f32>) -> Option<Box<ActionKind>> {
        match MoveAction::new_if_valid(x, y, dist) {
            None => None,
            Some(action) => Some(Box::new(action)),
        }
    }

    fn move_one(&mut self) {
        let cb = self.cb.take();
        GameState::move_towards_point(&self.selected[0], entities_to_ignore(),
            self.x, self.y, self.dist, cb);
    }

    fn move_all(&mut self) {
        let formation = GameState::party_formation();
        formation.borrow().move_group(&self.selected, entities_to_ignore(),
                                      self.x, self.y);
    }
}

impl ActionKind for MoveAction {
    fn cursor_state(&self) -> animation_state::Kind {
        animation_state::Kind::MouseMove
    }

    fn fire_action(&mut self, widget: &Rc<RefCell<Widget>>) {
        trace!("Firing move action");
        let root = Widget::get_root(widget);
        let view = Widget::downcast_kind_mut::<RootView>(&root);
        view.set_prop_window(&root, false, 0);
        view.set_map_window(&root, false, false);

        if self.cb.is_some() || GameState::is_combat_active() {
            self.move_one();
        } else {
            self.move_all();
        }

    }

    fn get_hover_info(&self) -> Option<(Rc<ObjectSize>, i32, i32)> {
        let size = Rc::clone(&self.selected[0].borrow().size);
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
