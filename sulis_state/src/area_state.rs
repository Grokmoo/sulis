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

use std::io::Error;
use std::{ptr};
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use std::collections::HashSet;

use rand::{self, Rng};

use sulis_core::util::{invalid_data_error, Point};
use sulis_core::config::CONFIG;
use sulis_module::{Actor, Area, LootList, Module, ObjectSize, prop};
use sulis_module::area::{EncounterData, PropData, Transition, TriggerKind};
use script::AreaTargeter;
use save_state::{AreaSaveState};
use *;

pub struct TriggerState {
    pub(crate) fired: bool,
    pub(crate) enabled: bool,
}

pub struct AreaState {
    pub area: Rc<Area>,

    // Members that need to be saved
    pub(crate) pc_explored: Vec<bool>,
    pub on_load_fired: bool,
    props: Vec<Option<PropState>>,
    entities: Vec<usize>,
    surfaces: Vec<usize>,
    pub(crate) triggers: Vec<TriggerState>,
    pub(crate) merchants: Vec<Merchant>,

    pub listeners: ChangeListenerList<AreaState>,

    prop_grid: Vec<Option<usize>>,
    entity_grid: Vec<Vec<usize>>,
    surface_grid: Vec<Vec<usize>>,
    transition_grid: Vec<Option<usize>>,
    trigger_grid: Vec<Option<usize>>,

    prop_vis_grid: Vec<bool>,
    prop_pass_grid: Vec<bool>,

    pub pc_vis_delta: (bool, i32, i32),
    pc_vis: Vec<bool>,

    feedback_text: Vec<AreaFeedbackText>,
    scroll_to_callback: Option<Rc<RefCell<EntityState>>>,

    targeter: Option<Rc<RefCell<AreaTargeter>>>,
}

impl PartialEq for AreaState {
    fn eq(&self, other: &AreaState) -> bool {
        self.area == other.area
    }
}

impl AreaState {
    pub fn new(area: Rc<Area>) -> AreaState {
        let dim = (area.width * area.height) as usize;
        let entity_grid = vec![Vec::new();dim];
        let surface_grid = vec![Vec::new();dim];
        let transition_grid = vec![None;dim];
        let prop_grid = vec![None;dim];
        let trigger_grid = vec![None;dim];
        let pc_vis = vec![false;dim];
        let pc_explored = vec![false;dim];

        info!("Initializing area state for '{}'", area.name);
        AreaState {
            area,
            props: Vec::new(),
            entities: Vec::new(),
            surfaces: Vec::new(),
            triggers: Vec::new(),
            transition_grid,
            entity_grid,
            surface_grid,
            prop_grid,
            trigger_grid,
            prop_vis_grid: vec![true;dim],
            prop_pass_grid: vec![true;dim],
            listeners: ChangeListenerList::default(),
            pc_vis,
            pc_explored,
            pc_vis_delta: (false, 0, 0),
            feedback_text: Vec::new(),
            scroll_to_callback: None,
            targeter: None,
            merchants: Vec::new(),
            on_load_fired: false,
        }
    }

    pub fn load(id: &str, save: AreaSaveState) -> Result<AreaState, Error> {
        let area = match Module::area(id) {
            None => invalid_data_error(&format!("Unable to find area '{}'", id)),
            Some(area) => Ok(area),
        }?;

        let mut area_state = AreaState::new(area);

        area_state.on_load_fired = save.on_load_fired;

        for (index, mut buf) in save.pc_explored.into_iter().enumerate() {
            for i in 0..64 {
                if buf % 2 == 1 {
                    let pc_exp_index = i + index * 64;
                    if pc_exp_index > area_state.pc_explored.len() { break; }
                    area_state.pc_explored[pc_exp_index] = true;
                }
                buf = buf / 2;
            }
        }

        for prop_save_state in save.props {
            let prop = match Module::prop(&prop_save_state.id) {
                None => invalid_data_error(&format!("No prop with ID '{}'", prop_save_state.id)),
                Some(prop) => Ok(prop),
            }?;

            let location = Location::from_point(&prop_save_state.location, &area_state.area);

            let prop_data = PropData {
                prop,
                location: prop_save_state.location,
                items: Vec::new(),
                enabled: prop_save_state.enabled,
            };

            let index = area_state.add_prop(&prop_data, location, false)?;
            area_state.props[index].as_mut().unwrap()
                .load_interactive(prop_save_state.interactive)?;

            area_state.update_prop_vis_pass_grid(index);
        }

        for (index, trigger_save) in save.triggers.into_iter().enumerate() {
            if index >= area_state.area.triggers.len() {
                return invalid_data_error(&format!("Too many triggers defined in save"));
            }

            let trigger_state = TriggerState {
                enabled: trigger_save.enabled,
                fired: trigger_save.fired,
            };
            area_state.add_trigger(index, trigger_state);
        }

        area_state.add_transitions_from_area();

        for merchant_save in save.merchants {
            area_state.merchants.push(Merchant::load(merchant_save)?);
        }

        Ok(area_state)
    }

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
            match self.add_actor(actor, location, false, None) {
                Ok(_) => (),
                Err(e) => {
                    warn!("Error adding actor to area: {}", e);
                }
            }
        }

        for prop_data in area.props.iter() {
            let location = Location::from_point(&prop_data.location, &area);
            debug!("Adding prop '{}' at '{:?}'", prop_data.prop.id, location);
            match self.add_prop(prop_data, location, false) {
                Err(e) => {
                    warn!("Unable to add prop at {:?}", &prop_data.location);
                    warn!("{}", e);
                }, Ok(_) => (),
            }
        }

        for (index, trigger) in area.triggers.iter().enumerate() {
            let trigger_state = TriggerState {
                fired: false,
                enabled: trigger.initially_enabled,
            };

            self.add_trigger(index, trigger_state);
        }

        self.add_transitions_from_area();

        for (enc_index, enc_data) in area.encounters.iter().enumerate() {
            let encounter = &enc_data.encounter;
            if !encounter.auto_spawn { continue; }

            self.spawn_encounter(enc_index, enc_data, true);
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

    pub fn targeter(&mut self) -> Option<Rc<RefCell<AreaTargeter>>> {
        match self.targeter {
            None => None,
            Some(ref targeter) => Some(Rc::clone(targeter)),
        }
    }

    pub (crate) fn set_targeter(&mut self, targeter: AreaTargeter) {
        self.targeter = Some(Rc::new(RefCell::new(targeter)));
    }

    pub fn push_scroll_to_callback(&mut self, entity: Rc<RefCell<EntityState>>) {
        self.scroll_to_callback = Some(entity);
    }

    pub fn pop_scroll_to_callback(&mut self) -> Option<Rc<RefCell<EntityState>>> {
        self.scroll_to_callback.take()
    }

    fn add_transitions_from_area(&mut self) {
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

    fn add_trigger(&mut self, index: usize, trigger_state: TriggerState) {
        let trigger = &self.area.triggers[index];
        self.triggers.push(trigger_state);

        let (location, size) = match trigger.kind {
            TriggerKind::OnPlayerEnter { location, size } => (location, size),
            _ => return,
        };

        let start_x = location.x as usize;
        let start_y = location.y as usize;
        let end_x = start_x + size.width as usize;
        let end_y = start_y + size.height as usize;

        for y in start_y..end_y {
            for x in start_x..end_x {
                self.trigger_grid[x + y * self.area.width as usize] = Some(index);
            }
        }
    }

    pub fn check_encounter_cleared(&self, index: usize, parent: &Rc<RefCell<EntityState>>,
                                   target: &Rc<RefCell<EntityState>>) {
        let mgr = GameState::turn_manager();
        for entity in mgr.borrow().entity_iter() {
            if let Some(entity_index) = entity.borrow().ai_group() {
                if entity_index == index && entity.borrow().actor.hp() > 0 { return; }
            }
        }

        for trigger_index in self.area.encounters[index].triggers.iter() {
            let trigger = &self.area.triggers[*trigger_index];

            match trigger.kind {
                TriggerKind::OnEncounterCleared { .. } => {
                    info!("    Calling OnEncounterCleared");
                    GameState::add_ui_callback(trigger.on_activate.clone(), parent, target);
                },
                _ => (),
            }
        }
    }

    pub fn spawn_encounter_at(&mut self, x: i32, y: i32) -> bool {
        let area = Rc::clone(&self.area);

        for (enc_index, enc_data) in area.encounters.iter().enumerate() {
            if enc_data.location.x != x || enc_data.location.y != y { continue; }

            // this method is called by script, still spawn in debug mode
            self.spawn_encounter(enc_index, enc_data, false);
            return true
        }

        false
    }

    pub fn spawn_encounter(&mut self, enc_index: usize, enc_data: &EncounterData,
                           respect_debug: bool) {
        if respect_debug && !CONFIG.debug.encounter_spawning { return; }
        let encounter = &enc_data.encounter;
        let actors = encounter.gen_actors();
        for actor in actors {
            let location = match self.gen_location(&actor, &enc_data) {
                None => {
                    warn!("Unable to generate location for encounter '{}'", encounter.id);
                    continue;
                }, Some(location) => location,
            };

            match self.add_actor(actor, location, false, Some(enc_index)) {
                Ok(_) => (),
                Err(e) => {
                    warn!("Error adding actor for spawned encounter: {}", e);
                }
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
                        if self.entity_grid[index].len() > 0 {
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

    pub fn is_passable(&self, requester: &Ref<EntityState>, entities_to_ignore: &Vec<usize>,
                       new_x: i32, new_y: i32) -> bool {
        if !self.is_terrain_passable(&requester.size(), new_x, new_y) { return false; }

        requester.points(new_x, new_y)
           .all(|p| self.point_entities_passable(entities_to_ignore, p.x, p.y))
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
            enabled: true,
            location: location.to_point(),
            items: Vec::new(),
        };

        match self.add_prop(&prop_data, location, true) {
            Err(e) => {
                warn!("Unable to add temp container at {},{}", x, y);
                warn!("{}", e);
            }, Ok(_) => (),
        }
    }

    pub fn set_prop_enabled_at(&mut self, x: i32, y: i32, enabled: bool) -> bool {
        match self.prop_mut_at(x, y) {
            None => false,
            Some(ref mut prop) => {
                prop.set_enabled(enabled);
                true
            }
        }
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

    pub fn toggle_prop_active(&mut self, index: usize) {
        {
            let state = self.get_prop_mut(index);
            state.toggle_active();
            if !state.is_door() { return; }
        }

        self.update_prop_vis_pass_grid(index);

        self.pc_vis_delta = (true, 0, 0);
        for member in GameState::party().iter() {
            self.compute_pc_visibility(member, 0, 0);
        }
        self.update_view_visibility();
    }

    fn update_prop_vis_pass_grid(&mut self, index: usize) {
        // borrow checker isn't smart enough to let us use get_prop_mut here
        let prop_ref = self.props[index].as_mut();
        let state = prop_ref.unwrap();

        if !state.is_door() { return; }

        let width = self.area.width;
        let start_x = state.location.x;
        let start_y = state.location.y;
        let end_x = start_x + state.prop.size.width;
        let end_y = start_y + state.prop.size.height;

        if state.is_active() {
            for y in start_y..end_y {
                for x in start_x..end_x {
                    self.prop_vis_grid[(x + y * width) as usize] = true;
                    self.prop_pass_grid[(x + y * width) as usize] = true;
                }
            }
        } else {
            match state.prop.interactive {
                prop::Interactive::Door { ref closed_invis, ref closed_impass, .. } => {
                    for p in closed_invis.iter() {
                        self.prop_vis_grid[(p.x + start_x + (p.y + start_y) * width) as usize] = false;
                    }

                    for p in closed_impass.iter() {
                        self.prop_pass_grid[(p.x + start_x + (p.y + start_y) * width) as usize] = false;
                    }
                },
                _ => (),
            }
        }
    }

    pub fn get_entity_at(&self, x: i32, y: i32) -> Option<Rc<RefCell<EntityState>>> {
        if !self.area.coords_valid(x, y) { return None; }

        let index = {
            let vec = &self.entity_grid[(x + y * self.area.width) as usize];
            if vec.is_empty() { return None; }
            vec[0]
        };

        let mgr = GameState::turn_manager();
        let mgr = mgr.borrow();
        mgr.entity_checked(index)
    }

    pub fn get_transition_at(&self, x: i32, y: i32) -> Option<&Transition> {
        if !self.area.coords_valid(x, y) { return None; }

        let index = match self.transition_grid[(x + y * self.area.width) as usize] {
            None => return None,
            Some(index) => index,
        };

        self.area.transitions.get(index)
    }

    pub fn has_visibility(&self, parent: &EntityState, target: &EntityState) -> bool {
        has_visibility(&self.area, &self.prop_vis_grid, parent, target)
    }

    pub fn compute_pc_visibility(&mut self, entity: &Rc<RefCell<EntityState>>, delta_x: i32, delta_y: i32) {
        calculate_los(&mut self.pc_explored, &self.area, &self.prop_vis_grid,
                      &mut entity.borrow_mut(), delta_x, delta_y);
    }

    pub fn update_view_visibility(&mut self) {
        unsafe {
            ptr::write_bytes(self.pc_vis.as_mut_ptr(), 0, self.pc_vis.len())
        }

        for entity in GameState::party().iter() {
            let entity = entity.borrow();
            let new_vis = entity.pc_vis();
            for y in 0..self.area.height {
                for x in 0..self.area.width {
                    let index = (x + y * self.area.width) as usize;
                    self.pc_vis[index] = self.pc_vis[index] || new_vis[index]
                }
            }
        }
    }

    pub fn set_trigger_enabled_at(&mut self, x: i32, y: i32, enabled: bool) -> bool{
        if !self.area.coords_valid(x, y) {
            warn!("Invalid coords to enable trigger at {},{}", x, y);
            return false;
        }

        let index = match self.trigger_grid[(x + y * self.area.width) as usize] {
            None => return false,
            Some(index) => index,
        };

        self.triggers[index].enabled = enabled;
        true
    }

    fn check_trigger_grid(&mut self, entity: &Rc<RefCell<EntityState>>) {
        let index = {
            let entity = entity.borrow();
            let grid_index = entity.location.x + entity.location.y * self.area.width;
            match self.trigger_grid[grid_index as usize] {
                None => return,
                Some(index) => index,
            }
        };

        if !self.triggers[index].enabled || self.triggers[index].fired { return; }

        self.triggers[index].fired = true;
        GameState::add_ui_callback(self.area.triggers[index].on_activate.clone(), entity, entity);
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

        let index = (x + y * self.area.width) as usize;
        if !self.prop_pass_grid[index] { return false; }

        let grid_index = &self.entity_grid[index];

        grid_index.is_empty()
    }

    fn point_entities_passable(&self, entities_to_ignore: &Vec<usize>,
                               x: i32, y: i32) -> bool {
        if !self.area.coords_valid(x, y) { return false; }

        let index = (x + y * self.area.width) as usize;
        if !self.prop_pass_grid[index] { return false; }

        let grid = &self.entity_grid[index];

        for index in grid.iter() {
            if !entities_to_ignore.contains(index) { return false; }
        }
        true
    }

    pub(crate) fn add_prop(&mut self, prop_data: &PropData, location: Location, temporary: bool) -> Result<usize, Error> {
        let prop = &prop_data.prop;

        if !self.area.coords_valid(location.x, location.y) {
            return invalid_data_error(&format!("Prop location outside area bounds"));
        }
        if !self.area.coords_valid(location.x + prop.size.width, location.y + prop.size.height) {
            return invalid_data_error(&format!("Prop location outside area bounds"));
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
        self.update_prop_vis_pass_grid(index);

        Ok(index)
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
                            is_pc: bool, ai_group: Option<usize>) -> Result<usize, Error> {
        let entity = Rc::new(RefCell::new(EntityState::new(actor,
                                                           location.clone(),
                                                           0,
                                                           is_pc,
                                                           ai_group)));
        match self.add_entity(entity, location) {
            Ok(index) => Ok(index),
            Err(e) => {
                warn!("Unable to add entity to area");
                warn!("{}", e);
                Err(e)
            }
        }
    }

    pub(crate) fn entities_with_points(&self, points: &Vec<Point>) -> Vec<usize> {
        let mut result = HashSet::new();
        for p in points {
            for entity in self.entity_grid[(p.x + p.y * self.area.width) as usize].iter() {
                result.insert(*entity);
            }
        }

        result.into_iter().collect()
    }

    #[must_use]
    pub(crate) fn remove_surface(&mut self, index: usize, points: &Vec<Point>) -> HashSet<usize> {
        info!("Removing surface {} from area", index);

        let mut entities = HashSet::new();
        for p in points {
            self.surface_grid[(p.x + p.y * self.area.width) as usize].retain(|i| *i != index);
            for entity in self.entity_grid[(p.x + p.y * self.area.width) as usize].iter() {
                entities.insert(*entity);
            }
        }

        self.surfaces.retain(|i| *i != index);

        entities
    }

    #[must_use]
    pub(crate) fn add_surface(&mut self, index: usize, points: Vec<Point>) -> HashSet<usize> {
        self.surfaces.push(index);

        let mut entities = HashSet::new();
        for p in points {
            if !self.area.coords_valid(p.x, p.y) {
                warn!("Attempted to add surface with invalid coordinate {},{}", p.x, p.y);
                continue;
            }

            self.surface_grid[(p.x + p.y * self.area.width) as usize].push(index);

            for entity in self.entity_grid[(p.x + p.y * self.area.width) as usize].iter() {
                entities.insert(*entity);
            }
        }

        entities
    }

    pub(crate) fn add_entity(&mut self, entity: Rc<RefCell<EntityState>>,
                                location: Location) -> Result<usize, Error> {

        let mgr = GameState::turn_manager();
        let index = mgr.borrow_mut().add_entity(&entity);
        self.transition_entity_to(entity, index, location)
    }

    pub(crate) fn transition_entity_to(&mut self, entity: Rc<RefCell<EntityState>>, index: usize,
                                location: Location) -> Result<usize, Error> {
        let x = location.x;
        let y = location.y;

        if !self.area.coords_valid(x, y) {
            return invalid_data_error(&format!("entity location is out of bounds: {},{}", x, y));
        }

        let entities_to_ignore = vec![entity.borrow().index];
        if !self.is_passable(&entity.borrow(), &entities_to_ignore, x, y) {
            warn!("Entity location is not passable: {},{}", x, y);
        }

        entity.borrow_mut().actor.compute_stats();
        entity.borrow_mut().actor.init();

        entity.borrow_mut().location = location;
        self.entities.push(index);

        let mgr = GameState::turn_manager();
        let surfaces = self.add_entity_points(&entity.borrow());
        for surface in surfaces {
            mgr.borrow_mut().add_to_surface(entity.borrow().index, surface);
        }

        if entity.borrow().is_party_member() {
            self.compute_pc_visibility(&entity, 0, 0);
        }

        self.listeners.notify(&self);
        Ok(index)
    }

    pub fn move_entity(&mut self, entity: &Rc<RefCell<EntityState>>, x: i32, y: i32, squares: u32) -> bool {
        let old_x = entity.borrow().location.x;
        let old_y = entity.borrow().location.y;
        if !entity.borrow_mut().move_to(x, y, squares) { return false; }

        self.update_entity_position(entity, old_x, old_y);

        true
    }

    fn update_entity_position(&mut self, entity: &Rc<RefCell<EntityState>>,
                                           old_x: i32, old_y: i32) {
        let entity_index = entity.borrow().index;
        let old_surfaces = self.clear_entity_points(&entity.borrow(), old_x, old_y);
        let new_surfaces = self.add_entity_points(&entity.borrow());

        let mgr = GameState::turn_manager();
        // remove from surfaces in old but not in new
        for surface in old_surfaces.difference(&new_surfaces) {
            mgr.borrow_mut().remove_from_surface(entity_index, *surface);
        }

        // add to surfaces in new but not in old
        for surface in new_surfaces.difference(&old_surfaces) {
            mgr.borrow_mut().add_to_surface(entity_index, *surface);
        }

        for surface in new_surfaces.intersection(&old_surfaces) {
            mgr.borrow_mut().increment_surface_squares_moved(entity_index, *surface);
        }

        let is_pc = entity.borrow().is_party_member();

        if is_pc {
            let d_x = old_x - entity.borrow().location.x;
            let d_y = old_y - entity.borrow().location.y;
            self.pc_vis_delta = (true, d_x, d_y);

            self.compute_pc_visibility(&entity, d_x, d_y);
            self.update_view_visibility();

            self.check_trigger_grid(&entity);
        }

        let mgr = GameState::turn_manager();
        mgr.borrow_mut().check_ai_activation(entity, self);
    }

    #[must_use]
    fn add_entity_points(&mut self, entity: &EntityState) -> HashSet<usize> {
        let mut surfaces = HashSet::new();
        for p in entity.location_points() {
            self.add_entity_to_grid(p.x, p.y, entity.index);
            for surface in self.surface_grid[(p.x + p.y * self.area.width) as usize].iter() {
                surfaces.insert(*surface);
            }
        }

        surfaces
    }

    #[must_use]
    fn clear_entity_points(&mut self, entity: &EntityState, x: i32, y: i32) -> HashSet<usize> {
        let mut surfaces = HashSet::new();
        for p in entity.points(x, y) {
            self.remove_entity_from_grid(p.x, p.y, entity.index);
            for surface in self.surface_grid[(p.x + p.y * self.area.width) as usize].iter() {
                surfaces.insert(*surface);
            }
        }

        surfaces
    }

    fn add_entity_to_grid(&mut self, x: i32, y: i32, index: usize) {
        self.entity_grid[(x + y * self.area.width) as usize].push(index);
    }

    fn remove_entity_from_grid(&mut self, x: i32, y: i32, index: usize) {
        self.entity_grid[(x + y * self.area.width) as usize].retain(|e| *e != index);
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

    pub (crate) fn update(&mut self) {
        let mut notify = false;
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
    }

    #[must_use]
    pub fn remove_entity(&mut self, entity: &Rc<RefCell<EntityState>>) -> HashSet<usize> {
        let entity = entity.borrow();
        let index = entity.index;
        trace!("Removing entity '{}' with index '{}'", entity.actor.actor.name, index);
        let x = entity.location.x;
        let y = entity.location.y;
        let surfaces = self.clear_entity_points(&entity, x, y);

        self.entities.retain(|i| *i != index);

        surfaces
    }

    fn find_prop_index_to_add(&mut self) -> usize {
        for (index, item) in self.props.iter().enumerate() {
            if item.is_none() { return index; }
        }

        self.props.push(None);
        self.props.len() - 1
    }

    pub fn add_feedback_text(&mut self, text: String, target: &Rc<RefCell<EntityState>>,
                             color_kind: area_feedback_text::ColorKind, move_rate: f32) {
        if text.trim().is_empty() { return; }

        let mut area_pos = target.borrow().location.to_point();
        loop {
            let mut area_pos_valid = true;

            let area_pos_y = area_pos.y as f32;
            for text in self.feedback_text.iter() {
                let text_pos_y = text.area_pos().y as f32 - text.cur_hover_y();
                if (area_pos_y - text_pos_y).abs() < 0.7 {
                    area_pos.y -= 1;
                    area_pos_valid = false;
                    break;
                }
            }

            if area_pos_valid { break; }
            if area_pos.y == 0 { break; }
        }
        let width = target.borrow().size.width as f32;
        let pos_x = area_pos.x as f32 + width / 2.0;
        let pos_y = area_pos.y as f32 - 1.5;

        self.feedback_text.push(AreaFeedbackText::new(area_pos, text, pos_x, pos_y,
                                                      color_kind, move_rate));
    }

    pub fn feedback_text_iter(&mut self) -> impl Iterator<Item = &mut AreaFeedbackText> {
        self.feedback_text.iter_mut()
    }

    pub fn entity_iter(&self) -> impl Iterator<Item = &usize> {
        self.entities.iter()
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
