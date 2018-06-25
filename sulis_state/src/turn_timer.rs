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

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;
use std::collections::VecDeque;
pub use std::collections::vec_deque::Iter;

use sulis_module::{Faction};

use {ActorState, AreaState, ChangeListenerList, EntityState, GameState};

pub const ROUND_TIME_MILLIS: u32 = 5000;

#[derive(Clone)]
enum Entry {
    Entity(Rc<RefCell<EntityState>>),
    Effect(usize),
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Entry::Entity(e) => write!(f, "Entry::Entity::{}", e.borrow().index),
            Entry::Effect(i) => write!(f, "Entry::Effect::{}", i),
        }
    }
}

/// `TurnTimer` maintains a list of all entities in a given `AreaState`.  The
/// list proceed in initiative order, with the front of the list always containing
/// the currently active entity.  Once an entity's turn is up, it is moved to the
/// back of the list.  Internally, this is accomplished using a `VecDeque`
pub struct TurnTimer {
    entries: VecDeque<Entry>,
    pub listeners: ChangeListenerList<TurnTimer>,
    active: bool,
}

impl Default for TurnTimer {
    fn default() -> TurnTimer {
        TurnTimer {
            entries: VecDeque::new(),
            listeners: ChangeListenerList::default(),
            active: false,
        }
    }
}

impl TurnTimer {
    pub fn new(area_state: &AreaState) -> TurnTimer {
        let entries: VecDeque<Entry> = area_state.entity_iter().map(|e| Entry::Entity(e)).collect();

        if let Some(entry) = entries.front() {
            if let Entry::Entity(ref entity) = entry {
                debug!("Starting turn for '{}'", entity.borrow().actor.actor.name);
                ActorState::init_actor_turn(entity);
            }
        }

        debug!("Got {} entities for turn timer", entries.len());
        TurnTimer {
            entries,
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

    pub fn check_ai_activation(&mut self, mover: &Rc<RefCell<EntityState>>, area_state: &mut AreaState) {
        let mut groups_to_activate: HashSet<usize> = HashSet::new();
        let mut updated = false;

        for entry in self.entries.iter() {
            let entity = match entry {
                Entry::Entity(ref e) => e,
                Entry::Effect(_) => continue,
            };

            if Rc::ptr_eq(mover, entity) { continue; }
            if !entity.borrow().is_hostile(mover) { continue; }

            if !area_state.has_visibility(&mover.borrow(), &entity.borrow()) &&
                !area_state.has_visibility(&entity.borrow(), &mover.borrow()) { continue; }

            self.activate_entity(entity, &mut groups_to_activate);
            updated = true;
        }

        if updated {
            self.activate_entity(mover, &mut groups_to_activate);
        }

        for entry in self.entries.iter() {
            let entity = match entry {
                Entry::Entity(ref e) => e,
                Entry::Effect(_) => continue,
            };

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
                info!("Set combat mode active");
                self.set_active(true);
                loop {
                    if self.current_is_active_entity() { break; }
                    let front = self.entries.pop_front().unwrap();
                    self.entries.push_back(front);
                }
                self.activate_current(area_state);
            } else {
                self.listeners.notify(&self);
            }
        }
    }

    pub fn roll_initiative(&mut self) {
        let mut entries: Vec<_> = self.entries.iter().map(|e| e.clone()).collect();

        entries.sort_by_key(|entry| {
            match entry {
                Entry::Entity(ref e) => {
                    e.borrow().actor.stats.initiative
                }, Entry::Effect(_) => {
                    // TODO effects may need to be associated with the
                    // nearest entity here for consistency when entering combat
                    0
                }
            }
        });

        self.entries.clear();
        for entry in entries {
            self.entries.push_front(entry);
        }

        for entry in self.entries.iter() {
            let entity = match entry {
                Entry::Entity(ref e) => e,
                Entry::Effect(_) => continue,
            };

            entity.borrow_mut().actor.end_turn();
            entity.borrow_mut().actor.set_overflow_ap(0);
        }

        GameState::set_clear_anims();
    }

    pub fn end_combat(&mut self) {
        for entry in self.entries.iter() {
            let entity = match entry {
                Entry::Entity(ref e) => e,
                Entry::Effect(_) => continue,
            };

            entity.borrow_mut().set_ai_active(false);

            if !entity.borrow().is_party_member() { continue; }

            ActorState::init_actor_turn(entity);

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

    pub fn remove_effect(&mut self, index: usize) {
        debug!("Remove effect with index {} from turn timer", index);
        self.entries.retain(|entry| {
            match entry {
                Entry::Effect(i) => *i != index,
                Entry::Entity(_) => true,
            }
        });
    }

    pub fn add_effect(&mut self, index: usize) {
        self.entries.push_back(Entry::Effect(index));
        debug!("Added effect with index {} to turn timer", index);
    }

    pub fn add(&mut self, entity: &Rc<RefCell<EntityState>>, area_state: &mut AreaState) {
        debug!("Added entity to turn timer: '{}'", entity.borrow().actor.actor.name);
        self.entries.push_back(Entry::Entity(Rc::clone(entity)));
        if self.entries.len() == 1 {
            // we just pushed the only entity
            ActorState::init_actor_turn(entity);
        }
        self.check_ai_activation(entity, area_state);
        self.listeners.notify(&self);
    }

    pub fn remove_entity(&mut self, entity: &Rc<RefCell<EntityState>>) {
        // TODO if entity being removed is current entity and next entity is player
        // this leads to a turn timer lock

        trace!("Removing entity from turn timer: '{}'", entity.borrow().actor.actor.name);
        self.entries.retain(|entry| {
            match entry {
                Entry::Entity(ref e) => !Rc::ptr_eq(e, entity),
                Entry::Effect(_) => true,
            }
        });

        if self.entries.iter().all(|entry| {
            match entry {
                Entry::Entity(ref e) => {
                    !e.borrow().is_ai_active() || e.borrow().actor.actor.faction == Faction::Friendly
                }, Entry::Effect(_) => true,
            }
        }) {
            self.set_active(false);
        } else {
            self.listeners.notify(&self);
        }
    }

    pub fn current(&self) -> Option<Rc<RefCell<EntityState>>> {
        if !self.active { return None; }

        match self.entries.front() {
            None => None,
            Some(ref entry) => {
                match entry {
                    Entry::Entity(ref e) => Some(Rc::clone(e)),
                    Entry::Effect(_) => None,
                }
            }
        }
    }

    fn current_is_active_entity(&self) -> bool {
        match self.entries.front() {
            Some(Entry::Entity(e)) => {
                if e.borrow().is_party_member() || e.borrow().is_ai_active() {
                    return true;
                }
            }, _ => (),
        }

        false
    }

    pub fn next(&mut self) {
        if !self.active { return; }

        let mut cbs_to_fire = Vec::new();

        {
            let area_state = GameState::area_state();
            let mut area_state = area_state.borrow_mut();
            let mut current_ended = false;
            loop {

                if current_ended && self.current_is_active_entity() { break; }

                let front = match self.entries.pop_front() {
                    None => return,
                    Some(entry) => entry,
                };

                match front {
                    Entry::Effect(index) => {
                        let effect = area_state.effect_mut(index);
                        if effect.update(ROUND_TIME_MILLIS) {
                            cbs_to_fire.append(&mut effect.callbacks());
                        }
                        self.entries.push_back(Entry::Effect(index));
                    },
                    Entry::Entity(entity) => {
                        entity.borrow_mut().actor.end_turn();
                        self.entries.push_back(Entry::Entity(entity));
                        current_ended = true;
                    }
                }
            }
        }

        cbs_to_fire.iter().for_each(|cb| cb.on_round_elapsed());

        let area_state = GameState::area_state();
        self.activate_current(&mut area_state.borrow_mut());
        self.listeners.notify(&self);
    }

    fn activate_current(&mut self, area_state: &mut AreaState) {
        let current = match self.entries.front() {
            None => return,
            Some(ref entry) => match entry {
                Entry::Effect(_) => return,
                Entry::Entity(ref entity) => Rc::clone(entity),
            }
        };

        ActorState::init_actor_turn(&current);
        ActorState::update(&current, &mut area_state.effects,
                           self, ROUND_TIME_MILLIS);

        if current.borrow().is_party_member() {
            GameState::set_selected_party_member(Rc::clone(&current));
        } else {
            GameState::clear_selected_party_member();
        }
        debug!("'{}' now has the active turn.", current.borrow().actor.actor.name);
    }

    pub fn active_iter(&self) -> ActiveEntityIterator {
        ActiveEntityIterator {
            entry_iter: self.entries.iter(),
            turn_timer: self,
        }
    }
}

pub struct ActiveEntityIterator<'a> {
    entry_iter: Iter<'a, Entry>,
    turn_timer: &'a TurnTimer,
}

impl<'a> Iterator for ActiveEntityIterator<'a> {
    type Item = &'a Rc<RefCell<EntityState>>;
    fn next(&mut self) -> Option<&'a Rc<RefCell<EntityState>>> {
        if !self.turn_timer.active { return None; }

        loop {
            match self.entry_iter.next() {
                None => return None,
                Some(ref entry) => match entry {
                    Entry::Effect(_) => (),
                    Entry::Entity(ref e) => {
                        if e.borrow().is_party_member() || e.borrow().is_ai_active() {
                            return Some(e);
                        }
                    }
                }
            }
        }
    }
}
