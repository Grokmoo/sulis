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

use std::collections::{HashMap, HashSet};
use std::cmp::Ordering;
use std::io::Error;

use sulis_core::util::{Point, gen_rand, shuffle};
use crate::generator::RoomParams;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TileKind {
    Wall,
    Corridor(usize),
    Room { region: usize, transition: bool },
    DoorWay,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
    None,
}

const DIRECTIONS: [Direction; 4] = [ Direction::North, Direction::South, Direction::East, Direction::West];

impl Direction {
    fn add(&self, p: Point, mult: i32) -> Point {
        match self {
            Direction::North => Point::new(p.x, p.y - mult),
            Direction::South => Point::new(p.x, p.y + mult),
            Direction::East => Point::new(p.x + mult, p.y),
            Direction::West => Point::new(p.x - mult, p.y),
            Direction::None => panic!(),
        }
    }
}

pub struct Maze {
    width: i32,
    height: i32,
    rooms: Vec<Room>,
    tiles: Vec<TileKind>,
    cur_region: usize,
}

impl Maze {
    pub(crate) fn new(width: u32, height: u32) -> Maze {
        Maze {
            width: width as i32,
            height: height as i32,
            rooms: Vec::new(),
            tiles: vec![TileKind::Wall; (width * height) as usize],
            cur_region: 0,
        }
    }

    pub(crate) fn generate(&mut self, params: &RoomParams,
                           open_locs: &[Point]) -> Result<(), Error> {
        self.generate_rooms(params, open_locs);
        info!("Generated {} total rooms", self.rooms.len());

        if params.gen_corridors {
            self.generate_corridors(params);

            self.connect_regions(params);

            self.remove_dead_ends(params);
        }
        Ok(())
    }

    fn remove_dead_ends(&mut self, params: &RoomParams) {
        let mut did_work = true;
        while did_work {
            did_work = false;
            for y in 0..self.height {
                for x in 0..self.width {
                    if self.tile(x, y) == TileKind::Wall { continue; }

                    let mut exits = 0;
                    for dir in DIRECTIONS.iter() {
                        let p = dir.add(Point::new(x, y), 1);
                        if p.x < 0 || p.y < 0 || p.x >= self.width || p.y >= self.height {
                            continue;
                        }

                        if self.tile(p.x, p.y) != TileKind::Wall { exits += 1; }
                    }

                    if exits > 1 { continue; }

                    if gen_rand(1, 101) < params.dead_end_keep_chance { continue; }

                    self.set_tile(x, y, TileKind::Wall);

                    did_work = true;
                }
            }
        }
    }

    fn connect_regions(&mut self, params: &RoomParams) {
        let mut connector_regions = HashMap::new();
        for y in 0..self.height {
            for x in 0..self.width {
                if self.tile(x, y) != TileKind::Wall { continue; }

                let mut regions = HashSet::new();
                for dir in DIRECTIONS.iter() {
                    let p = dir.add(Point::new(x, y), 1);
                    if p.x < 0 || p.y < 0 || p.x >= self.width || p.y >= self.height {
                        continue;
                    }
                    match self.tile(p.x, p.y) {
                        TileKind::Room { region, .. } => { regions.insert(region); },
                        TileKind::Corridor(region) => { regions.insert(region); },
                        TileKind::Wall => (),
                        TileKind::DoorWay => panic!(), // should not be any doorways yet
                    }
                }

                if regions.len() < 2 { continue; }

                connector_regions.insert(Point::new(x, y), regions);
            }
        }

        let mut connectors: Vec<Point> = connector_regions.keys().map(|k| k.clone()).collect();
        shuffle(&mut connectors);
        info!("Found connectors: {}", connectors.len());

        let mut merged = vec![0; self.cur_region as usize];
        let mut open_regions = HashSet::new();
        for i in 0..self.cur_region {
            open_regions.insert(i);
            merged[i] = i;
        }

        while open_regions.len() > 1 {
            let connector = match connectors.get(0) {
                None => break,
                Some(conn) => *conn,
            };

            self.set_tile(connector.x, connector.y, TileKind::DoorWay);

            let mut sources: Vec<usize> = connector_regions[&connector].iter()
                .map(|region| merged[*region]).collect();
            let dest = sources.remove(0);

            for i in 0..self.cur_region {
                if sources.contains(&merged[i]) { merged[i] = dest; }
            }

            open_regions.retain(|region| !sources.contains(region));

            connectors.retain(|conn| {
                if (connector.x - conn.x) * (connector.x - conn.x) +
                    (connector.y - conn.y) * (connector.y - conn.y) < 4 { return false; }

                let regions: HashSet<_> = connector_regions[conn].iter().
                    map(|region| merged[*region]).collect();
                if regions.len() > 1 { return true; }

                // randomly add some additional connectors
                if gen_rand(1, 101) < params.extra_connection_chance {
                    self.set_tile(conn.x, conn.y, TileKind::DoorWay);
                }

                false
            });
        }
    }

    fn generate_rooms(&mut self, params: &RoomParams, open_locs: &[Point]) {
        if !params.invert {
            for loc in open_locs {
                let room = Room::center_on(self.width, self.height, params, *loc);
                self.add_room(room, true);
            }
        }

        debug!("Generating rooms with {} attempts", params.room_placement_attempts);
        for _ in 0..params.room_placement_attempts {
            let room = Room::gen(self.width, self.height, params);
            let mut overlaps = false;
            for other in &self.rooms {
                if room.overlaps(other, params) {
                    overlaps = true;
                    break;
                }
            }

            if params.invert {
                for p in open_locs {
                    if room.contains(*p) {
                        overlaps = true;
                        break;
                    }
                }
            }

            if overlaps { continue; }

            self.add_room(room, false);
        }
    }

    fn generate_corridors(&mut self, params: &RoomParams) {
        for y in (1..self.height - 1).step_by(2) {
            for x in (1..self.width - 1).step_by(2) {
                if self.tile(x, y) != TileKind::Wall { continue; }

                self.grow_maze(x, y, params);
                self.cur_region += 1;
            }
        }
    }

    fn grow_maze(&mut self, x: i32, y: i32, params: &RoomParams) {
        self.set_tile(x, y, TileKind::Corridor(self.cur_region));

        let mut last_dir = Direction::None;
        let mut cells = vec![Point::new(x, y)];

        loop {
            let cell = match cells.last() {
                None => break,
                Some(cell) => cell,
            };

            let mut unmade_cells = Vec::new();
            for dir in DIRECTIONS.iter() {
                let p = dir.add(*cell, 2);
                if p.x < 0 || p.y < 0 || p.x >= self.width || p.y >= self.height {
                    continue;
                }

                if self.tile(p.x, p.y) != TileKind::Wall { continue; }

                unmade_cells.push(*dir);
            }

            // prefer sticking to the same direction with some probability
            unmade_cells.sort_by(|a, b| {
                if *a == last_dir { return Ordering::Less; }
                if *b == last_dir { return Ordering::Greater; }
                return Ordering::Equal;
            });

            if unmade_cells.is_empty() {
                cells.pop();
                last_dir = Direction::None;
            } else {
                let dir = if unmade_cells.len() == 1 || gen_rand(1, 101) >= params.winding_chance {
                    unmade_cells[0]
                } else {
                    unmade_cells[gen_rand(0, unmade_cells.len())]
                };

                let new_cell = dir.add(*cell, 1);
                self.set_tile(new_cell.x, new_cell.y, TileKind::Corridor(self.cur_region));
                let new_cell = dir.add(*cell, 2);
                self.set_tile(new_cell.x, new_cell.y, TileKind::Corridor(self.cur_region));
                cells.push(new_cell);
                last_dir = dir;
            }
        }
    }

    fn add_room(&mut self, room: Room, transition: bool) {
        for yi in room.y..(room.y + room.height) {
            for xi in room.x..(room.x + room.width) {
                self.set_tile(xi, yi, TileKind::Room{
                    region: self.cur_region,
                    transition,
                });
            }
        }

        self.cur_region += 1;

        self.rooms.push(room);
    }

    /// Returns an array of the tilekind of the specified tile and its 4 neighbors.
    /// In order: self (center), North, East, South, West
    pub fn neighbors(&self, x: i32, y: i32) -> [Option<TileKind>; 5] {
        let c = self.tile_checked(x, y);
        let n = self.tile_checked(x, y - 1);
        let e = self.tile_checked(x + 1, y);
        let s = self.tile_checked(x, y + 1);
        let w = self.tile_checked(x - 1, y);

        [c, n, e, s, w]
    }

    pub fn region(&self, x: i32, y: i32) -> Option<usize> {
        match self.tile(x, y) {
            TileKind::Wall => None,
            TileKind::Corridor(region) => Some(region),
            TileKind::Room { region, .. } => Some(region),
            TileKind::DoorWay => Some(std::u32::MAX as usize),
        }
    }

    pub fn tile_checked(&self, x: i32, y: i32) -> Option<TileKind> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height { return None; }

        Some(self.tile(x, y))
    }

    pub fn tile(&self, x: i32, y: i32) -> TileKind {
        self.tiles[(x + y * self.width) as usize]
    }

    fn set_tile(&mut self, x: i32, y: i32, tile: TileKind) {
        self.tiles[(x + y * self.width) as usize] = tile;
    }

    pub fn width(&self) -> i32 { self.width }

    pub fn height(&self) -> i32 { self.height }
}

#[derive(Debug)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Room {
    fn gen(area_width: i32, area_height: i32, params: &RoomParams) -> Room {
        // align rooms with odd tiles
        let width = (gen_rand(params.min_size.x, params.max_size.x + 1) / 2) * 2 + 1;
        let height = (gen_rand(params.min_size.y, params.max_size.y + 1) / 2) * 2 + 1;
        let x = (gen_rand(0, area_width - width) / 2) * 2 + 1;
        let y = (gen_rand(0, area_height - height) / 2) * 2 + 1;

        Room {
            x,
            y,
            width,
            height
        }
    }

    fn center_on(area_width: i32, area_height: i32, params: &RoomParams, loc: Point) -> Room {
        let mut room = Room::gen(area_width, area_height, params);
        room.x = loc.x - room.width / 2;
        room.y = loc.y - room.height / 2;

        if room.x < 0 { room.x = 0; }
        else if room.x + room.width >= area_width {
            room.x = area_width - room.width - 1;
        }

        if room.y < 0 { room.y = 0; }
        else if room.y + room.height >= area_height {
            room.y = area_height - room.height - 1;
        }

        room
    }

    fn contains(&self, p: Point) -> bool {
        if p.x < self.x || p.x > self.x + self.width { return false; }
        if p.y < self.y || p.y > self.y + self.height { return false; }

        true
    }

    fn overlaps(&self, other: &Room, params: &RoomParams) -> bool {
        !self.not_overlaps(other, params)
    }

    fn not_overlaps(&self, other: &Room, params: &RoomParams) -> bool {
        let sp = params.min_spacing as i32 - 1;

        if self.x > other.x + other.width + sp || other.x > self.x + self.width + sp {
            return true;
        }

        if self.y > other.y + other.height + sp || other.y > self.y + self.height + sp {
            return true;
        }

        false
    }
}
