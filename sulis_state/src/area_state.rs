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

mod prop_handler;
use prop_handler::PropHandler;

use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Error;
use std::rc::Rc;
use std::time;

use crate::save_state::AreaSaveState;
use crate::script::AreaTargeter;
use crate::*;
use sulis_core::config::Config;
use sulis_core::util::{self, gen_rand, invalid_data_error, Point, Size};
use sulis_module::area::{Transition, TriggerKind, Trigger};
use sulis_module::{Actor, Area, LootList, Module, ObjectSize, Time};

pub struct TriggerState {
    pub(crate) fired: bool,
    pub(crate) enabled: bool,
}

impl TriggerState {
    pub fn can_fire(&self, trigger: &Trigger) -> bool {
        if !self.enabled { return false; }

        trigger.fire_more_than_once || !self.fired
    }
}

#[derive(Clone, Copy)]
pub enum PCVisRedraw {
    Full,
    Partial { delta_x: i32, delta_y: i32 },
    Not,
}

pub struct AreaState {
    pub area: GeneratedArea,
    pub area_gen_seed: u128,

    // Members that need to be saved
    pub(crate) pc_explored: Vec<bool>,
    pub on_load_fired: bool,
    entities: Vec<usize>,
    surfaces: Vec<usize>,
    pub(crate) triggers: Vec<TriggerState>,
    pub(crate) merchants: Vec<MerchantState>,

    pub(crate) entity_grid: Vec<Vec<usize>>,
    surface_grid: Vec<Vec<usize>>,
    transition_grid: Vec<Option<usize>>,
    trigger_grid: Vec<Option<usize>>,

    props: PropHandler,

    pc_vis_redraw: PCVisRedraw,
    pc_vis: Vec<bool>,

    feedback_text: Vec<AreaFeedbackText>,
    scroll_to_callback: Option<Rc<RefCell<EntityState>>>,

    targeter: Option<Rc<RefCell<AreaTargeter>>>,
    range_indicators: RangeIndicatorHandler,
}

impl PartialEq for AreaState {
    fn eq(&self, other: &AreaState) -> bool {
        Rc::ptr_eq(&self.area.area, &other.area.area)
    }
}

fn gen_area(area: Rc<Area>, seed: Option<u128>) -> Result<(GeneratedArea, u128), Error> {
    let pregen_output = PregenOutput::new(&area, seed)?;
    let seed = match &pregen_output {
        None => 0,
        Some(out) => out.seed(),
    };

    let area = GeneratedArea::new(area, pregen_output)?;
    Ok((area, seed))
}

impl AreaState {
    pub fn new(area: Rc<Area>, seed: Option<u128>) -> Result<AreaState, Error> {
        let (gened, area_gen_seed) = gen_area(Rc::clone(&area), seed)?;

        let dim = (gened.area.width * gened.area.height) as usize;
        let entity_grid = vec![Vec::new(); dim];
        let surface_grid = vec![Vec::new(); dim];
        let transition_grid = vec![None; dim];
        let trigger_grid = vec![None; dim];
        let pc_vis = vec![false; dim];
        let pc_explored = vec![false; dim];

        let props = PropHandler::new(dim, &area);

        info!("Initializing area state for '{}'", gened.area.name);
        Ok(AreaState {
            area: gened,
            area_gen_seed,
            props,
            entities: Vec::new(),
            surfaces: Vec::new(),
            triggers: Vec::new(),
            transition_grid,
            entity_grid,
            surface_grid,
            trigger_grid,
            pc_vis,
            pc_explored,
            pc_vis_redraw: PCVisRedraw::Not,
            feedback_text: Vec::new(),
            scroll_to_callback: None,
            targeter: None,
            range_indicators: RangeIndicatorHandler::default(),
            merchants: Vec::new(),
            on_load_fired: false,
        })
    }

    pub fn load(id: &str, save: AreaSaveState) -> Result<AreaState, Error> {
        let area = match Module::area(id) {
            None => invalid_data_error(&format!("Unable to find area '{}'", id)),
            Some(area) => Ok(area),
        }?;

        let mut area_state = AreaState::new(area, Some(save.seed))?;

        area_state.on_load_fired = save.on_load_fired;

        for (index, mut buf) in save.pc_explored.into_iter().enumerate() {
            for i in 0..64 {
                if buf % 2 == 1 {
                    let pc_exp_index = i + index * 64;
                    if pc_exp_index > area_state.pc_explored.len() {
                        break;
                    }
                    area_state.pc_explored[pc_exp_index] = true;
                }
                buf /= 2;
            }
        }

        area_state.props.load(save.props)?;

        for (index, trigger_save) in save.triggers.into_iter().enumerate() {
            if index >= area_state.area.area.triggers.len() {
                return invalid_data_error("Too many triggers defined in save");
            }

            let trigger_state = TriggerState {
                enabled: trigger_save.enabled,
                fired: trigger_save.fired,
            };
            area_state.add_trigger(index, trigger_state);
        }

        area_state.add_transitions_from_area();

        for merchant_save in save.merchants {
            area_state
                .merchants
                .push(MerchantState::load(merchant_save)?);
        }

        Ok(area_state)
    }

    pub fn props(&self) -> &PropHandler {
        &self.props
    }

    pub fn props_mut(&mut self) -> &mut PropHandler {
        &mut self.props
    }

    fn pc_vis_partial_redraw(&mut self, x: i32, y: i32) {
        if let PCVisRedraw::Not = self.pc_vis_redraw {
            self.pc_vis_redraw = PCVisRedraw::Partial {
                delta_x: x,
                delta_y: y,
            }
        }
    }

    pub fn pc_vis_full_redraw(&mut self) {
        self.pc_vis_redraw = PCVisRedraw::Full;
    }

    pub fn take_pc_vis(&mut self) -> PCVisRedraw {
        let result = self.pc_vis_redraw;
        self.pc_vis_redraw = PCVisRedraw::Not;
        result
    }

    pub fn get_merchant(&self, id: &str) -> Option<&MerchantState> {
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

    pub fn get_merchant_mut(&mut self, id: &str) -> Option<&mut MerchantState> {
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
        let area = Rc::clone(&self.area.area);
        for actor_data in area.actors.iter() {
            let actor = match Module::actor(&actor_data.id) {
                None => {
                    warn!(
                        "No actor with id '{}' found when initializing area '{}'",
                        actor_data.id, area.id
                    );
                    continue;
                }
                Some(actor_data) => actor_data,
            };

            let unique_id = match actor_data.unique_id {
                None => actor_data.id.to_string(),
                Some(ref uid) => uid.to_string(),
            };

            let location = Location::from_point(actor_data.location, &area);
            debug!("Adding actor '{}' at '{:?}'", actor.id, location);
            match self.add_actor(actor, location, Some(unique_id), false, None) {
                Ok(_) => (),
                Err(e) => {
                    warn!("Error adding actor to area: {}", e);
                }
            }
        }

        self.props.populate(&self.area.props);

        for (index, trigger) in area.triggers.iter().enumerate() {
            let trigger_state = TriggerState {
                fired: false,
                enabled: trigger.initially_enabled,
            };

            self.add_trigger(index, trigger_state);
        }

        self.add_transitions_from_area();

        let mut auto_spawn = Vec::with_capacity(self.area.encounters.len());
        for encounter in &self.area.encounters {
            auto_spawn.push(encounter.encounter.auto_spawn);
        }

        for (index, spawn) in auto_spawn.into_iter().enumerate() {
            if spawn {
                self.spawn_encounter(index, true);
            }
        }
    }

    pub fn get_or_create_merchant(
        &mut self,
        id: &str,
        loot_list: &Rc<LootList>,
        buy_frac: f32,
        sell_frac: f32,
        refresh_time: Time,
    ) -> &mut MerchantState {
        let mut index = None;
        for (i, merchant) in self.merchants.iter().enumerate() {
            if merchant.id == id {
                index = Some(i);
                break;
            }
        }

        match index {
            Some(i) => {
                self.merchants[i].check_refresh();
                &mut self.merchants[i]
            }
            None => {
                info!("Creating merchant '{}'", id);
                let len = self.merchants.len();
                let merchant = MerchantState::new(id, loot_list, buy_frac, sell_frac, refresh_time);
                self.merchants.push(merchant);
                &mut self.merchants[len]
            }
        }
    }

    pub fn range_indicators(&mut self) -> &mut RangeIndicatorHandler {
        &mut self.range_indicators
    }

    pub fn range_indicator(&self) -> Option<&RangeIndicator> {
        self.range_indicators.current()
    }

    pub fn targeter(&mut self) -> Option<Rc<RefCell<AreaTargeter>>> {
        match self.targeter {
            None => None,
            Some(ref targeter) => Some(Rc::clone(targeter)),
        }
    }

    pub(crate) fn set_targeter(&mut self, mut targeter: AreaTargeter) {
        self.range_indicators.remove_targeter();
        if targeter.parent().borrow().is_party_member() {
            self.range_indicators.add(targeter.take_range_indicator());
        }
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
                    self.transition_grid[(transition.from.x
                        + x
                        + (transition.from.y + y) * self.area.width)
                        as usize] = Some(index);
                }
            }
        }
    }

    fn add_trigger(&mut self, index: usize, trigger_state: TriggerState) {
        let trigger = &self.area.area.triggers[index];
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

    pub fn fire_on_encounter_activated(&mut self, index: usize, target: &Rc<RefCell<EntityState>>) {
        info!("OnEncounterActivated for {}", index);

        let player = GameState::player();
        for trigger_index in self.area.encounters[index].triggers.iter() {
            let trigger = &self.area.area.triggers[*trigger_index];

            if *trigger_index >= self.triggers.len() {
                // this can happen when loading from a save and
                // the area data has since been updated
                continue;
            }

            if !self.triggers[*trigger_index].can_fire(&trigger) {
                continue;
            }
            self.triggers[*trigger_index].fired = true;

            if let TriggerKind::OnEncounterActivated { .. } = trigger.kind {
                info!("    Calling OnEncounterActivated");
                GameState::add_ui_callback(trigger.on_activate.clone(), &player, target);
            }
        }
    }

    pub fn fire_on_encounter_cleared(&mut self, index: usize, target: &Rc<RefCell<EntityState>>) {
        info!("OnEncounterCleared for {}", index);

        let player = GameState::player();
        for trigger_index in self.area.encounters[index].triggers.iter() {
            let trigger = &self.area.area.triggers[*trigger_index];
            self.triggers[*trigger_index].fired = true;

            if let TriggerKind::OnEncounterCleared { .. } = trigger.kind {
                info!("    Calling OnEncounterCleared");
                GameState::add_ui_callback(trigger.on_activate.clone(), &player, target);
            }
        }
    }

    pub fn spawn_encounter_at(&mut self, x: i32, y: i32) -> bool {
        let mut enc_index = None;
        for (index, data) in self.area.encounters.iter().enumerate() {
            if data.location.x != x || data.location.y != y {
                continue;
            }

            enc_index = Some(index);
            break;
        }

        if let Some(index) = enc_index {
            // this method is called by script, still spawn in debug mode
            self.spawn_encounter(index, false);
            true
        } else {
            false
        }
    }

    pub fn spawn_encounter(&mut self, enc_index: usize, respect_debug: bool) {
        let (actors, point, size, ai_group) = {
            let enc_data = &self.area.encounters[enc_index];

            let mgr = GameState::turn_manager();
            let ai_group = mgr
                .borrow_mut()
                .get_next_ai_group(&self.area.area.id, enc_index);
            if respect_debug && !Config::debug().encounter_spawning {
                return;
            }
            let encounter = &enc_data.encounter;
            (
                encounter.gen_actors(),
                enc_data.location,
                enc_data.size,
                ai_group,
            )
        };

        for (actor, unique_id) in actors {
            let location = match self.gen_location(&actor, point, size) {
                None => {
                    warn!(
                        "Unable to generate location for encounter '{}' at {},{}",
                        enc_index, point.x, point.y
                    );
                    continue;
                }
                Some(location) => location,
            };

            match self.add_actor(actor, location, unique_id, false, Some(ai_group)) {
                Ok(_) => (),
                Err(e) => {
                    warn!(
                        "Error adding actor for spawned encounter: '{}' at {},{}",
                        e, point.x, point.y
                    );
                }
            }
        }
    }

    fn gen_location(&self, actor: &Rc<Actor>, loc: Point, size: Size) -> Option<Location> {
        let available = self.get_available_locations(actor, loc, size);
        if available.is_empty() {
            return None;
        }

        let roll = gen_rand(0, available.len());

        let point = available[roll];
        let location = Location::from_point(point, &self.area.area);
        Some(location)
    }

    fn get_available_locations(&self, actor: &Rc<Actor>, loc: Point, size: Size) -> Vec<Point> {
        let mut locations = Vec::new();

        let min_x = loc.x;
        let min_y = loc.y;
        let max_x = loc.x + size.width - actor.race.size.width + 1;
        let max_y = loc.y + size.height - actor.race.size.height + 1;

        for y in min_y..max_y {
            for x in min_x..max_x {
                if !self.area.area.coords_valid(x, y) {
                    continue;
                }

                if !self.area.path_grid(&actor.race.size.id).is_passable(x, y) {
                    continue;
                }

                let mut impass = false;
                for y in y..(y + actor.race.size.height) {
                    for x in x..(x + actor.race.size.width) {
                        let index = (x + y * self.area.width) as usize;
                        if !self.entity_grid[index].is_empty() {
                            impass = true;
                            break;
                        }
                    }
                }

                if impass {
                    continue;
                }

                locations.push(Point::new(x, y));
            }
        }

        locations
    }

    pub fn is_terrain_passable(&self, size: &str, x: i32, y: i32) -> bool {
        if !self.area.area.coords_valid(x, y) {
            return false;
        }

        if !self.area.path_grid(size).is_passable(x, y) {
            return false;
        }

        true
    }

    pub fn is_passable_size(&self, size: &ObjectSize, x: i32, y: i32) -> bool {
        if !self.is_terrain_passable(&size.id, x, y) {
            return false;
        }

        size.points(x, y)
            .all(|p| self.point_size_passable(p.x, p.y))
    }

    pub fn is_passable(
        &self,
        requester: &EntityState,
        entities_to_ignore: &[usize],
        new_x: i32,
        new_y: i32,
    ) -> bool {
        if !self.is_terrain_passable(&requester.size(), new_x, new_y) {
            return false;
        }

        requester
            .points(new_x, new_y)
            .all(|p| self.point_entities_passable(entities_to_ignore, p.x, p.y))
    }

    pub fn is_passable_for_entity(&self, requester: &EntityState, x: i32, y: i32) -> bool {
        self.is_passable(
            &requester,
            &[requester.index()],
            x,
            y,
        )
    }

    pub fn get_entity_at(&self, x: i32, y: i32) -> Option<Rc<RefCell<EntityState>>> {
        if !self.area.area.coords_valid(x, y) {
            return None;
        }

        let index = {
            let vec = &self.entity_grid[(x + y * self.area.width) as usize];
            if vec.is_empty() {
                return None;
            }
            vec[0]
        };

        let mgr = GameState::turn_manager();
        let mgr = mgr.borrow();
        mgr.entity_checked(index)
    }

    pub fn get_transition_at(&self, x: i32, y: i32) -> Option<&Transition> {
        if !self.area.area.coords_valid(x, y) {
            return None;
        }

        let index = match self.transition_grid[(x + y * self.area.width) as usize] {
            None => return None,
            Some(index) => index,
        };

        self.area.transitions.get(index)
    }

    pub fn toggle_prop_active(&mut self, index: usize) {
        if !self.props.toggle_active(index) {
            return;
        }

        self.pc_vis_partial_redraw(0, 0);
        for member in GameState::party().iter() {
            self.compute_pc_visibility(member, 0, 0);
        }
        self.update_view_visibility();
    }

    pub fn has_visibility(&self, parent: &EntityState, target: &EntityState) -> bool {
        has_visibility(&self.area, &self.props.entire_vis_grid(), parent, target)
    }

    pub fn compute_pc_visibility(
        &mut self,
        entity: &Rc<RefCell<EntityState>>,
        delta_x: i32,
        delta_y: i32,
    ) {
        let start_time = time::Instant::now();

        let props_vis = calculate_los(
            &mut self.pc_explored,
            &self.area,
            self.props.entire_vis_grid(),
            self.props.grid(),
            &mut entity.borrow_mut(),
            delta_x,
            delta_y,
        );

        // set explored to true for any partially visible props
        for prop_index in props_vis {
            let prop = self.props.get(prop_index);
            for point in prop.location_points() {
                let index = (point.x + point.y * self.area.width) as usize;
                self.pc_explored[index] = true;
            }
        }

        trace!(
            "Visibility compute time: {}",
            util::format_elapsed_secs(start_time.elapsed())
        );
    }

    pub fn update_view_visibility(&mut self) {
        unsafe { std::ptr::write_bytes(self.pc_vis.as_mut_ptr(), 0, self.pc_vis.len()) }

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

    pub fn set_trigger_enabled_at(&mut self, x: i32, y: i32, enabled: bool) -> bool {
        if !self.area.area.coords_valid(x, y) {
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

        if !self.triggers[index].can_fire(&self.area.area.triggers[index]) {
            return;
        }

        self.triggers[index].fired = true;
        GameState::add_ui_callback(
            self.area.area.triggers[index].on_activate.clone(),
            entity,
            entity,
        );
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
        if !self.area.area.coords_valid(x, y) {
            return false;
        }

        let index = (x + y * self.area.width) as usize;
        if !self.props.pass_grid(index) {
            return false;
        }

        let grid_index = &self.entity_grid[index];

        grid_index.is_empty()
    }

    fn point_entities_passable(&self, entities_to_ignore: &[usize], x: i32, y: i32) -> bool {
        if !self.area.area.coords_valid(x, y) {
            return false;
        }

        let index = (x + y * self.area.width) as usize;
        if !self.props.pass_grid(index) {
            return false;
        }

        let grid = &self.entity_grid[index];

        for index in grid.iter() {
            if !entities_to_ignore.contains(index) {
                return false;
            }
        }
        true
    }

    pub(crate) fn add_actor(
        &mut self,
        actor: Rc<Actor>,
        location: Location,
        unique_id: Option<String>,
        is_pc: bool,
        ai_group: Option<usize>,
    ) -> Result<usize, Error> {
        let entity = Rc::new(RefCell::new(EntityState::new(
            actor,
            unique_id,
            location.clone(),
            is_pc,
            ai_group,
        )));
        match self.add_entity(&entity, location) {
            Ok(index) => Ok(index),
            Err(e) => {
                warn!("Unable to add entity to area");
                warn!("{}", e);
                Err(e)
            }
        }
    }

    pub(crate) fn entities_with_points(&self, points: &[Point]) -> Vec<usize> {
        let mut result = HashSet::new();
        for p in points {
            if !self.area.area.coords_valid(p.x, p.y) {
                continue;
            }
            for entity in self.entity_grid[(p.x + p.y * self.area.width) as usize].iter() {
                result.insert(*entity);
            }
        }

        result.into_iter().collect()
    }

    #[must_use]
    pub(crate) fn remove_surface(&mut self, index: usize, points: &[Point]) -> HashSet<usize> {
        debug!("Removing surface {} from area", index);

        let mut entities = HashSet::new();
        for p in points {
            if !self.area.area.coords_valid(p.x, p.y) {
                continue;
            }
            self.surface_grid[(p.x + p.y * self.area.width) as usize].retain(|i| *i != index);
            for entity in self.entity_grid[(p.x + p.y * self.area.width) as usize].iter() {
                entities.insert(*entity);
            }
        }

        self.surfaces.retain(|i| *i != index);

        entities
    }

    #[must_use]
    pub(crate) fn add_surface(&mut self, index: usize, points: &[Point]) -> HashSet<usize> {
        self.surfaces.push(index);

        let mut entities = HashSet::new();
        for p in points {
            if !self.area.area.coords_valid(p.x, p.y) {
                continue;
            }

            self.surface_grid[(p.x + p.y * self.area.width) as usize].push(index);

            for entity in self.entity_grid[(p.x + p.y * self.area.width) as usize].iter() {
                entities.insert(*entity);
            }
        }

        entities
    }

    pub(crate) fn load_entity(
        &mut self,
        entity: &Rc<RefCell<EntityState>>,
        location: Location,
        is_dead: bool,
    ) -> Result<usize, Error> {
        let mgr = GameState::turn_manager();
        let index = mgr.borrow_mut().add_entity(&entity, is_dead);

        if is_dead {
            Ok(index)
        } else {
            self.transition_entity_to(&entity, index, location)
        }
    }

    pub(crate) fn add_entity(
        &mut self,
        entity: &Rc<RefCell<EntityState>>,
        location: Location,
    ) -> Result<usize, Error> {
        let result = self.load_entity(entity, location, false);
        entity.borrow_mut().actor.init_day();
        result
    }

    fn compute_threatened(
        &self,
        mover: &Rc<RefCell<EntityState>>,
        mgr: &TurnManager,
        removal: bool,
    ) {
        let mut mover = mover.borrow_mut();
        let mover_index = mover.index();
        for index in self.entities.iter() {
            let index = *index;
            if index == mover_index {
                continue;
            }

            let entity = mgr.entity(index);
            let mut entity = entity.borrow_mut();

            self.check_threatened(&mut mover, &mut entity, removal);
            self.check_threatened(&mut entity, &mut mover, removal);
        }
    }

    fn check_threatened(&self, att: &mut EntityState, def: &mut EntityState, removal: bool) {
        if removal || !is_threat(att, def) {
            att.actor.remove_threatening(def.index());
            def.actor.remove_threatener(att.index());
        } else {
            att.actor.add_threatening(def.index());
            def.actor.add_threatener(att.index());
        }
    }

    pub(crate) fn transition_entity_to(
        &mut self,
        entity: &Rc<RefCell<EntityState>>,
        index: usize,
        location: Location,
    ) -> Result<usize, Error> {
        let x = location.x;
        let y = location.y;

        if !self.area.area.coords_valid(x, y) {
            return invalid_data_error(&format!("entity location is out of bounds: {},{}", x, y));
        }

        let entities_to_ignore = vec![entity.borrow().index()];
        if !self.is_passable(&entity.borrow(), &entities_to_ignore, x, y) {
            info!(
                "Entity location in '{}' is not passable: {},{} for '{}'",
                &self.area.area.id,
                x,
                y,
                &entity.borrow().actor.actor.id
            );
        }

        entity.borrow_mut().actor.compute_stats();

        entity.borrow_mut().location = location;
        self.entities.push(index);

        let mgr = GameState::turn_manager();

        self.compute_threatened(entity, &mgr.borrow(), false);

        let surfaces = self.add_entity_points(&entity.borrow());
        for surface in surfaces {
            let index = entity.borrow().index();
            mgr.borrow_mut().add_to_surface(index, surface);
        }

        if entity.borrow().is_party_member() {
            self.compute_pc_visibility(&entity, 0, 0);
        }

        Ok(index)
    }

    pub fn move_entity(
        &mut self,
        entity: &Rc<RefCell<EntityState>>,
        x: i32,
        y: i32,
        squares: u32,
    ) -> bool {
        let old_x = entity.borrow().location.x;
        let old_y = entity.borrow().location.y;
        if !entity.borrow_mut().move_to(x, y, squares) {
            return false;
        }

        let mgr = GameState::turn_manager();

        self.update_entity_position(entity, old_x, old_y, &mut mgr.borrow_mut());

        true
    }

    pub(crate) fn update_entity_position(
        &mut self,
        entity: &Rc<RefCell<EntityState>>,
        old_x: i32,
        old_y: i32,
        mgr: &mut TurnManager,
    ) {
        let d_x = old_x - entity.borrow().location.x;
        let d_y = old_y - entity.borrow().location.y;

        let entity_index = entity.borrow().index();

        let aura_indices = mgr.auras_for(entity_index);
        for aura_index in aura_indices {
            let aura = mgr.effect_mut(aura_index);
            let surface = match aura.surface {
                None => continue,
                Some(ref mut surface) => surface,
            };
            let old_entities = self.remove_surface(aura_index, &surface.points);
            for p in surface.points.iter_mut() {
                p.x -= d_x;
                p.y -= d_y;
            }
            let new_entities = self.add_surface(aura_index, &surface.points);

            debug!("Update aura: {}: {}", aura_index, aura.name);

            for entity in old_entities.difference(&new_entities) {
                // remove from entities in old but not in new
                mgr.remove_from_surface(*entity, aura_index);
            }

            for entity in new_entities.difference(&old_entities) {
                // add to entities in new but not old
                mgr.add_to_surface(*entity, aura_index);
            }
        }

        let old_surfaces = self.clear_entity_points(&entity.borrow(), old_x, old_y);
        let new_surfaces = self.add_entity_points(&entity.borrow());

        self.compute_threatened(entity, mgr, false);
        // remove from surfaces in old but not in new
        for surface in old_surfaces.difference(&new_surfaces) {
            mgr.remove_from_surface(entity_index, *surface);
        }

        // add to surfaces in new but not in old
        for surface in new_surfaces.difference(&old_surfaces) {
            mgr.add_to_surface(entity_index, *surface);
        }

        for surface in new_surfaces.intersection(&old_surfaces) {
            mgr.increment_surface_squares_moved(entity_index, *surface);
        }

        let is_pc = entity.borrow().is_party_member();

        if is_pc {
            self.pc_vis_partial_redraw(d_x, d_y);
            self.compute_pc_visibility(&entity, d_x, d_y);
            self.update_view_visibility();

            self.check_trigger_grid(&entity);
        }

        mgr.fire_on_moved_next_update(entity_index);
        mgr.check_ai_activation(entity, self);
    }

    #[must_use]
    fn add_entity_points(&mut self, entity: &EntityState) -> HashSet<usize> {
        let mut surfaces = HashSet::new();
        for p in entity.location_points() {
            self.add_entity_to_grid(p.x, p.y, entity.index());
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
            self.remove_entity_from_grid(p.x, p.y, entity.index());
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

    pub(crate) fn update(&mut self) {
        self.props.update();

        self.feedback_text.iter_mut().for_each(|f| f.update());
        self.feedback_text.retain(|f| f.retain());

        let remove_targeter = match self.targeter {
            None => false,
            Some(ref targeter) => targeter.borrow().cancel(),
        };

        if remove_targeter {
            self.targeter.take();
            self.range_indicators.remove_targeter();
        }
    }

    #[must_use]
    pub fn remove_entity(
        &mut self,
        entity: &Rc<RefCell<EntityState>>,
        mgr: &TurnManager,
    ) -> HashSet<usize> {
        let (index, surfaces) = {
            let entity = entity.borrow();
            let index = entity.index();
            trace!(
                "Removing entity '{}' with index '{}'",
                entity.actor.actor.name,
                index
            );
            let x = entity.location.x;
            let y = entity.location.y;
            (index, self.clear_entity_points(&entity, x, y))
        };

        self.entities.retain(|i| *i != index);

        self.compute_threatened(entity, mgr, true);

        surfaces
    }

    pub fn add_feedback_text(&mut self, text: AreaFeedbackText) {
        if text.is_empty() {
            return;
        }

        self.feedback_text.push(text);
    }

    pub fn feedback_text_iter(&self) -> impl Iterator<Item = &AreaFeedbackText> {
        self.feedback_text.iter()
    }

    pub fn feedback_text_iter_mut(&mut self) -> impl Iterator<Item = &mut AreaFeedbackText> {
        self.feedback_text.iter_mut()
    }

    pub fn entity_iter(&self) -> impl Iterator<Item = &usize> {
        self.entities.iter()
    }
}
