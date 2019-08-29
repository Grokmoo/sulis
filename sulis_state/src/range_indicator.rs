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

use std::cell::RefCell;
use std::rc::Rc;

use sulis_core::image::Image;
use sulis_core::io::DrawList;
use sulis_core::resource::ResourceSet;
use sulis_core::ui::animation_state;

use crate::EntityState;

const NW: u8 = 1;
const N: u8 = 2;
const NE: u8 = 4;
const E: u8 = 8;
const SE: u8 = 16;
const S: u8 = 32;
const SW: u8 = 64;
const W: u8 = 128;

pub struct RangeIndicator {
    parent: Rc<RefCell<EntityState>>,
    neighbors: Vec<u8>,
    half_width: i32,
}

impl RangeIndicator {
    pub fn new(radius: f32, parent: &Rc<RefCell<EntityState>>) -> RangeIndicator {
        let parent = Rc::clone(parent);

        let half_width = radius.ceil() as i32 + 5;
        let width = (half_width * 2) as usize;

        let mut points = vec![true; width * width];

        {
            let parent = parent.borrow();
            for y in 0..width {
                for x in 0..width {
                    let (x1, y1) = (
                        x as i32 + parent.location.x - half_width,
                        y as i32 + parent.location.y - half_width,
                    );
                    let (x1, y1) = (x1 as f32 + 0.5, y1 as f32 + 0.5);

                    let idx = x + y * width;
                    points[idx] = parent.dist_to(x1, y1) > radius;
                }
            }
        }

        let mut neighbors = vec![0; width * width];
        for y in 0..width {
            for x in 0..width {
                neighbors[x + y * width] = find_neighbors(width, &points, x, y);
            }
        }

        RangeIndicator {
            neighbors,
            half_width,
            parent,
        }
    }

    pub fn get_draw_list(
        &self,
        image_set: &RangeIndicatorImageSet,
        x_offset: f32,
        y_offset: f32,
        millis: u32,
    ) -> DrawList {
        let x_offset = x_offset - self.parent.borrow().location.x as f32;
        let y_offset = y_offset - self.parent.borrow().location.y as f32;
        let mut draw_list = DrawList::empty_sprite();

        let half_width_f32 = self.half_width as f32;
        let width = (self.half_width * 2) as usize;
        for y in 0..width {
            for x in 0..width {
                let n = self.neighbors[x + y * width];

                if let Some(ref image) = image_set.images[n as usize] {
                    image.append_to_draw_list(
                        &mut draw_list,
                        &animation_state::NORMAL,
                        x as f32 - x_offset - half_width_f32,
                        y as f32 - y_offset - half_width_f32,
                        1.0,
                        1.0,
                        millis,
                    );
                }
            }
        }

        draw_list
    }
}

fn find_neighbors(width: usize, points: &[bool], x: usize, y: usize) -> u8 {
    let mut total = 0;

    let (x, y, w) = (x as i32, y as i32, width as i32);

    if check_idx(w, points, x, y) {
        return 255;
    }

    if check_idx(w, points, x - 1, y) {
        total += W;
    }
    if check_idx(w, points, x - 1, y - 1) {
        total += NW;
    }
    if check_idx(w, points, x, y - 1) {
        total += N;
    }
    if check_idx(w, points, x + 1, y - 1) {
        total += NE;
    }
    if check_idx(w, points, x + 1, y) {
        total += E;
    }
    if check_idx(w, points, x + 1, y + 1) {
        total += SE;
    }
    if check_idx(w, points, x, y + 1) {
        total += S;
    }
    if check_idx(w, points, x - 1, y + 1) {
        total += SW;
    }

    total
}

fn check_idx(width: i32, points: &[bool], x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= width || y >= width {
        return false;
    }

    points[x as usize + y as usize * width as usize]
}

pub struct RangeIndicatorImageSet {
    images: Vec<Option<Rc<dyn Image>>>,
}

impl RangeIndicatorImageSet {
    pub fn new(prefix: String) -> RangeIndicatorImageSet {
        let rules = [
            ("outer_nw", W + NW + N),
            ("outer_n", NW + N + NE),
            ("outer_ne", N + NE + E),
            ("outer_e", NE + E + SE),
            ("outer_se", E + SE + S),
            ("outer_s", SE + S + SW),
            ("outer_sw", S + SW + W),
            ("outer_w", SW + W + NW),
            ("outer_se", SW + W + NW + N + NE),
            ("outer_sw", NW + N + NE + E + SE),
            ("outer_nw", NE + E + SE + S + SW),
            ("outer_ne", SE + S + SW + W + NW),
            ("outer_nw", SW + W + NW + N),
            ("outer_ne", SE + N + NE + E),
            ("outer_se", NE + E + SE + S),
            ("outer_sw", NW + S + SW + W),
            ("outer_n", N + NE),
            ("outer_n", N + NW),
            ("outer_s", S + SE),
            ("outer_s", S + SW),
            ("outer_e", E + NE),
            ("outer_e", E + SE),
            ("outer_w", W + NW),
            ("outer_w", W + SW),
            ("outer_ne", NW + N + NE + E),
            ("outer_nw", NW + N + NE + W),
            ("outer_se", E + SE + S + SW),
            ("outer_sw", S + SW + W + SE),
            ("inner_nw", SE),
            ("inner_ne", SW),
            ("inner_sw", NE),
            ("inner_se", NW),
            ("outer_n", NE + E + SE + S + SW + W + NW),
            ("outer_s", SW + W + NW + N + NE + E + SE),
            ("outer_e", SE + S + SW + W + NW + N + NE),
            ("outer_w", NW + N + NE + E + SE + S + SW),
            ("center", 0),
        ];

        let mut images = vec![None; 256];
        for rule in rules.into_iter() {
            let rule = match ImageRule::new(&prefix, rule.0, rule.1) {
                None => continue,
                Some(rule) => rule,
            };

            images[rule.neighbors as usize] = Some(rule.image);
        }

        RangeIndicatorImageSet { images }
    }
}

struct ImageRule {
    image: Rc<dyn Image>,
    neighbors: u8,
}

impl ImageRule {
    fn new(prefix: &str, postfix: &str, neighbors: u8) -> Option<ImageRule> {
        let id = format!("{}{}", prefix, postfix);
        let image = match ResourceSet::image(&id) {
            None => {
                warn!("No image found for selection area {}", id);
                return None;
            }
            Some(image) => image,
        };

        Some(ImageRule { image, neighbors })
    }
}
