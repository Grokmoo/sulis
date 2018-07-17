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
use std::collections::{HashSet, VecDeque, vec_deque::Iter};

use sulis_module::Faction;
use {AreaState, ChangeListenerList, Effect, EntityState, GameState, ScriptCallback};

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

    pub listeners: ChangeListenerList<TurnManager>,
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
    pub(crate) fn clear(&mut self) {
        self.entities.clear();
        self.effects.clear();
        self.combat_active = false;
        self.listeners = ChangeListenerList::default();
        self.order.clear();
    }

    pub fn effect_mut(&mut self, index: usize) -> &mut Effect {
        self.effects[index].as_mut().unwrap()
    }

    pub fn effect(&self, index: usize) -> &Effect {
        self.effects[index].as_ref().unwrap()
    }

    pub fn active_iter(&self) -> ActiveEntityIterator {
        ActiveEntityIterator { mgr: &self, entry_iter: self.order.iter() }
    }

    pub fn entity_iter(&self) -> EntityIterator {
        EntityIterator { mgr: &self, index: 0 }
    }

    pub fn has_entity(&self, index: usize) -> bool {
        if index >= self.entities.len() { return false; }

        self.entities[index].is_some()
    }

    pub fn entity_checked(&self, index: usize) -> Option<Rc<RefCell<EntityState>>> {
        if index >= self.entities.len() { return None; }
        self.entities[index].clone()
    }

    pub fn entity(&self, index: usize) -> Rc<RefCell<EntityState>> {
        Rc::clone(self.entities[index].as_ref().unwrap())
    }

    #[must_use]
    pub fn update(&mut self, current_millis: u32) -> Vec<Rc<ScriptCallback>> {
        let mut cbs = Vec::new();

        let elapsed_millis = if !self.combat_active { current_millis - self.last_millis } else { 0 };
        self.last_millis = current_millis;

        // removal just replaces some with none, so we can safely iterate
        for index in 0..self.effects.len() {
            let (is_removal, mut effect_cbs) = self.update_effect(index, elapsed_millis);
            cbs.append(&mut effect_cbs);
            if is_removal {
                self.remove_effect(index);
            }
        }

        for index in 0..self.entities.len() {
            if self.update_entity(index, elapsed_millis) {
                self.remove_entity(index);
            }
        }

        cbs
    }

    #[must_use]
    fn update_effect(&mut self, index: usize, elapsed_millis: u32) -> (bool, Vec<Rc<ScriptCallback>>) {
        let effect = match self.effects[index] {
            None => return (false, Vec::new()),
            Some(ref mut effect) => effect,
        };

        let cbs = effect.update(elapsed_millis);
        (effect.is_removal(), cbs)
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

    #[must_use]
    pub fn next(&mut self) -> Vec<Rc<ScriptCallback>> {
        let cbs = self.iterate_to_next_entity();
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
        cbs
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

        if current.borrow().is_party_member() {
            GameState::set_selected_party_member(Rc::clone(current));
        }

        let mut current = current.borrow_mut();
        current.actor.init_turn();
        current.actor.elapse_time(ROUND_TIME_MILLIS, &self.effects);

        debug!("'{}' now has the active turn", current.actor.actor.name);
    }

    pub fn current(&self) -> Option<Rc<RefCell<EntityState>>> {
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

    #[must_use]
    fn iterate_to_next_entity(&mut self) -> Vec<Rc<ScriptCallback>> {
        let mut cbs = Vec::new();
        let mut current_ended = false;

        loop {
            if current_ended && self.current_is_active_entity() { break; }

            let front = match self.order.pop_front() {
                None => unreachable!(),
                Some(entry) => entry,
            };

            match front {
                Entry::Effect(index) => {
                    let (removal, mut effect_cbs) = self.update_effect(index, ROUND_TIME_MILLIS);
                    cbs.append(&mut effect_cbs);
                    if removal { self.remove_effect(index); }
                    else { self.order.push_back(Entry::Effect(index)); }
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

        cbs
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

    pub fn check_ai_activation_for_party(&mut self, area_state: &AreaState) {
        for entity in GameState::party() {
            self.check_ai_activation(&entity, area_state);
        }
    }

    pub fn check_ai_activation(&mut self, mover: &Rc<RefCell<EntityState>>, area_state: &AreaState) {
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
            self.set_combat_active(true);
            loop {
                if self.current_is_active_entity() { break; }
                let front = self.order.pop_front().unwrap();
                self.order.push_back(front);
            }
            self.init_turn_for_current_entity();
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

    pub fn is_combat_active(&self) -> bool {
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
            let mut entity = entity.borrow_mut();

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

    pub fn add_entity(&mut self, entity: &Rc<RefCell<EntityState>>) -> usize {
        let entity_to_add = Rc::clone(entity);
        self.entities.push(Some(entity_to_add));
        let index = self.entities.len() - 1;
        self.order.push_back(Entry::Entity(index));
        debug!("Added entity at {} to turn timer", index);

        entity.borrow_mut().index = index;
        self.listeners.notify(&self);

        index
    }

    pub fn add_effect(&mut self, effect: Effect, entity: &Rc<RefCell<EntityState>>) -> usize {
        let bonuses = effect.bonuses().clone();

        self.effects.push(Some(effect));
        let index = self.effects.len() - 1;
        self.order.push_back(Entry::Effect(index));
        debug!("Added effect at {} to turn timer", index);

        entity.borrow_mut().actor.add_effect(index, bonuses);

        index
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
        let entity = Rc::clone(self.entities[index].as_ref().unwrap());
        let area_state = GameState::get_area_state(&entity.borrow().location.area_id).unwrap();
        area_state.borrow_mut().remove_entity(&entity);

        self.entities[index] = None;

        // can't do this with a collect because of lifetime issues
        let mut effects_to_remove = Vec::new();
        {
            let entity = entity.borrow();
            for index in entity.actor.effects_iter() {
                effects_to_remove.push(*index);
                self.effects[*index] = None;
            }
        }

        self.order.retain(|e| {
            match e {
                Entry::Entity(i) => *i != index,
                Entry::Effect(i) => !effects_to_remove.contains(i),
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

pub struct ActiveEntityIterator<'a> {
    entry_iter: Iter<'a, Entry>,
    mgr: &'a TurnManager,
}

impl<'a> Iterator for ActiveEntityIterator<'a> {
    type Item = &'a Rc<RefCell<EntityState>>;
    fn next(&mut self) -> Option<&'a Rc<RefCell<EntityState>>> {
        if !self.mgr.is_combat_active() { return None; }

        loop {
            match self.entry_iter.next() {
                None => return None,
                Some(ref entry) => match entry {
                    Entry::Effect(_) => (),
                    Entry::Entity(index) => {
                        let entity = self.mgr.entities[*index].as_ref().unwrap();
                        if entity.borrow().is_party_member() || entity.borrow().is_ai_active() {
                            return Some(entity);
                        }
                    }
                }
            }
        }
    }
}
pub struct EntityIterator<'a> {
    mgr: &'a TurnManager,
    index: usize,
}

impl<'a> Iterator for EntityIterator<'a> {
    type Item = Rc<RefCell<EntityState>>;
    fn next(&mut self) -> Option<Rc<RefCell<EntityState>>> {
        loop {
            let next = self.mgr.entities.get(self.index);

            self.index += 1;

            match next {
                None => return None,
                Some(e) => match e {
                    &None => continue,
                    &Some(ref entity) => return Some(Rc::clone(entity))
                }
            }
        }
    }
}
