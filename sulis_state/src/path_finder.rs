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
use std::collections::{HashMap, HashSet};
use std::{f32, ptr};

use sulis_core::util::Point;
use sulis_module::Area;
use {EntityState, AreaState};

const MAX_ITERATIONS: i32 = 200;

pub struct PathFinder {
    pub width: i32,
    pub height: i32,

    f_score: Vec<f32>,
    g_score: Vec<f32>,
    open: HashSet<i32>,
    closed: HashSet<i32>,
    came_from: HashMap<i32, i32>,

    goal_x: f32,
    goal_y: f32,
    requester_center_x: f32,
    requester_center_y: f32,
}

impl PathFinder {
    pub fn new(area: &Area) -> PathFinder {
        let width = area.width;
        let height = area.height;

        debug!("Initializing pathfinder for {}", area.id);
        PathFinder {
            width,
            height,
            f_score: vec![0.0;(width*height) as usize],
            g_score: vec![0.0;(width*height) as usize],
            open: HashSet::new(),
            closed: HashSet::new(),
            came_from: HashMap::new(),
            goal_x: 0.0,
            goal_y: 0.0,
            requester_center_x: 0.0,
            requester_center_y: 0.0,
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
    pub fn find(&mut self, area_state: &AreaState, requester: Ref<EntityState>,
                dest_x: f32, dest_y: f32, dest_dist: f32) -> Option<Vec<Point>> {
        if dest_x < 0.0 || dest_y < 0.0 { return None; }
        if dest_x >= self.width as f32 || dest_y >= self.height as f32 { return None; }

        debug!("Finding path from {:?} to within {} of {},{}",
               requester.location, dest_dist, dest_x, dest_y);

        // let start_time = time::Instant::now();
        self.goal_x = dest_x;
        self.goal_y = dest_y;
        self.requester_center_x = requester.size.size as f32 / 2.0 - 0.5;
        self.requester_center_y = requester.size.size as f32 / 2.0 - 0.5;
        let dest_dist_squared = dest_dist * dest_dist;
        let start = requester.location.x + requester.location.y * self.width;

        // the set of discovered nodes that are not evaluated yet
        self.open.clear();
        self.open.insert(start);

        // the set of nodes that have already been evaluated
        self.closed.clear();

        // for each node, the node it can be most efficiently reached from
        self.came_from.clear();

        // let f_g_init_time = time::Instant::now();
        unsafe {
            // memset g_score and f_score to a large floating point number
            // benchmarking revealed that setting these values using the naive
            // approach below is the majority of time spent for most
            // path finds
            ptr::write_bytes(self.g_score.as_mut_ptr(), 127, self.g_score.len() - 1);
            ptr::write_bytes(self.f_score.as_mut_ptr(), 127, self.f_score.len() - 1);
        }

        // for each node, cost of getting from start to that node
        // self.g_score.iter_mut().for_each(|v| *v = f32::INFINITY);
        // for each node, total cost of getting from start to goal passing by this node
        // self.f_score.iter_mut().for_each(|v| *v = f32::INFINITY);

        self.g_score[start as usize] = 0.0;
        self.f_score[start as usize] = self.dist_squared(start);
        // info!("F and G score init: {}", animation::format_elapsed_secs(f_g_init_time.elapsed()));

        // info!("Spent {} secs in path find init", animation::format_elapsed_secs(start_time.elapsed()));

        // let loop_start_time = time::Instant::now();

        let mut iterations = 0;
        while iterations < MAX_ITERATIONS && !self.open.is_empty() {
            let current = self.find_lowest_f_score_in_open_set();
            if self.is_goal(current, dest_dist_squared) {
                // info!("Path find loop time: {}", animation::format_elapsed_secs(loop_start_time.elapsed()));
                return Some(self.reconstruct_path(current));
            }

            self.open.remove(&current);
            self.closed.insert(current);

            for neighbor in self.get_neighbors(current) {
                //trace!("Checking neighbor {}", neighbor);
                if self.closed.contains(&neighbor) {
                    //trace!("Already evaluated.");
                    continue; // neighbor has already been evaluated
                }

                // we compute the passability of each point as needed here
                let neighbor_x = neighbor % self.width;
                let neighbor_y = neighbor / self.width;
                if !area_state.is_passable(&requester, neighbor_x, neighbor_y) {
                    self.closed.insert(neighbor);
                    //trace!("Not passable");
                    continue;
                }

                if !self.open.contains(&neighbor) {
                    self.open.insert(neighbor);
                }

                let tentative_g_score = self.g_score[current as usize] +
                    self.get_cost(current, neighbor);
                if tentative_g_score >= self.g_score[neighbor as usize] {
                    //trace!("G score indicates this neighbor is not preferable.");
                    continue; // this is not a better path
                }

                self.came_from.insert(neighbor, current);
                self.g_score[neighbor as usize] = tentative_g_score;
                self.f_score[neighbor as usize] = tentative_g_score + self.dist_squared(neighbor);
            }

            iterations += 1;
        }

        None
    }

    fn is_goal(&self, current: i32, dest_dist_squared: f32) -> bool {
        self.dist_squared(current) <= dest_dist_squared
    }

    fn reconstruct_path(&self, current: i32) -> Vec<Point> {
        trace!("Reconstructing path");

        // let path_reconstruct_time = time::Instant::now();
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

        // remove the last point which is the entity start pos
        path.pop();
        path.reverse();
        debug!("Found path: {:?}", path);
        // info!("Reconstruct path time: {}", animation::format_elapsed_secs(path_reconstruct_time.elapsed()));
        path
    }

    fn get_point(&self, index: i32) -> Point {
        Point::new(index % self.width, index / self.width)
    }

    fn get_cost(&self, _from: i32, _to: i32) -> f32 {
        1.0
    }

    fn get_neighbors(&self, point: i32) -> Vec<i32> {
        let width = self.width;
        let height = self.height;

        let top = point - width;
        let right = point + 1;
        let left = point - 1;
        let bottom = point + width;

        let mut neighbors: Vec<i32> = Vec::new();
        if top > 0 { neighbors.push(top); }
        if bottom < width * height { neighbors.push(bottom); }
        if right % width != point % width { neighbors.push(right); }
        if left % width != point % width { neighbors.push(left); }

        //trace!("Got neighbors for {}: {:?}", point, neighbors);
        neighbors
    }

    fn find_lowest_f_score_in_open_set(&self) -> i32 {
        let mut lowest = f32::INFINITY;
        let mut lowest_index = 0;

        for val in self.open.iter() {
            let f_score = &self.f_score[*val as usize];
            if f_score < &lowest {
                lowest = *f_score;
                lowest_index = *val;
            }
        }

        //trace!("Found lowest f score of {} at {}", lowest, lowest_index);
        lowest_index
    }

    fn dist_squared(&self, start: i32) -> f32 {
        let s_x = start % self.width;
        let s_y = start / self.width;

        let x_part = s_x as f32 + self.requester_center_x - self.goal_x;
        let y_part = s_y as f32 + self.requester_center_y - self.goal_y;

        x_part * x_part + y_part * y_part
    }
}
