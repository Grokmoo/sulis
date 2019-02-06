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

use std::cell::Ref;
use std::collections::BTreeMap;
use std::time;
use std::{f32, ptr};

use hashbrown::{HashMap, HashSet};

use crate::{AreaState, EntityState};
use sulis_core::util::{self, Point};
use sulis_module::Area;

const MAX_ITERATIONS: i32 = 400;

pub struct PathFinder {
    pub width: i32,
    pub height: i32,

    f_score: Vec<i32>,
    g_score: Vec<i32>,
    open: BTreeMap<i32, i32>,
    closed: HashSet<i32>,
    came_from: HashMap<i32, i32>,

    goal_x: f32,
    goal_y: f32,
}

impl PathFinder {
    pub fn new(area: &Area) -> PathFinder {
        let width = area.width;
        let height = area.height;

        debug!("Initializing pathfinder for {}", area.id);
        PathFinder {
            width,
            height,
            f_score: vec![0; (width * height) as usize],
            g_score: vec![0; (width * height) as usize],
            open: BTreeMap::new(),
            closed: HashSet::default(),
            came_from: HashMap::default(),
            goal_x: 0.0,
            goal_y: 0.0,
        }
    }

    /// Finds a path within the given `AreaState`, from the position of `requester`
    /// to the specified destination.  `dest_dist` allows points within that distance
    /// of the destination to also be allowable goals.
    ///
    /// Returns a vec of `Point`s; which is the path that requester should take
    /// in order to reach within `dest_dist` of the destination in the most
    /// efficient manner.  Returns `None` if no path exists to reach the destination.
    /// Will return a vec of length zero if the dest is already reached by the
    /// requester.
    /// If reconstruct is set to false, does not produce a path.  Instead, only
    /// checks if a path exists, returning Some if it does, None if not
    pub fn find(
        &mut self,
        area_state: &AreaState,
        requester: Ref<EntityState>,
        mut entities_to_ignore: Vec<usize>,
        reconstruct: bool,
        dest_x: f32,
        dest_y: f32,
        dest_dist: f32,
    ) -> Option<Vec<Point>> {
        if dest_x < 0.0 || dest_y < 0.0 {
            return None;
        }
        if dest_x >= self.width as f32 || dest_y >= self.height as f32 {
            return None;
        }

        let grid = area_state.area.get_path_grid(&requester.size());
        let prop_grid = &area_state.prop_pass_grid;
        let entity_grid = &area_state.entity_grid;

        trace!(
            "Finding path from {:?} to within {} of {},{}",
            requester.location,
            dest_dist,
            dest_x,
            dest_y
        );

        entities_to_ignore.push(requester.index());

        // let start_time = time::Instant::now();
        self.goal_x = dest_x - (requester.size.width / 2) as f32;
        self.goal_y = dest_y - (requester.size.height / 2) as f32;;
        let dest_dist_squared = (dest_dist * dest_dist) as i32;
        let start = requester.location.x + requester.location.y * self.width;

        // the set of discovered nodes that are not evaluated yet
        self.open.clear();
        // self.open.insert(start);

        // the set of nodes that have already been evaluated
        self.closed.clear();

        // for each node, the node it can be most efficiently reached from
        self.came_from.clear();

        // let f_g_init_time = time::Instant::now();
        unsafe {
            // memset g_score and f_score to a large integer value
            // benchmarking revealed that setting these values using the naive
            // approach is the majority of time spent for most path finds
            ptr::write_bytes(self.g_score.as_mut_ptr(), 127, self.g_score.len());
            ptr::write_bytes(self.f_score.as_mut_ptr(), 127, self.f_score.len());
        }

        self.g_score[start as usize] = 0;
        self.f_score[start as usize] = self.dist_squared(start);
        // info!("F and G score init: {}", util::format_elapsed_secs(f_g_init_time.elapsed()));

        self.open.insert(self.f_score[start as usize] as i32, start);
        // info!("Spent {} secs in path find init", util::format_elapsed_secs(start_time.elapsed()));

        let loop_start_time = time::Instant::now();

        let mut iterations = 0;
        while iterations < MAX_ITERATIONS && !self.open.is_empty() {
            let (current_f_score, current) = self.find_lowest_f_score_in_open_set();
            if self.is_goal(current, dest_dist_squared) {
                trace!(
                    "Path loop time: {}",
                    util::format_elapsed_secs(loop_start_time.elapsed())
                );
                let path = self.reconstruct_path(current);
                if path.len() == 1
                    && path[0].x == requester.location.x
                    && path[0].y == requester.location.y
                {
                    return None;
                } else if reconstruct {
                    return Some(self.reconstruct_path(current));
                } else {
                    return Some(Vec::new());
                }
            }

            self.open.remove(&current_f_score);
            self.closed.insert(current);

            let neighbors = self.get_neighbors(current);
            for i in 0..4 {
                let neighbor = neighbors[i];
                if neighbor == -1 {
                    continue;
                }
                //trace!("Checking neighbor {}", neighbor);
                if self.closed.contains(&neighbor) {
                    //trace!("Already evaluated.");
                    continue; // neighbor has already been evaluated
                }

                // we compute the passability of each point as needed here
                let neighbor_x = neighbor % self.width;
                let neighbor_y = neighbor / self.width;

                // this should be equivalent to checking is_passable on the area_state
                // but significantly faster here, removing unneccesary checks
                if !grid.is_passable(neighbor_x, neighbor_y)
                    || !requester.points(neighbor_x, neighbor_y).all(|p| {
                        let index = (p.x + p.y * self.width) as usize;
                        if !prop_grid[index] {
                            return false;
                        }
                        for i in entity_grid[index].iter() {
                            if !entities_to_ignore.contains(i) {
                                return false;
                            }
                        }
                        true
                    })
                {
                    self.closed.insert(neighbor);
                    //trace!("Not passable");
                    continue;
                }

                let tentative_g_score =
                    self.g_score[current as usize] + self.get_cost(current, neighbor);
                if tentative_g_score >= self.g_score[neighbor as usize] {
                    self.open.insert(self.f_score[neighbor as usize], neighbor);
                    //trace!("G score indicates this neighbor is not preferable.");
                    continue; // this is not a better path
                }

                if reconstruct {
                    self.came_from.insert(neighbor, current);
                }

                self.g_score[neighbor as usize] = tentative_g_score;
                self.f_score[neighbor as usize] = tentative_g_score + self.dist_squared(neighbor);
                self.open.insert(self.f_score[neighbor as usize], neighbor);
            }

            iterations += 1;
        }

        None
    }

    #[inline]
    fn is_goal(&self, current: i32, dest_dist_squared: i32) -> bool {
        self.dist_squared(current) <= dest_dist_squared
    }

    #[inline]
    fn reconstruct_path(&self, current: i32) -> Vec<Point> {
        trace!("Reconstructing path");

        // let reconstruct_time = time::Instant::now();
        let mut path: Vec<Point> = Vec::new();

        path.push(self.get_point(current));
        let mut current = current;
        loop {
            //trace!("Current {}", current);
            current = match self.came_from.get(&current) {
                None => break,
                Some(point) => *point,
            };
            path.push(self.get_point(current));
        }

        path.reverse();
        trace!("Found path: {:?}", path);
        // info!("Reconstruct path time: {}", util::format_elapsed_secs(reconstruct_time.elapsed()));
        path
    }

    #[inline]
    fn get_point(&self, index: i32) -> Point {
        Point::new(index % self.width, index / self.width)
    }

    #[inline]
    fn get_cost(&self, _from: i32, _to: i32) -> i32 {
        1
    }

    #[inline]
    // using an array here instead of a vec is much faster
    fn get_neighbors(&self, point: i32) -> [i32; 4] {
        let width = self.width;
        let height = self.height;

        let top = point - width;
        let right = point + 1;
        let left = point - 1;
        let bottom = point + width;

        let mut neighbors = [-1; 4];
        if top > 0 {
            neighbors[0] = top;
        }
        if bottom < width * height {
            neighbors[1] = bottom;
        }
        if right % width != point % width {
            neighbors[2] = right;
        }
        if left % width != point % width {
            neighbors[3] = left;
        }

        //trace!("Got neighbors for {}: {:?}", point, neighbors);
        neighbors
    }

    #[inline]
    fn find_lowest_f_score_in_open_set(&self) -> (i32, i32) {
        let (f_score, index) = self.open.iter().next().unwrap();
        (*f_score, *index)
    }

    #[inline]
    fn dist_squared(&self, start: i32) -> i32 {
        let s_x = start % self.width;
        let s_y = start / self.width;

        let x_part = s_x as f32 - self.goal_x;
        let y_part = s_y as f32 - self.goal_y;

        (x_part * x_part + y_part * y_part) as i32
    }
}
