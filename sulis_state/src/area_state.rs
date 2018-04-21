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

use rand::{self, Rng};
use sulis_core::ui::Color;
use sulis_module::{Actor, Area, LootList, Module, ObjectSize};
use sulis_module::area::{EncounterData, PropData, Transition};
use sulis_core::util::Point;

use {AreaFeedbackText, calculate_los, ChangeListenerList, EntityState, Location, Merchant, PropState, Targeter, TurnTimer};

use std::slice::Iter;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub struct AreaState {
    pub area: Rc<Area>,
    pub listeners: ChangeListenerList<AreaState>,
    pub turn_timer: TurnTimer,
    entities: Vec<Option<Rc<RefCell<EntityState>>>>,
    props: Vec<Option<PropState>>,

    prop_grid: Vec<Option<usize>>,
    entity_grid: Vec<Option<usize>>,
    transition_grid: Vec<Option<usize>>,

    pub pc_vis_delta: (i32, i32),
    pc_vis: Vec<bool>,
    pc_explored: Vec<bool>,

    feedback_text: Vec<AreaFeedbackText>,
    scroll_to_callback: Option<Rc<RefCell<EntityState>>>,

    last_time_millis: u32,

    targeter: Option<Rc<RefCell<Box<Targeter>>>>,

    merchants: Vec<Merchant>,

    pub on_load_fired: bool,
}

impl PartialEq for AreaState {
    fn eq(&self, other: &AreaState) -> bool {
        self.area == other.area
    }
}

impl AreaState {
    pub fn get_merchant(&self, id: &str) -> Option<&Merchant> {
        let mut index = None;
        for (i, merchant) in self.merchants.iter().enumerate() {
            if merchant.id == id {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(i) => Some(&self.merchants[i]),
            None => None,
        }
    }

    pub fn get_merchant_mut(&mut self, id: &str) -> Option<&mut Merchant> {
        let mut index = None;
        for (i, merchant) in self.merchants.iter().enumerate() {
            if merchant.id == id {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(i) => Some(&mut self.merchants[i]),
            None => None,
        }
    }

    pub fn get_or_create_merchant(&mut self, id: &str, loot_list: &Rc<LootList>,
                                  buy_frac: f32, sell_frac: f32) -> &mut Merchant {
        let mut index = None;
        for (i, merchant) in self.merchants.iter().enumerate() {
            if merchant.id == id {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(i) => &mut self.merchants[i],
            None => {
                info!("Creating merchant '{}'", id);
                let len = self.merchants.len();
                let merchant = Merchant::new(id, loot_list, buy_frac, sell_frac);
                self.merchants.push(merchant);
                &mut self.merchants[len]
            }
        }
    }

    pub fn targeter(&mut self) -> Option<Rc<RefCell<Box<Targeter>>>> {
        match self.targeter {
            None => None,
            Some(ref targeter) => Some(Rc::clone(targeter)),
        }
    }

    pub (crate) fn set_targeter(&mut self, targeter: Box<Targeter>) {
        self.targeter = Some(Rc::new(RefCell::new(targeter)));
    }

    pub fn push_scroll_to_callback(&mut self, entity: Rc<RefCell<EntityState>>) {
        self.scroll_to_callback = Some(entity);
    }

    pub fn pop_scroll_to_callback(&mut self) -> Option<Rc<RefCell<EntityState>>> {
        self.scroll_to_callback.take()
    }

    pub fn new(area: Rc<Area>) -> AreaState {
        let dim = (area.width * area.height) as usize;
        let entity_grid = vec![None;dim];
        let transition_grid = vec![None;dim];
        let prop_grid = vec![None;dim];
        let pc_vis = vec![false;dim];
        let pc_explored = vec![false;dim];

        info!("Initializing area state for '{}'", area.name);
        AreaState {
            area,
            entities: Vec::new(),
            props: Vec::new(),
            turn_timer: TurnTimer::default(),
            transition_grid,
            entity_grid,
            prop_grid,
            listeners: ChangeListenerList::default(),
            pc_vis,
            pc_explored,
            pc_vis_delta: (0, 0),
            feedback_text: Vec::new(),
            scroll_to_callback: None,
            last_time_millis: 0,
            targeter: None,
            merchants: Vec::new(),
            on_load_fired: false,
        }
    }

    /// Adds entities defined in the area definition to this area state
    pub fn populate(&mut self) {
        let area = Rc::clone(&self.area);
        for actor_data in area.actors.iter() {
            let actor = match Module::actor(&actor_data.id) {
                None => {
                    warn!("No actor with id '{}' found when initializing area '{}'",
                              actor_data.id, self.area.id);
                    continue;
                },
                Some(actor_data) => actor_data,
            };

            let location = Location::from_point(&actor_data.location, &self.area);
            debug!("Adding actor '{}' at '{:?}'", actor.id, location);
            self.add_actor(actor, location, false, None);
        }

        for prop_data in area.props.iter() {
            let location = Location::from_point(&prop_data.location, &area);
            debug!("Adding prop '{}' at '{:?}'", prop_data.prop.id, location);
            self.add_prop(prop_data, location, false);
        }

        let turn_timer = TurnTimer::new(&self);
        self.turn_timer = turn_timer;
        trace!("Set up turn timer for area.");

        for (index, transition) in self.area.transitions.iter().enumerate() {
            debug!("Adding transition '{}' at '{:?}'", index, transition.from);
            for y in 0..transition.size.height {
                for x in 0..transition.size.width {
                    self.transition_grid[(transition.from.x + x +
                        (transition.from.y + y) * self.area.width) as usize] = Some(index);
                }
            }
        }

        for (enc_index, enc_data) in area.encounters.iter().enumerate() {
            let encounter = &enc_data.encounter;

            let actors = encounter.gen_actors();
            for actor in actors {
                let location = match self.gen_location(&actor, &enc_data) {
                    None => {
                        warn!("Unable to generate location for encounter '{}'", encounter.id);
                        continue;
                    }, Some(location) => location,
                };

                self.add_actor(actor, location, false, Some(enc_index));
            }
        }
    }

    fn gen_location(&self, actor: &Rc<Actor>, data: &EncounterData) -> Option<Location> {
        let available = self.get_available_locations(actor, data);
        if available.is_empty() { return None; }

        let roll = rand::thread_rng().gen_range(0, available.len());

        let point = available[roll];
        let location = Location::from_point(&point, &self.area);
        Some(location)
    }

    fn get_available_locations(&self, actor: &Rc<Actor>, data: &EncounterData) -> Vec<Point> {
        let mut locations = Vec::new();

        let min_x = data.location.x;
        let min_y = data.location.y;
        let max_x = data.location.x + data.size.width - actor.race.size.width + 1;
        let max_y = data.location.y + data.size.height - actor.race.size.height + 1;

        for y in min_y..max_y {
            for x in min_x..max_x {
                if !self.area.coords_valid(x, y) { continue; }

                if !self.area.get_path_grid(&actor.race.size.id).is_passable(x, y) { continue; }

                let mut impass = false;
                for y in y..(y + actor.race.size.height) {
                    for x in x..(x + actor.race.size.width) {
                        let index = (x + y * self.area.width) as usize;
                        if self.entity_grid[index].is_some() {
                            impass = true;
                            break;
                        }
                    }
                }

                if impass { continue; }

                locations.push(Point::new(x, y));
            }
        }

        locations
    }

    pub fn is_terrain_passable(&self, size: &str, x: i32, y: i32) -> bool {
        if !self.area.coords_valid(x, y) { return false; }

        if !self.area.get_path_grid(size).is_passable(x, y) { return false; }

        true
    }

    pub fn is_passable_size(&self, size: &Rc<ObjectSize>, x: i32, y: i32) -> bool {
        if !self.is_terrain_passable(&size.id, x, y) { return false; }

        size.points(x, y).all(|p| self.point_size_passable(p.x, p.y))
    }

    pub fn is_passable(&self, requester: &Ref<EntityState>,
                       new_x: i32, new_y: i32) -> bool {
        if !self.is_terrain_passable(&requester.size(), new_x, new_y) { return false; }

        requester.points(new_x, new_y)
           .all(|p| self.point_entities_passable(&requester, p.x, p.y))
    }

    pub fn prop_index_valid(&self, index: usize) -> bool {
        if index >= self.props.len() { return false; }

        self.props[index].is_some()
    }

    pub fn prop_index_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.area.coords_valid(x, y) { return None; }

        let x = x as usize;
        let y = y as usize;
        self.prop_grid[x + y * self.area.width as usize]
    }

    pub fn check_create_prop_container_at(&mut self, x: i32, y: i32) {
        match self.prop_index_at(x, y) {
            Some(_) => return,
            None => (),
        };

        let prop = match Module::prop(&Module::rules().loot_drop_prop) {
            None => {
                warn!("Unable to generate prop for item drop as the loot_drop_prop does not exist.");
                return;
            }, Some(prop) => prop,
        };

        let location = Location::new(x, y, &self.area);
        let prop_data = PropData {
            prop,
            location: location.to_point(),
            items: Vec::new(),
        };

        self.add_prop(&prop_data, location, true);
    }

    pub fn prop_mut_at(&mut self, x: i32, y: i32) -> Option<&mut PropState> {
        let index = match self.prop_index_at(x, y) {
            None => return None,
            Some(index) => index,
        };

        Some(self.get_prop_mut(index))
    }

    pub fn prop_at(&self, x: i32, y: i32) -> Option<&PropState> {
        let index = match self.prop_index_at(x, y) {
            None => return None,
            Some(index) => index,
        };

        Some(self.get_prop(index))
    }

    pub fn get_entity_at(&self, x: i32, y: i32) -> Option<Rc<RefCell<EntityState>>> {
        if !self.area.coords_valid(x, y) { return None; }

        let index = match self.entity_grid.get((x + y * self.area.width) as usize).unwrap() {
            &None => return None,
            &Some(index) => index,
        };

        Some(self.get_entity(index))
    }

    pub fn get_transition_at(&self, x: i32, y: i32) -> Option<&Transition> {
        if !self.area.coords_valid(x, y) { return None; }

        let index = match self.transition_grid[(x + y * self.area.width) as usize] {
            None => return None,
            Some(index) => index,
        };

        self.area.transitions.get(index)
    }

    fn compute_pc_visibility(&mut self, entity: &EntityState, delta_x: i32, delta_y: i32) {
        calculate_los(&mut self.pc_vis, &mut self.pc_explored, &self.area, entity, delta_x, delta_y);
    }

    /// whether the pc has current visibility to the specified coordinations
    /// No bounds checking is done on the `x` and `y` arguments
    pub fn is_pc_visible(&self, x: i32, y: i32) -> bool {
        self.pc_vis[(x + y * self.area.width) as usize]
    }

    /// whether the pc has current explored vis to the specified coordinates
    /// No bounds checking is done
    pub fn is_pc_explored(&self, x: i32, y: i32) -> bool {
        self.pc_explored[(x + y * self.area.width) as usize]
    }

    fn point_size_passable(&self, x: i32, y: i32) -> bool {
        if !self.area.coords_valid(x, y) { return false; }

        let grid_index = self.entity_grid[(x + y * self.area.width) as usize];

        match grid_index {
            None => true,
            Some(_) => false,
        }
    }

    fn point_entities_passable(&self, requester: &Ref<EntityState>,
                               x: i32, y: i32) -> bool {
        if !self.area.coords_valid(x, y) { return false; }

        let grid_index = self.entity_grid[(x + y * self.area.width) as usize];

        match grid_index {
            None => true, // grid position is empty
            Some(index) => (index == requester.index),
        }
    }

    pub(crate) fn add_prop(&mut self, prop_data: &PropData, location: Location, temporary: bool) -> bool {
        let prop = &prop_data.prop;

        if !self.area.coords_valid(location.x, location.y) { return false; }
        if !self.area.coords_valid(location.x + prop.size.width, location.y + prop.size.height) {
            return false;
        }

        let prop_state = PropState::new(prop_data, location, temporary);

        let start_x = prop_state.location.x as usize;
        let start_y = prop_state.location.y as usize;
        let end_x = start_x + prop_state.prop.size.width as usize;
        let end_y = start_y + prop_state.prop.size.height as usize;

        let index = self.find_prop_index_to_add();
        for y in start_y..end_y {
            for x in start_x..end_x {
                self.prop_grid[x + y * self.area.width as usize] = Some(index);
            }
        }

        self.props[index] = Some(prop_state);

        true
    }

    pub(crate) fn remove_prop(&mut self, index: usize) {
        {
            let prop = match self.props[index] {
                None => return,
                Some(ref prop) => prop,
            };
            trace!("Removing prop '{}'", prop.prop.id);

            let start_x = prop.location.x as usize;
            let start_y = prop.location.y as usize;
            let end_x = start_x + prop.prop.size.width as usize;
            let end_y = start_y + prop.prop.size.height as usize;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    self.prop_grid[x + y * self.area.width as usize] = None;
                }
            }
        }

        self.props[index] = None;
    }

    pub(crate) fn add_actor(&mut self, actor: Rc<Actor>, location: Location,
                            is_pc: bool, ai_group: Option<usize>) -> bool {
        let entity = Rc::new(RefCell::new(EntityState::new(actor,
                                                           location.clone(),
                                                           0,
                                                           is_pc,
                                                           ai_group)));
        self.add_entity(entity, location)
    }

    pub(crate) fn add_entity(&mut self, entity: Rc<RefCell<EntityState>>,
                                location: Location) -> bool {
        let x = location.x;
        let y = location.y;

        if !self.area.coords_valid(x, y) { return false; }

        if !self.is_passable(&entity.borrow(), x, y) { return false; }

        entity.borrow_mut().actor.compute_stats();
        entity.borrow_mut().actor.init();

        let new_index = self.find_entity_index_to_add();
        entity.borrow_mut().index = new_index;
        entity.borrow_mut().location = location;

        for p in entity.borrow().points(x, y) {
            self.update_entity_grid(p.x, p.y, Some(new_index));
        }

        if entity.borrow().is_pc() {
            self.compute_pc_visibility(&entity.borrow(), 0, 0);
        }

        self.turn_timer.add(&entity);
        self.entities[new_index] = Some(entity);

        self.listeners.notify(&self);
        true
    }

    pub fn move_entity(&mut self, entity: &Rc<RefCell<EntityState>>, x: i32, y: i32, squares: u32) -> bool {
        let old_x = entity.borrow().location.x;
        let old_y = entity.borrow().location.y;
        if !entity.borrow_mut().move_to(self, x, y, squares) { return false; }

        self.update_entity_position(entity, old_x, old_y);

        true
    }

    fn update_entity_position(&mut self, entity: &Rc<RefCell<EntityState>>,
                                           old_x: i32, old_y: i32) {
        self.clear_entity_points(&*entity.borrow(), old_x, old_y);

        for p in entity.borrow().location_points() {
            self.update_entity_grid(p.x, p.y, Some(entity.borrow().index));
        }

        if entity.borrow().is_pc() {
            let d_x = old_x - entity.borrow().location.x;
            let d_y = old_y - entity.borrow().location.y;
            self.pc_vis_delta = (d_x, d_y);

            self.compute_pc_visibility(&*entity.borrow(), d_x, d_y);

            self.turn_timer.check_ai_activation(entity, &self.area);
        }
    }

    fn clear_entity_points(&mut self, entity: &EntityState, x: i32, y: i32) {
        for p in entity.points(x, y) {
            self.update_entity_grid(p.x, p.y, None);
        }
    }

    fn update_entity_grid(&mut self, x: i32, y: i32, index: Option<usize>) {
        *self.entity_grid.get_mut((x + y * self.area.width) as usize).unwrap() = index;
    }

    pub fn get_last_entity(&self) -> Option<&Rc<RefCell<EntityState>>> {
        for item in self.entities.iter().rev() {
            if let &Some(ref entity) = item {
                return Some(entity);
            }
        }

        None
    }

    pub fn prop_iter<'a>(&'a self) -> PropIterator {
        PropIterator { area_state: &self, index: 0 }
    }

    pub fn get_prop<'a>(&'a self, index: usize) -> &'a PropState {
        &self.props[index].as_ref().unwrap()
    }

    pub fn get_prop_mut<'a>(&'a mut self, index: usize) -> &'a mut PropState {
        let prop_ref = self.props[index].as_mut();
        prop_ref.unwrap()
    }

    pub fn props_len(&self) -> usize {
        self.props.len()
    }

    pub fn entity_iter(&self) -> EntityIterator {
        EntityIterator { area_state: &self, index: 0 }
    }

    pub fn has_entity(&self, index: usize) -> bool {
        if index >= self.entities.len() {
            false
        } else {
            self.entities[index].is_some()
        }
    }

    pub fn check_get_entity(&self, index: usize) -> Option<Rc<RefCell<EntityState>>> {
        if index >= self.entities.len() {
            None
        } else {
            match self.entities[index] {
                None => None,
                Some(ref entity) => Some(Rc::clone(entity)),
            }
        }
    }

    pub fn get_entity(&self, index: usize) -> Rc<RefCell<EntityState>> {
        let entity = &self.entities[index];

        Rc::clone(&entity.as_ref().unwrap())
    }

    pub (crate) fn update(&mut self, millis: u32) -> Option<&Rc<RefCell<EntityState>>> {
        let elapsed_millis = millis - self.last_time_millis;
        self.last_time_millis = millis;

        let real_time = !self.turn_timer.is_active();

        // removal does not shuffle the vector around, so we can safely just iterate
        let mut notify = false;
        let len = self.entities.len();
        for index in 0..len {
            let entity = {
                let entity = match &self.entities[index].as_ref() {
                    &None => continue,
                    &Some(entity) => entity,
                };

                if real_time {
                    entity.borrow_mut().actor.update(elapsed_millis);
                } else {
                    entity.borrow_mut().actor.check_removal();
                }

                if !entity.borrow().is_marked_for_removal() { continue; }

                Rc::clone(entity)
            };

            self.remove_entity_at_index(&entity, index);
            notify = true;
        }

        let len = self.props.len();
        for index in 0..len {
            {
                let prop = match self.props[index] {
                    None => continue,
                    Some(ref prop) => prop,
                };

                if !prop.is_marked_for_removal() { continue; }
            }

            self.remove_prop(index);
            notify = true;
        }

        self.feedback_text.iter_mut().for_each(|f| f.update());
        self.feedback_text.retain(|f| f.retain());

        let remove_targeter = match self.targeter {
            None => false,
            Some(ref targeter) => targeter.borrow().cancel()
        };

        if remove_targeter {
            self.targeter.take();
        }

        if notify {
            self.listeners.notify(&self);
        }

        self.turn_timer.current()
    }

    pub(crate) fn remove_entity(&mut self, entity: &Rc<RefCell<EntityState>>) {
        if !entity.borrow().location.is_in(&self) {
            warn!("Unable to remove entity '{}' from area '{}' as it is not in the area.",
                  entity.borrow().actor.actor.id, self.area.id);
        }

        let index = entity.borrow().index;
        self.remove_entity_at_index(entity, index);
        self.listeners.notify(&self);
    }

    fn remove_entity_at_index(&mut self, entity: &Rc<RefCell<EntityState>>, index: usize) {
        trace!("Removing entity '{}' with index '{}'", entity.borrow().actor.actor.name, index);
        let x = entity.borrow().location.x;
        let y = entity.borrow().location.y;
        self.clear_entity_points(&*entity.borrow(), x, y);
        self.entities[index] = None;
        self.turn_timer.remove(entity);
    }

    fn find_prop_index_to_add(&mut self) -> usize {
        for (index, item) in self.props.iter().enumerate() {
            if item.is_none() { return index; }
        }

        self.props.push(None);
        self.props.len() - 1
    }

    fn find_entity_index_to_add(&mut self) -> usize {
        for (index, item) in self.entities.iter().enumerate() {
            if item.is_none() {
                return index;
            }
        }

        self.entities.push(None);
        self.entities.len() - 1
    }

    pub fn add_feedback_text(&mut self, text: String, target: &Rc<RefCell<EntityState>>,
                             color: Color) {
        let width = target.borrow().size.width as f32;
        let pos_x = target.borrow().location.x as f32 + width / 2.0;
        let pos_y = target.borrow().location.y as f32 - 1.5;

        self.feedback_text.push(AreaFeedbackText::new(text, pos_x, pos_y, color));
    }

    pub fn feedback_text_iter(&self) -> Iter<AreaFeedbackText> {
        self.feedback_text.iter()
    }
}

pub struct PropIterator<'a> {
    area_state: &'a AreaState,
    index: usize,
}

impl<'a> Iterator for PropIterator<'a> {
    type Item = &'a PropState;
    fn next(&mut self) -> Option<&'a PropState> {
        loop {
            let next = self.area_state.props.get(self.index);
            self.index += 1;

            match next {
                None => return None,
                Some(prop) => match prop {
                    &None => continue,
                    &Some(ref prop) => return Some(prop),
                }
            }
        }
    }
}

pub struct EntityIterator<'a> {
    area_state: &'a AreaState,
    index: usize,
}

impl<'a> Iterator for EntityIterator<'a> {
    type Item = Rc<RefCell<EntityState>>;
    fn next(&mut self) -> Option<Rc<RefCell<EntityState>>> {
        loop {
            let next = self.area_state.entities.get(self.index);

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
