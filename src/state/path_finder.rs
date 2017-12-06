use std::rc::Rc;
use std::cell::Ref;
use std::collections::{HashMap, HashSet};
use std::usize;

use resource::Area;
use resource::Point;

use state::EntityState;

pub struct PathFinder {
    pub area: Rc<Area>,
    pub width: usize,
    pub height: usize,

    f_score: Vec<usize>,
    g_score: Vec<usize>,
    open: HashSet<usize>,
    closed: HashSet<usize>,
    came_from: HashMap<usize, usize>,
}

impl PathFinder {
    pub fn new(area: Rc<Area>) -> PathFinder {
        let width = area.width;
        let height = area.height;

        PathFinder {
            area,
            width,
            height,
            f_score: vec![0;width*height],
            g_score: vec![0;width*height],
            open: HashSet::new(),
            closed: HashSet::new(),
            came_from: HashMap::new(),
        }
    }

    pub fn find(&mut self, requester: Ref<EntityState>, dest_x: usize,
                dest_y: usize) -> Option<Vec<Point>> {
        let start = requester.location.x + requester.location.y * self.width;
        let goal = dest_x + dest_y * self.width;

        // the set of discovered nodes that are not evaluated yet
        self.open.clear();
        self.open.insert(start);

        // the set of nodes that have already been evaluated
        self.closed.clear();

        // initialize closed set based on passability
        for i in 0..(self.width*self.height) {
            if !self.area.get_path_grid(requester.size()).is_passable_index(i) {
                self.closed.insert(i);
            }
        }

        if self.closed.contains(&goal) { return None; }

        // for each node, the node it can be most efficiently reached from
        self.came_from.clear();

        // for each node, cost of getting from start to that node
        self.g_score.iter_mut().for_each(|v| *v = usize::MAX);

        // for each node, total cost of getting from start to goal passing by
        // this node
        self.f_score.iter_mut().for_each(|v| *v = usize::MAX);

        *self.g_score.get_mut(start).unwrap() = 0;
        *self.f_score.get_mut(start).unwrap() =
            self.heuristic_cost_estimate(start, goal);

        while !self.open.is_empty() {
            let current = self.find_lowest_f_score_in_open_set();
            if current == goal {
                return Some(self.reconstruct_path(current));
            }

            self.open.remove(&current);
            self.closed.insert(current);

            for neighbor in self.get_neighbors(current) {
                if self.closed.contains(&neighbor) {
                    continue; // neighbor has already been evaluated
                }

                if !self.open.contains(&neighbor) {
                    self.open.insert(neighbor);
                }

                let tentative_g_score = self.g_score.get(current).unwrap() +
                    self.get_cost(current, neighbor);
                if tentative_g_score >= *self.g_score.get(neighbor).unwrap() {
                    continue; // this is not a better path
                }

                self.came_from.insert(neighbor, current);
                *self.g_score.get_mut(neighbor).unwrap() = tentative_g_score;
                *self.f_score.get_mut(neighbor).unwrap() = tentative_g_score +
                    self.heuristic_cost_estimate(neighbor, goal);
            }
        }

        None
    }

    fn reconstruct_path(&self, current: usize) -> Vec<Point> {
        let mut path: Vec<Point> = Vec::new();

        path.push(self.get_point(current));
        let mut current = current;
        loop {
            if let None = self.came_from.get(&current) {
                break;
            }
            current = *self.came_from.get(&current).unwrap();
            path.push(self.get_point(current));
        }

        path.reverse();
        path
    }

    fn get_point(&self, index: usize) -> Point {
        Point::new(index % self.width, index / self.width)
    }

    fn get_cost(&self, _from: usize, _to: usize) -> usize {
        1
    }

    fn get_neighbors(&self, point: usize) -> Vec<usize> {
        let point = point as i32;
        let width = self.width as i32;
        let height = self.height as i32;

        let top = point - width;
        let right = point + 1;
        let left = point - 1;
        let bottom = point + width;

        let mut neighbors: Vec<usize> = Vec::new();
        if top > 0 { neighbors.push(top as usize); }
        if bottom < width * height { neighbors.push(bottom as usize); }
        if right % width != point % width { neighbors.push(right as usize); }
        if left % width != point % width { neighbors.push(left as usize); }

        neighbors
    }

    fn find_lowest_f_score_in_open_set(&self) -> usize {
        let mut lowest = usize::MAX;
        let mut lowest_index = 0;

        for val in self.open.iter() {
            let f_score = self.f_score.get(*val).unwrap();
            if f_score < &lowest {
                lowest = *f_score;
                lowest_index = *val;
            }
        }

        lowest_index
    }

    fn heuristic_cost_estimate(&self, start: usize, end: usize) -> usize {
        let s_x = start % self.width;
        let s_y = start / self.width;

        let e_x = end % self.width;
        let e_y = end / self.width;

        let x_component = if s_x > e_x { s_x - e_x } else { e_x - s_x };
        let y_component = if s_y > e_y { s_y - e_y } else { e_y - s_y };

        // we don't actually need the euclidean distance here; this equal weighted
        // always positive function is good enough
        x_component + y_component
    }
}
