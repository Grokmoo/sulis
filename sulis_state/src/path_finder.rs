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

use std::collections::HashSet;
use std::cell::RefCell;
use std::rc::Rc;

use crate::{animation, animation::Anim, script::ScriptCallback, AreaState, EntityState};
use sulis_core::{
    config::Config,
    util::{self, Point},
};
use sulis_module::area::{Destination, LocationChecker, PathFinder, PathFinderGrid};
use sulis_module::Faction;
use crate::GameState;

pub struct StateLocationChecker<'a, 'b> {
    width: i32,
    grid: &'a PathFinderGrid,
    prop_grid: &'a [bool],
    entity_grid: &'a [Vec<usize>],
    requester: &'b EntityState,
    entities_to_ignore: HashSet<usize>,
    non_endpoints: HashSet<usize>,
}

impl<'a, 'b> StateLocationChecker<'a, 'b> {
    pub fn new(
        area_state: &'a AreaState,
        requester: &'b EntityState,
        mut entities_to_ignore: HashSet<usize>,
        non_endpoints: HashSet<usize>,
    ) -> StateLocationChecker<'a, 'b> {
        let width = area_state.area.width;
        let grid = &area_state.area.path_grid(&requester.size());
        let prop_grid = area_state.props().entire_pass_grid();
        let entity_grid = &area_state.entity_grid;
        entities_to_ignore.insert(requester.index());

        StateLocationChecker {
            width,
            grid,
            prop_grid,
            entity_grid,
            requester,
            entities_to_ignore,
            non_endpoints,
        }
    }
}

impl<'a, 'b> LocationChecker for StateLocationChecker<'a, 'b> {
    fn passable(&self, x: i32, y: i32) -> bool {
        if !self.grid.is_passable(x, y) {
            return false;
        }

        self.requester.points(x, y).all(|p| {
            let index = (p.x + p.y * self.width) as usize;

            if !self.prop_grid[index] {
                return false;
            }

            for i in self.entity_grid[index].iter() {
                if !self.entities_to_ignore.contains(i) && !self.non_endpoints.contains(i) {
                    return false;
                }
            }

            true
        })
    }
    fn is_invalid_endpoint(&self, x: i32, y:i32) -> bool {

        let index = (x + y * self.width) as usize;
        self.non_endpoints.contains(&index)
    }
}

pub fn move_towards_point(
    finder: &mut PathFinder,
    area: &AreaState,
    entity: &Rc<RefCell<EntityState>>,
    entities_to_ignore: Vec<usize>,
    dest: Destination,
    cb: Option<Box<dyn ScriptCallback>>,
) -> Option<Anim> {
    let path = match find_path(
        finder,
        area,
        &entity.borrow(),
        entities_to_ignore,
        dest,
        true,
    ) {
        None => return None,
        Some(path) => path,
    };

    let mut anim =
        animation::move_animation::new(entity, path, Config::animation_base_time_millis());
    if let Some(cb) = cb {
        anim.add_completion_callback(cb);
    }

    Some(anim)
}

pub fn can_move_towards_point(
    finder: &mut PathFinder,
    area: &AreaState,
    entity: &EntityState,
    entities_to_ignore: Vec<usize>,
    dest: Destination,
) -> Option<Vec<Point>> {
    find_path(finder, area, entity, entities_to_ignore, dest, true)
}

pub fn can_move_ignore_ap(
    finder: &mut PathFinder,
    area: &AreaState,
    entity: &EntityState,
    entities_to_ignore: Vec<usize>,
    dest: Destination,
) -> Option<Vec<Point>> {
    find_path(finder, area, entity, entities_to_ignore, dest, false)
}

fn find_path(
    path_finder: &mut PathFinder,
    area_state: &AreaState,
    entity: &EntityState,
    entities_to_ignore: Vec<usize>,
    dest: Destination,
    check_ap: bool,
) -> Option<Vec<Point>> {

    let entities_to_ignore: HashSet<usize> = entities_to_ignore.iter().cloned().collect();
    let mut non_endpoints = HashSet::new();

    if entity.actor.faction() != Faction::Hostile {
        for member in GameState::party().iter() {
            non_endpoints.insert(member.borrow().index());
        }
    }

    let checker = StateLocationChecker::new(area_state, entity, entities_to_ignore, non_endpoints);

    debug!(
        "Attempting move '{}' to {:?}",
        entity.actor.actor.name, dest
    );

    if check_ap {
        if entity.actor.stats.move_disabled || entity.actor.ap() < entity.actor.get_move_ap_cost(1)
        {
            return None;
        }

        trace!("  Entity is able to move");
    }

    let start_time = std::time::Instant::now();

    let path = path_finder.find(&checker, entity.location.x, entity.location.y, dest);

    debug!(
        "Pathing complete in {} secs",
        util::format_elapsed_secs(start_time.elapsed())
    );
    path
}
