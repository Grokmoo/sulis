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

use std::{f32};

use crate::{AreaState, EntityState};
use sulis_core::util::{Point};
use sulis_module::{area::{LocationChecker, PathFinderGrid, PathFinder}};

pub struct StateLocationChecker<'a, 'b> {
    width: i32,
    grid: &'a PathFinderGrid,
    prop_grid: &'a [bool],
    entity_grid: &'a [Vec<usize>],
    requester: &'b EntityState,
    entities_to_ignore: Vec<usize>,
}

impl<'a, 'b> StateLocationChecker<'a, 'b> {
    pub fn new(area_state: &'a AreaState, requester: &'b EntityState,
               mut entities_to_ignore: Vec<usize>) -> StateLocationChecker<'a, 'b> {

        let width = area_state.area.width;
        let grid = &area_state.area.path_grid(&requester.size());
        let prop_grid = &area_state.prop_pass_grid;
        let entity_grid = &area_state.entity_grid;
        entities_to_ignore.push(requester.index());

        StateLocationChecker {
            width,
            grid,
            prop_grid,
            entity_grid,
            requester,
            entities_to_ignore,
        }
    }
}


impl<'a, 'b> LocationChecker for StateLocationChecker<'a, 'b> {
    fn goal(&self, x: f32, y: f32) -> (f32, f32) {
        (x - (self.requester.size.width / 2) as f32,
         y - (self.requester.size.height / 2) as f32)
    }

    fn passable(&self, x: i32, y: i32) -> bool {
        if !self.grid.is_passable(x, y) { return false; }

        self.requester.points(x, y).all(|p| {
            let index = (p.x + p.y * self.width) as usize;

            if !self.prop_grid[index] { return false; }

            for i in self.entity_grid[index].iter() {
                if !self.entities_to_ignore.contains(i) { return false; }
            }

            true
        })
    }
}


pub fn find_path(path_finder: &mut PathFinder,
                 area_state: &AreaState,
                 entity: &EntityState,
                 entities_to_ignore: Vec<usize>,
                 dest_x: f32,
                 dest_y: f32,
                 dest_dist: f32) -> Option<Vec<Point>> {
    let checker = StateLocationChecker::new(area_state, entity, entities_to_ignore);

    path_finder.find(&checker, entity.location.x, entity.location.y,
                     dest_x, dest_y, dest_dist)
}

