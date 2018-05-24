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
use std::collections::HashSet;
use std::collections::VecDeque;
pub use std::collections::vec_deque::Iter;

use sulis_module::{Area, Faction};

use {AreaState, ChangeListenerList, EntityState, GameState};

pub const ROUND_TIME_MILLIS: u32 = 5000;

/// `TurnTimer` maintains a list of all entities in a given `AreaState`.  The
/// list proceed in initiative order, with the front of the list always containing
/// the currently active entity.  Once an entity's turn is up, it is moved to the
/// back of the list.  Internally, this is accomplished using a `VecDeque`
pub struct TurnTimer {
    entities: VecDeque<Rc<RefCell<EntityState>>>,
    pub listeners: ChangeListenerList<TurnTimer>,
    active: bool,
}

impl Default for TurnTimer {
    fn default() -> TurnTimer {
        TurnTimer {
            entities: VecDeque::new(),
            listeners: ChangeListenerList::default(),
            active: false,
        }
    }
}

impl TurnTimer {
    pub fn new(area_state: &AreaState) -> TurnTimer {
        let entities: VecDeque<Rc<RefCell<EntityState>>> = area_state.entity_iter().collect();

        if let Some(entity) = entities.front() {
            debug!("Starting turn for '{}'", entity.borrow().actor.actor.name);
            entity.borrow_mut().actor.init_turn();
        }

        debug!("Got {} entities for turn timer", entities.len());
        TurnTimer {
            entities,
            ..Default::default()
        }
    }

    fn activate_entity(&self, entity: &Rc<RefCell<EntityState>>, groups: &mut HashSet<usize>) -> bool {
        if entity.borrow().is_party_member() { return false; }
        if entity.borrow().is_ai_active() { return false; }

        trace!("activate ai for {}", entity.borrow().actor.actor.name);
        entity.borrow_mut().set_ai_active(true);

        if let Some(ai_group) = entity.borrow().ai_group() {
            groups.insert(ai_group);
        }

        true
    }

    pub fn check_ai_activation(&mut self, mover: &Rc<RefCell<EntityState>>, area: &Rc<Area>) {
        let mut groups_to_activate: HashSet<usize> = HashSet::new();
        let mut updated = false;

        for entity in self.entities.iter() {
            if Rc::ptr_eq(mover, entity) { continue; }
            if !entity.borrow().is_hostile(mover) { continue; }

            if !mover.borrow().has_visibility(entity, area) && !entity.borrow().has_visibility(mover, area) {
                continue;
            }

            self.activate_entity(entity, &mut groups_to_activate);
            updated = true;
        }

        if updated {
            self.activate_entity(mover, &mut groups_to_activate);
        }

        for entity in self.entities.iter() {
            if entity.borrow().is_ai_active() { continue; }

            let ai_group = match entity.borrow().ai_group() {
                None => continue,
                Some(ai_group) => ai_group,
            };

            if groups_to_activate.contains(&ai_group) {
                entity.borrow_mut().set_ai_active(true);
            }
        }

        if updated {
            if !self.active {
                self.set_active(true);
                self.activate_current();
            } else {
                self.listeners.notify(&self);
            }
        }
    }

    pub fn roll_initiative(&mut self) {
        let mut entities: Vec<(i32, Rc<RefCell<EntityState>>)> = Vec::new();
        for ref entity in self.entities.iter() {
            let initiative = entity.borrow().actor.stats.initiative;
            entities.push((initiative, Rc::clone(entity)));
        }
        entities.sort_by_key(|e| e.0);

        self.entities.clear();
        for (_initiative, entity) in entities {
            self.entities.push_front(entity);
        }

        self.entities.iter().for_each(|e| {
            e.borrow_mut().actor.end_turn();
            e.borrow_mut().actor.set_overflow_ap(0);
        });
        GameState::set_entering_combat();
    }

    pub fn end_combat(&mut self) {
        for entity in self.entities.iter() {
            entity.borrow_mut().set_ai_active(false);

            if !entity.borrow().is_party_member() { continue; }

            entity.borrow_mut().actor.init_turn();

            // TODO don't just heal at the end of combat
            entity.borrow_mut().actor.init();
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        if active != self.active {
            debug!("Set turn timer active = {}", active);
            self.active = active;

            if !active {
                self.end_combat();
            } else {
                self.roll_initiative();
            }
        }
        self.listeners.notify(&self);
    }

    pub fn add(&mut self, entity: &Rc<RefCell<EntityState>>, area: &Rc<Area>) {
        debug!("Added entity to turn timer: '{}'", entity.borrow().actor.actor.name);
        self.entities.push_back(Rc::clone(entity));
        if self.entities.len() == 1 {
            // we just pushed the only entity
            entity.borrow_mut().actor.init_turn();
        }
        self.check_ai_activation(entity, area);
        self.listeners.notify(&self);
    }

    pub fn remove(&mut self, entity: &Rc<RefCell<EntityState>>) {
        trace!("Removing entity from turn timer: '{}'", entity.borrow().actor.actor.name);
        self.entities.retain(|other| *entity.borrow() != *other.borrow());

        if self.entities.iter().all(|e| !e.borrow().is_ai_active() ||
                                    e.borrow().actor.actor.faction == Faction::Friendly) {
            self.set_active(false);
        } else {
            self.listeners.notify(&self);
        }
    }

    pub fn current(&self) -> Option<&Rc<RefCell<EntityState>>> {
        if !self.active { return None; }

        self.entities.front()
    }

    pub fn next(&mut self) {
        if !self.active || self.entities.front().is_none() { return; }

        let front = self.entities.pop_front().unwrap();
        front.borrow_mut().actor.end_turn();
        self.entities.push_back(front);

        self.activate_current();
        self.listeners.notify(&self);
    }

    fn activate_current(&mut self) {
        loop {
            {
                let front = self.entities.front().unwrap();
                if front.borrow().is_party_member() || front.borrow().is_ai_active() {
                    break;
                }
            }

            let front = self.entities.pop_front().unwrap();
            self.entities.push_back(front);
        }

        if let Some(current) = self.entities.front() {
            current.borrow_mut().actor.init_turn();
            current.borrow_mut().actor.update(ROUND_TIME_MILLIS);
            if current.borrow().is_party_member() {
                GameState::set_selected_party_member(Rc::clone(current));
            } else {
                GameState::clear_selected_party_member();
            }
            debug!("'{}' now has the active turn.", current.borrow().actor.actor.name);
        }
    }

    pub fn active_iter(&self) -> ActiveEntityIterator {
        ActiveEntityIterator {
            entity_iter: self.entities.iter(),
            turn_timer: self,
        }
    }
}

pub struct ActiveEntityIterator<'a> {
    entity_iter: Iter<'a, Rc<RefCell<EntityState>>>,
    turn_timer: &'a TurnTimer,
}

impl<'a> Iterator for ActiveEntityIterator<'a> {
    type Item = &'a Rc<RefCell<EntityState>>;
    fn next(&mut self) -> Option<&'a Rc<RefCell<EntityState>>> {
        if !self.turn_timer.active { return None; }

        loop {
            match self.entity_iter.next() {
                None => return None,
                Some(entity) => {
                    if entity.borrow().is_party_member() || entity.borrow().is_ai_active() {
                        return Some(entity);
                    }
                }
            }
        }
    }
}
