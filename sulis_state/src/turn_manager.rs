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
use std::collections::{HashSet, VecDeque};

use sulis_module::Faction;
use {ChangeListenerList, Effect, EntityState, GameState};

pub const ROUND_TIME_MILLIS: u32 = 5000;

#[derive(Clone, Copy)]
enum Entry {
    Entity(usize),
    Effect(usize),
}

pub struct TurnManager {
    entities: Vec<Option<Rc<RefCell<EntityState>>>>,
    effects: Vec<Option<Effect>>,
    combat_active: bool,
    last_millis: u32,

    listeners: ChangeListenerList<TurnManager>,
    order: VecDeque<Entry>,
}

impl Default for TurnManager {
    fn default() -> TurnManager {
        TurnManager {
            entities: Vec::new(),
            effects: Vec::new(),
            listeners: ChangeListenerList::default(),
            order: VecDeque::new(),
            combat_active: false,
            last_millis: 0,
        }
    }
}

impl TurnManager {
    fn update(&mut self, current_millis: u32) {
        self.last_millis = current_millis;
        let elapsed_millis = if self.combat_active { current_millis - self.last_millis } else { 0 };

        if !self.combat_active {
            // removal just replaces some with none, so we can safely iterate
            for index in 0..self.effects.len() {
                if self.update_effect(index, elapsed_millis) {
                    self.remove_effect(index);
                }
            }
        }

        for index in 0..self.entities.len() {
            if self.update_entity(index, elapsed_millis) {
                self.remove_entity(index);
                // TODO area needs to learn of this
            }
        }
    }

    fn update_effect(&mut self, index: usize, elapsed_millis: u32) -> bool {
        let effect = match self.effects[index] {
            None => return false,
            Some(ref mut effect) => effect,
        };

        effect.update(elapsed_millis);
        effect.is_removal()
    }

    fn update_entity(&mut self, index: usize, elapsed_millis: u32) -> bool {
        let entity = match self.entities[index].as_ref() {
            None => return false,
            Some(entity) => entity,
        };

        let mut entity = entity.borrow_mut();
        entity.actor.elapse_time(elapsed_millis, &self.effects);
        entity.is_marked_for_removal()
    }

    fn next(&mut self) {
        self.iterate_to_next_entity();
        self.init_turn_for_current_entity();

        match self.current() {
            Some(entity) => {
                if entity.borrow().is_party_member() {
                    GameState::set_selected_party_member(entity);
                } else {
                    GameState::clear_selected_party_member();
                }
            }, None => unreachable!(),
        }

        self.listeners.notify(&self);
    }

    fn init_turn_for_current_entity(&mut self) {
        let current = match self.order.front() {
            Some(Entry::Entity(index)) => {
                match self.entities[*index] {
                    None => unreachable!(),
                    Some(ref entity) => entity,
                }
            },
            _ => unreachable!(),
        };

        let current = current.borrow_mut();
        current.actor.init_turn();
        current.actor.elapse_time(ROUND_TIME_MILLIS, &self.effects);

        debug!("'{}' now has the active turn", current.actor.actor.name);
    }

    fn current(&self) -> Option<Rc<RefCell<EntityState>>> {
        if !self.combat_active { return None; }

        match self.order.front() {
            Some(Entry::Entity(index)) => {
                match self.entities[*index] {
                    None => unreachable!(),
                    Some(ref entity) => Some(Rc::clone(entity)),
                }
            },
            _ => None,
        }
    }

    fn iterate_to_next_entity(&mut self) {
        let mut current_ended = false;

        loop {
            if current_ended && self.current_is_active_entity() { break; }

            let front = match self.order.pop_front() {
                None => return,
                Some(entry) => entry,
            };

            match front {
                Entry::Effect(index) => {
                    if self.update_effect(index, ROUND_TIME_MILLIS) {
                        self.remove_effect(index);
                    } else {
                        self.order.push_back(Entry::Effect(index));
                    }
                },
                Entry::Entity(index) => {
                    if let Some(entity) = &self.entities[index] {
                        entity.borrow_mut().actor.end_turn();
                    }

                    self.order.push_back(Entry::Entity(index));
                    current_ended = true;
                }
            }
        }
    }

    fn current_is_active_entity(&self) -> bool {
        if let Some(Entry::Entity(index)) = self.order.front() {
            if let Some(entity) = &self.entities[*index] {
                let entity = entity.borrow();
                return entity.is_party_member() || entity.is_ai_active();
            }
        }

        false
    }

    fn check_ai_activation(&mut self, mover: &Rc<RefCell<EntityState>>) {
        let area_state = GameState::area_state();
        let mut area_state = area_state.borrow_mut();

        let mut groups_to_activate: HashSet<usize> = HashSet::new();
        let mut state_changed = false;

        for entity in self.entities.iter() {
            let entity = match entity {
                None => continue,
                Some(ref entity) => entity,
            };

            if Rc::ptr_eq(mover, entity) { continue; }

            let mut entity = entity.borrow_mut();
            if !entity.is_hostile(mover) { continue; }
            if !entity.location.is_in(&area_state) { continue; }

            let mover = mover.borrow();
            if !area_state.has_visibility(&mover, &entity) && !area_state.has_visibility(&entity, &mover) {
                continue;
            }

            self.activate_entity_ai(&mut entity, &mut groups_to_activate);
            state_changed = true;
        }

        if !state_changed { return; }

        self.activate_entity_ai(&mut mover.borrow_mut(), &mut groups_to_activate);

        for entity in self.entities.iter() {
            let entity = match entity {
                None => continue,
                Some(ref entity) => entity,
            };

            let mut entity = entity.borrow_mut();
            if entity.is_ai_active() { continue; }

            match entity.ai_group() {
                None => continue,
                Some(group) => {
                    if groups_to_activate.contains(&group) {
                        entity.set_ai_active(true);
                    }
                }
            }
        }

        if !self.combat_active {
            info!("Set combat mode active");
            self.set_combat_active(true);
            loop {
                if self.current_is_active_entity() { break; }
                let front = self.order.pop_front().unwrap();
                self.order.push_back(front);
            }
        } else {
            self.listeners.notify(&self);
        }

    }

    fn activate_entity_ai(&self, entity: &mut EntityState, groups: &mut HashSet<usize>) {
        if entity.is_party_member() { return; }
        if entity.is_ai_active() { return; }

        trace!("Activate AI for {}", entity.actor.actor.name);
        entity.set_ai_active(true);

        if let Some(group) = entity.ai_group() {
            groups.insert(group);
        }
    }

    fn is_combat_active(&self) -> bool {
        self.combat_active
    }

    fn set_combat_active(&mut self, active: bool) {
        if active == self.combat_active { return; }

        info!("Setting combat mode active = {}", active);
        self.combat_active = active;

        if !active {
            self.end_combat();
        } else {
            self.initiate_combat();
        }

        self.listeners.notify(&self);
    }

    fn end_combat(&mut self) {
        for entity in self.entities.iter() {
            let entity = match entity {
                None => continue,
                Some(ref entity) => entity,
            };
            let entity = entity.borrow_mut();

            entity.set_ai_active(false);

            if !entity.is_party_member() { continue; }

            entity.actor.init_turn();

            // TODO this is healing the party at the end of each combat
            entity.actor.init();
        }
    }

    fn initiate_combat(&mut self) {
        let mut entries: Vec<_> = self.order.drain(..).collect();
        entries.sort_by_key(|entry| {
            match entry {
                Entry::Entity(index) => {
                    self.entities[*index].as_ref().unwrap().borrow().actor.stats.initiative
                }, Entry::Effect(_) => {
                    // TODO effects should be preferrably moved along with the
                    // entity who is next in the queue to preserve this ordering
                    0
                }
            }
        });
        entries.drain(..).for_each(|entry| self.order.push_front(entry));

        for entity in self.entities.iter() {
            let entity = match entity {
                None => continue,
                Some(ref entity) => entity,
            };

            entity.borrow_mut().actor.end_turn();
            entity.borrow_mut().actor.set_overflow_ap(0);
        }
        GameState::set_clear_anims();
    }

    fn add_entity(&mut self, entity: &Rc<RefCell<EntityState>>) {
        let entity_to_add = Rc::clone(entity);
        self.entities.push(Some(entity_to_add));
        let index = self.entities.len() - 1;
        self.order.push_back(Entry::Entity(index));
        debug!("Added entity at {} to turn timer", index);

        self.check_ai_activation(&entity);
        self.listeners.notify(&self);
    }

    fn add_effect(&mut self, effect: Effect) {
        self.effects.push(Some(effect));
        let index = self.effects.len() - 1;
        self.order.push_back(Entry::Effect(index));
        debug!("Added effect at {} to turn timer", index);
    }

    fn remove_effect(&mut self, index: usize) {
        self.effects[index] = None;
        self.order.retain(|e| {
            match e {
                Entry::Effect(i) => *i != index,
                Entry::Entity(_) => true,
            }
        });
    }

    fn remove_entity(&mut self, index: usize) {
        self.entities[index] = None;

        self.order.retain(|e| {
            match e {
                Entry::Entity(i) => *i != index,
                Entry::Effect(_) => true,
            }
        });

        if self.order.iter().all(|e| {
            match e {
                Entry::Effect(_) => true,
                Entry::Entity(index) => {
                    let entity = self.entities[*index].as_ref().unwrap().borrow();
                    !entity.is_ai_active() || entity.actor.actor.faction == Faction::Friendly
                }
            }
        }) {
            self.set_combat_active(false);
        } else {
            self.listeners.notify(&self);
        }
    }
}
