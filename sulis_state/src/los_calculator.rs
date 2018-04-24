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

use std::rc::Rc;
use std::time;
use std::cmp;

use sulis_core::util;
use sulis_module::Area;
use {EntityState};

pub fn calculate_los(los: &mut Vec<bool>, exp: &mut Vec<bool>,
                     area: &Rc<Area>, entity: &EntityState, delta_x: i32, delta_y: i32) {
    let start_time = time::Instant::now();

    let max_dist = area.vis_dist;
    let entity_x = entity.location.x + entity.size.width / 2;
    let entity_y = entity.location.y + entity.size.height / 2;

    let min_x = cmp::max(0, entity_x - max_dist + if delta_x < 0 { delta_x } else { 0 });
    let max_x = cmp::min(area.width, entity_x + max_dist + if delta_x > 0 { delta_x } else { 0 });
    let min_y = cmp::max(0, entity_y - max_dist + if delta_y < 0 { delta_y } else { 0 });
    let max_y = cmp::min(area.height, entity_y + max_dist + if delta_y > 0 { delta_y } else { 0 });

    let src_elev = area.layer_set.elevation(entity_x, entity_y);

    for y in min_y..max_y {
        for x in min_x..max_x {
            let index = (x + y * area.width) as usize;
            if check_vis(area, entity_x, entity_y, x, y, src_elev) {
                los[index] = true;
                exp[index] = true;
            } else {
                los[index] = false;
            }
        }
    }

    trace!("Visibility compute time: {}", util::format_elapsed_secs(start_time.elapsed()));
}

pub fn has_visibility(area: &Rc<Area>, entity: &EntityState, target: &EntityState) -> bool {
    let start_x = entity.location.x + entity.size.width / 2;
    let start_y = entity.location.y + entity.size.height / 2;
    let src_elev = area.layer_set.elevation(start_x, start_y);

    for p in target.location_points() {
        if check_vis(area, start_x, start_y, p.x, p.y, src_elev) { return true; }
    }

    false
}

fn check_vis(area: &Rc<Area>, start_x: i32, start_y: i32, end_x: i32, end_y: i32, src_elev: u8) -> bool {
    let dist_squared = (start_x - end_x) * (start_x - end_x) + (start_y - end_y) * (start_y - end_y);

    if dist_squared < area.vis_dist_up_one_squared {
        cast_ray(area, start_x, start_y, end_x, end_y, src_elev + 1)
    } else if dist_squared < area.vis_dist_squared {
        cast_ray(area, start_x, start_y, end_x, end_y, src_elev)
    } else {
        false
    }
}

fn cast_ray(area: &Rc<Area>, start_x: i32, start_y: i32, end_x: i32, end_y: i32, src_elev: u8) -> bool {
    if (end_y - start_y).abs() < (end_x - start_x).abs() {
        if start_x > end_x {
            cast_low(area, end_x, end_y, start_x, start_y, src_elev)
        } else {
            cast_low(area, start_x, start_y, end_x, end_y, src_elev)
        }
    } else {
        if start_y > end_y {
            cast_high(area, end_x, end_y, start_x, start_y, src_elev)
        } else {
            cast_high(area, start_x, start_y, end_x, end_y, src_elev)
        }
    }
}

fn check(area: &Rc<Area>, x: i32, y: i32, src_elev: u8) -> bool {
    let index = (x + y * area.width) as usize;

    area.layer_set.is_visible_index(index) && area.layer_set.elevation_index(index) <= src_elev
}

fn cast_high(area: &Rc<Area>, start_x: i32, start_y: i32, end_x: i32, end_y: i32, src_elev: u8) -> bool {
    let mut delta_x = end_x - start_x;
    let delta_y = end_y - start_y;

    let xi = if delta_x < 0 {
        delta_x = -delta_x;
        -1
    } else {
        1
    };

    // don't check the first point
    let mut first = true;
    let mut d = 2 * delta_x - delta_y;
    let mut x = start_x;
    for y in start_y..end_y {
        if first {
            first = false;
        } else if !check(area, x, y, src_elev) {
            return false;
        }

        if d > 0 {
            x += xi;
            d -= 2 * delta_y;
        }
        d += 2 * delta_x;
    }

    true
}

fn cast_low(area: &Rc<Area>, start_x: i32, start_y: i32, end_x: i32, end_y: i32, src_elev: u8) -> bool {
    let delta_x = end_x - start_x;
    let mut delta_y = end_y - start_y;

    let yi = if delta_y < 0 {
        delta_y = -delta_y;
        -1
    } else {
        1
    };

    // don't check the first point
    let mut first = true;
    let mut d = 2 * delta_y - delta_x;
    let mut y = start_y;
    for x in start_x..end_x {
        if first {
            first = false;
        } else if !check(area, x, y, src_elev) {
            return false;
        }

        if d > 0 {
            y += yi;
            d -= 2 * delta_x;
        }
        d += 2 * delta_y;
    }

    true
}


