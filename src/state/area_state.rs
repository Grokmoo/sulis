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

use module::{Actor, Area, Module};
use module::area::Transition;
use state::{ChangeListenerList, EntityState, Location, TurnTimer};
use animation;

use std::cmp;
use std::time;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub const VIS_TILES: i32 = 18;

pub struct AreaState {
    pub area: Rc<Area>,
    pub listeners: ChangeListenerList<AreaState>,
    pub turn_timer: TurnTimer,
    entities: Vec<Option<Rc<RefCell<EntityState>>>>,

    entity_grid: Vec<Option<usize>>,
    transition_grid: Vec<Option<usize>>,

    pub pc_vis_cache_invalid: bool,
    pc_vis: Vec<bool>,
}

impl PartialEq for AreaState {
    fn eq(&self, other: &AreaState) -> bool {
        self.area == other.area
    }
}

impl AreaState {
    pub fn new(area: Rc<Area>) -> AreaState {
        let dim = (area.width * area.height) as usize;
        let entity_grid = vec![None;dim];
        let transition_grid = vec![None;dim];
        let pc_vis = vec![false;dim];

        AreaState {
            area,
            entities: Vec::new(),
            turn_timer: TurnTimer::default(),
            transition_grid,
            entity_grid,
            listeners: ChangeListenerList::default(),
            pc_vis,
            pc_vis_cache_invalid: true,
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
            self.add_actor(actor, location, false);
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
    }

    pub fn is_passable(&self, requester: &Ref<EntityState>,
                       new_x: i32, new_y: i32) -> bool {
        if !self.area.coords_valid(new_x, new_y) { return false; }

        if !self.area.get_path_grid(requester.size()).is_passable(new_x, new_y) {
            return false;
        }

        requester.points(new_x, new_y)
           .all(|p| self.point_entities_passable(&requester, p.x, p.y))
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

    fn compute_pc_visibility(&mut self, entity: &EntityState) {
        let start_time = time::Instant::now();
        let entity_x = entity.location.x + entity.size.size / 2;
        let entity_y = entity.location.y + entity.size.size / 2;

        let min_x = cmp::max(0, entity_x - VIS_TILES - 2);
        let max_x = cmp::min(self.area.width, entity_x + VIS_TILES + 2);
        let min_y = cmp::max(0, entity_y - VIS_TILES - 2);
        let max_y = cmp::min(self.area.height, entity_y + VIS_TILES + 2);

        let e_x = entity_x as f32;
        let e_y = entity_y as f32;

        for y in min_y..max_y {
            for x in min_x..max_x {
                let index = (x + y * self.area.width) as usize;
                // if !self.area.terrain.is_visible_index(index) {
                //     self.pc_vis[index] = false;
                //     continue;
                // }

                let xf = x as f32;
                let yf = y as f32;
                let dist_squared = (xf - e_x) * (xf - e_x) + (yf - e_y) * (yf - e_y);

                if dist_squared < (VIS_TILES * VIS_TILES) as f32 {
                    self.pc_vis[index] = true;
                } else {
                    self.pc_vis[index] = false;
                }
            }
        }

        self.pc_vis_cache_invalid = true;

        trace!("Visibility compute time: {}", animation::format_elapsed_secs(start_time.elapsed()));
    }

    /// whether the pc has current visibility to the specified coordinations
    /// No bounds checking is done on the `x` and `y` arguments
    pub fn is_pc_visible(&self, x: i32, y: i32) -> bool {
        self.pc_vis[(x + y * self.area.width) as usize]
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

    pub(in state) fn add_actor(&mut self, actor: Rc<Actor>,
                     location: Location, is_pc: bool) -> bool {
        let entity = EntityState::new(actor, location.clone(), 0, is_pc);
        let entity = Rc::new(RefCell::new(entity));
        self.add_entity(Rc::clone(&entity), location)
    }

    pub(in state) fn add_entity(&mut self, entity: Rc<RefCell<EntityState>>,
                                location: Location) -> bool {
        let x = location.x;
        let y = location.y;

        if !self.area.coords_valid(x, y) { return false; }

        if !self.is_passable(&entity.borrow(), x, y) { return false; }

        entity.borrow_mut().actor.compute_stats();
        entity.borrow_mut().actor.init();

        let new_index = self.find_index_to_add();
        entity.borrow_mut().index = new_index;
        entity.borrow_mut().location = location;

        for p in entity.borrow().points(x, y) {
            self.update_entity_grid(p.x, p.y, Some(new_index));
        }

        if entity.borrow().is_pc() {
            self.compute_pc_visibility(&entity.borrow());
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
            self.compute_pc_visibility(&*entity.borrow());

            self.turn_timer.check_ai_activation(entity);
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

    pub fn entity_iter(&self) -> EntityIterator {
        EntityIterator { area_state: &self, index: 0 }
    }

    fn get_entity(&self, index: usize) -> Rc<RefCell<EntityState>> {
        let entity = &self.entities[index];

        Rc::clone(&entity.as_ref().unwrap())
    }

    pub (in state) fn update(&mut self) -> Option<&Rc<RefCell<EntityState>>> {
        // removal does not shuffle the vector around, so we can safely just iterate
        let mut notify = false;
        let len = self.entities.len();
        for index in 0..len {
            let entity = match &self.entities[index].as_ref() {
                &None => continue,
                &Some(entity) => Rc::clone(entity),
            };
            if !entity.borrow().is_marked_for_removal() { continue; }

            self.remove_entity_at_index(&entity, index);
            notify = true;
        }

        if notify {
            self.listeners.notify(&self);
        }

        self.turn_timer.current()
    }

    pub(in state) fn remove_entity(&mut self, entity: &Rc<RefCell<EntityState>>) {
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

    fn find_index_to_add(&mut self) -> usize {
        for (index, item) in self.entities.iter().enumerate() {
            if item.is_none() {
                return index;
            }
        }

        self.entities.push(None);
        self.entities.len() - 1
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
