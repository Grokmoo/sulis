use std::rc::Rc;
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::i32;

use grt::util::Point;
use state::{EntityState, AreaState};

pub struct PathFinder {
    area_state: Rc<RefCell<AreaState>>,
    pub width: i32,
    pub height: i32,

    f_score: Vec<i32>,
    g_score: Vec<i32>,
    open: HashSet<i32>,
    closed: HashSet<i32>,
    came_from: HashMap<i32, i32>,
}

impl PathFinder {
    pub fn new(area_state: Rc<RefCell<AreaState>>) -> PathFinder {
        let width = area_state.borrow().area.width;
        let height = area_state.borrow().area.height;

        debug!("Initializing pathfinder for {}", area_state.borrow().area.id);
        PathFinder {
            area_state,
            width,
            height,
            f_score: vec![0;(width*height) as usize],
            g_score: vec![0;(width*height) as usize],
            open: HashSet::new(),
            closed: HashSet::new(),
            came_from: HashMap::new(),
        }
    }

    pub fn find(&mut self, requester: Ref<EntityState>, dest_x: i32,
                dest_y: i32) -> Option<Vec<Point>> {
        if !Point::new(dest_x, dest_y).in_bounds(self.width, self.height) {
            return None;
        }
        debug!("Finding path from {:?} to {},{}",
               requester.location, dest_x, dest_y);

        let start = requester.location.x + requester.location.y * self.width;
        let goal = dest_x + dest_y * self.width;

        // the set of discovered nodes that are not evaluated yet
        self.open.clear();
        self.open.insert(start);

        // the set of nodes that have already been evaluated
        self.closed.clear();

        // initialize closed set based on passability
        let mut i = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.area_state.borrow().is_passable(&requester, x, y) {
                    self.closed.insert(i);
                }

                i += 1;
            }
        }

        if self.closed.contains(&goal) {
            trace!("Goal is unreachable, returning None");
            return None;
        }

        // for each node, the node it can be most efficiently reached from
        self.came_from.clear();

        // for each node, cost of getting from start to that node
        self.g_score.iter_mut().for_each(|v| *v = i32::MAX);

        // for each node, total cost of getting from start to goal passing by
        // this node
        self.f_score.iter_mut().for_each(|v| *v = i32::MAX);

        *self.g_score.get_mut(start as usize).unwrap() = 0;
        *self.f_score.get_mut(start as usize).unwrap() =
            self.heuristic_cost_estimate(start, goal);

        while !self.open.is_empty() {
            let current = self.find_lowest_f_score_in_open_set();
            if current == goal {
                return Some(self.reconstruct_path(current));
            }

            self.open.remove(&current);
            self.closed.insert(current);

            for neighbor in self.get_neighbors(current) {
                trace!("Checking neighbor {}", neighbor);
                if self.closed.contains(&neighbor) {
                    trace!("Already evaluated.");
                    continue; // neighbor has already been evaluated
                }

                if !self.open.contains(&neighbor) {
                    self.open.insert(neighbor);
                }

                let tentative_g_score = self.g_score.get(current as usize).unwrap() +
                    self.get_cost(current, neighbor);
                if tentative_g_score >= *self.g_score.get(neighbor as usize).unwrap() {
                    trace!("G score indicates this neighbor is not preferable.");
                    continue; // this is not a better path
                }

                self.came_from.insert(neighbor, current);
                *self.g_score.get_mut(neighbor as usize).unwrap() = tentative_g_score;
                *self.f_score.get_mut(neighbor as usize).unwrap() = tentative_g_score +
                    self.heuristic_cost_estimate(neighbor, goal);
            }
        }

        None
    }

    fn reconstruct_path(&self, current: i32) -> Vec<Point> {
        trace!("Reconstructing path");

        let mut path: Vec<Point> = Vec::new();

        path.push(self.get_point(current));
        let mut current = current;
        loop {
            trace!("Current {}", current);
            if let None = self.came_from.get(&current) {
                break;
            }
            current = *self.came_from.get(&current).unwrap();
            path.push(self.get_point(current));
        }

        trace!("Path reconstructed.  reversing.");
        path.reverse();
        path
    }

    fn get_point(&self, index: i32) -> Point {
        Point::new(index % self.width, index / self.width)
    }

    fn get_cost(&self, _from: i32, _to: i32) -> i32 {
        1
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

        trace!("Got neighbors for {}: {:?}", point, neighbors);
        neighbors
    }

    fn find_lowest_f_score_in_open_set(&self) -> i32 {
        let mut lowest = i32::MAX;
        let mut lowest_index = 0;

        for val in self.open.iter() {
            let f_score = self.f_score.get(*val as usize).unwrap();
            if f_score < &lowest {
                lowest = *f_score;
                lowest_index = *val;
            }
        }

        trace!("Found lowest f score of {} at {}", lowest, lowest_index);
        lowest_index
    }

    fn heuristic_cost_estimate(&self, start: i32, end: i32) -> i32 {
        let s_x = start % self.width;
        let s_y = start / self.width;

        let e_x = end % self.width;
        let e_y = end / self.width;

        let x_part = s_x - e_x;
        let y_part = s_y - e_y;

        trace!("Computed cost estimate from {} to {}", start, end);
        x_part * x_part + y_part * y_part
    }
}
