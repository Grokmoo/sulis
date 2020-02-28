//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::time;
use std::{f32, ptr};

use crate::MOVE_TO_THRESHOLD;
use sulis_core::util::{self, Point};

const MAX_ITERATIONS: i32 = 2_000;

#[derive(Debug, Clone, Copy)]
pub struct Destination {
    pub parent_w: f32,
    pub parent_h: f32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub dist: f32,
    pub max_path_len: Option<u32>,
}

impl Destination {
    pub fn with_defaults(x: f32, y: f32) -> Destination {
        Destination {
            x,
            y,
            w: 0.0,
            h: 0.0,
            parent_w: 0.0,
            parent_h: 0.0,
            dist: MOVE_TO_THRESHOLD,
            max_path_len: None,
        }
    }
}

#[derive(Eq)]
struct OpenEntry {
    f_score: i32,
    index: i32,
}

impl OpenEntry {
    pub fn new(index: i32, f_score: i32) -> OpenEntry {
        OpenEntry { index, f_score }
    }
}

impl Ord for OpenEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // min ordering
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for OpenEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for OpenEntry {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score
    }
}

pub trait LocationChecker {
    fn passable(&self, x: i32, y: i32) -> bool;
    fn in_friend_space(&self, _current: i32) -> bool {false}
}

pub struct PathFinder {
    pub width: i32,
    pub height: i32,

    f_score: Vec<i32>,
    g_score: Vec<i32>,
    open: BinaryHeap<OpenEntry>,
    open_set: HashSet<i32>,
    closed: HashSet<i32>,
    came_from: HashMap<i32, i32>,

    dest_x: f32,
    dest_y: f32,
    dest_w_over2: f32,
    dest_h_over2: f32,
    parent_w_over2: f32,
    parent_h_over2: f32,

    max_iterations: i32,
}

impl PathFinder {
    pub fn new(width: i32, height: i32) -> PathFinder {
        PathFinder {
            width,
            height,
            f_score: vec![0; (width * height) as usize],
            g_score: vec![0; (width * height) as usize],
            open: BinaryHeap::new(),
            open_set: HashSet::default(),
            closed: HashSet::default(),
            came_from: HashMap::default(),
            dest_x: 0.0,
            dest_y: 0.0,
            dest_w_over2: 0.0,
            dest_h_over2: 0.0,
            parent_w_over2: 0.0,
            parent_h_over2: 0.0,
            max_iterations: MAX_ITERATIONS,
        }
    }

    pub fn set_max_iterations(&mut self, iterations: i32) {
        self.max_iterations = iterations;
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
    pub fn find<T: LocationChecker>(
        &mut self,
        checker: &T,
        start_x: i32,
        start_y: i32,
        dest: Destination,
    ) -> Option<Vec<Point>> {
        if dest.x < 0.0 || dest.y < 0.0 {
            return None;
        }
        if dest.x + dest.w > self.width as f32 + 0.1 || dest.y + dest.h > self.height as f32 + 0.1 {
            return None;
        }

        trace!(
            "Finding path from {},{} to within {} of {},{},{},{}",
            start_x,
            start_y,
            dest.dist,
            dest.x,
            dest.y,
            dest.w,
            dest.h
        );

        // let start_time = time::Instant::now();
        self.dest_x = dest.x + dest.w / 2.0;
        self.dest_y = dest.y + dest.h / 2.0;
        self.dest_w_over2 = dest.w / 2.0;
        self.dest_h_over2 = dest.h / 2.0;
        self.parent_w_over2 = dest.parent_w / 2.0;
        self.parent_h_over2 = dest.parent_h / 2.0;
        let dest_dist_squared = (dest.dist * dest.dist) as i32;
        let start = start_x + start_y * self.width;
        let initial_dist_squared = self.dist_squared(start);

        if initial_dist_squared <= dest_dist_squared {
            debug!("Mover is already inside the destination");
            return None;
        }

        // the set of discovered nodes that are not evaluated yet
        self.open.clear();
        self.open_set.clear();

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

        self.open
            .push(OpenEntry::new(start, self.f_score[start as usize]));
        self.open_set.insert(start);

        // info!("Spent {} secs in path find init", util::format_elapsed_secs(start_time.elapsed()));

        let loop_start_time = time::Instant::now();

        let mut iterations = 0;
        while iterations < self.max_iterations && !self.open.is_empty() {
            let current = self.pop_lowest_f_score_in_open_set();
            if self.is_goal(checker, current, dest_dist_squared) {
                trace!(
                    "Path loop time: {}",
                    util::format_elapsed_secs(loop_start_time.elapsed())
                );

                let path = self.reconstruct_path(current);
                if path.len() == 1 && path[0].x == start_x && path[0].y == start_y {
                    debug!("Found path with no moves.");
                    return None;
                }

                let final_dist_squared = self.dist_squared(path[0].x + path[0].y * self.width);
                trace!("Initial dist vs final dist: {} vs {}",
                    initial_dist_squared, final_dist_squared);

                if let Some(max_path_len) = dest.max_path_len {
                    if path.len() > max_path_len as usize {
                        debug!(
                            "Found path with too many moves: {} > {}",
                            path.len(),
                            max_path_len
                        );
                        return None;
                    }
                }

                return Some(path);
            }

            self.closed.insert(current);

            let neighbors = self.get_neighbors(current);
            for neighbor in neighbors.iter() {
                let neighbor = *neighbor;
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

                if !checker.passable(neighbor_x, neighbor_y) {
                    self.closed.insert(neighbor);
                    //trace!("Not passable");
                    continue;
                }

                let tentative_g_score =
                    self.g_score[current as usize] + self.get_cost(current, neighbor);
                if tentative_g_score >= self.g_score[neighbor as usize] {
                    self.push_to_open_set(neighbor, self.f_score[neighbor as usize]);
                    //trace!("G score indicates this neighbor is not preferable.");
                    continue; // this is not a better path
                }

                self.came_from.insert(neighbor, current);

                self.g_score[neighbor as usize] = tentative_g_score;
                self.f_score[neighbor as usize] = tentative_g_score + self.dist_squared(neighbor);
                self.push_to_open_set(neighbor, self.f_score[neighbor as usize]);
            }
            iterations += 1;
        }

        debug!(
            "No path found with {} iterations and {} in open set",
            iterations,
            self.open.len()
        );
        None
    }

    #[inline]
    fn is_goal<T: LocationChecker>(&self, checker: &T, current: i32, dest_dist_squared: i32) -> bool {
        self.dist_squared(current) <= dest_dist_squared && !checker.in_friend_space(current)
    }

    #[inline]
    fn reconstruct_path(&self, mut current: i32) -> Vec<Point> {
        trace!("Reconstructing path");

        // let reconstruct_time = time::Instant::now();
        let mut path: Vec<Point> = Vec::new();

        path.push(self.get_point(current));
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
    fn push_to_open_set(&mut self, index: i32, f_score: i32) {
        if self.open_set.contains(&index) {
            return;
        }

        self.open_set.insert(index);
        self.open.push(OpenEntry::new(index, f_score));
    }

    #[inline]
    fn pop_lowest_f_score_in_open_set(&mut self) -> i32 {
        let entry = self.open.pop().unwrap();
        self.open_set.remove(&entry.index);
        entry.index
    }

    #[inline]
    fn dist_squared(&self, start: i32) -> i32 {
        let s_x = (start % self.width) as f32 + self.parent_w_over2;
        let s_y = (start / self.width) as f32 + self.parent_h_over2;

        // closest distance from s_x, s_y to axis aligned dest
        // rect

        let mut dx = (s_x - self.dest_x).abs() - self.dest_w_over2;
        let mut dy = (s_y - self.dest_y).abs() - self.dest_h_over2;
        if dx < 0.0 {
            dx = 0.0;
        }
        if dy < 0.0 {
            dy = 0.0;
        }

        (dx * dx + dy * dy) as i32
    }
}
