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

use crate::{EntityState, GeneratedArea};

#[must_use]
pub fn calculate_los(
    exp: &mut [bool],
    area: &GeneratedArea,
    prop_vis_grid: &[bool],
    prop_grid: &[Vec<usize>],
    entity: &mut EntityState,
    delta_x: i32,
    delta_y: i32,
) -> HashSet<usize> {
    let max_dist = area.area.vis_dist;
    let entity_x = entity.location.x + entity.size.width / 2;
    let entity_y = entity.location.y + entity.size.height / 2;

    let los = entity.pc_vis_mut();

    let min_x = 0.max(entity_x - max_dist + delta_x.min(0));

    let max_x = area.width.min(entity_x + max_dist + delta_x.max(0));

    let min_y = 0.max(entity_y - max_dist + delta_y.min(0));

    let max_y = area.height.min(entity_y + max_dist + delta_y.max(0));

    let src_elev = area.layer_set.elevation(entity_x, entity_y);

    let mut props_vis: HashSet<usize> = HashSet::new();

    for y in min_y..max_y {
        for x in min_x..max_x {
            let index = (x + y * area.width) as usize;
            if check_vis(area, prop_vis_grid, entity_x, entity_y, x, y, src_elev) {
                los[index] = true;
                exp[index] = true;

                for prop in &prop_grid[index] {
                    props_vis.insert(*prop);
                }
            } else {
                los[index] = false;
            }
        }
    }

    props_vis
}

pub fn has_visibility(
    area: &GeneratedArea,
    prop_vis_grid: &[bool],
    entity: &EntityState,
    target: &EntityState,
) -> bool {
    let start_x = entity.location.x + entity.size.width / 2;
    let start_y = entity.location.y + entity.size.height / 2;
    let src_elev = area.layer_set.elevation(start_x, start_y);

    for p in target.location_points() {
        if check_vis(area, prop_vis_grid, start_x, start_y, p.x, p.y, src_elev) {
            return true;
        }
    }

    false
}

fn check_vis(
    area: &GeneratedArea,
    prop_vis_grid: &[bool],
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    src_elev: u8,
) -> bool {
    let dist_squared =
        (start_x - end_x) * (start_x - end_x) + (start_y - end_y) * (start_y - end_y);

    if dist_squared < area.area.vis_dist_up_one_squared {
        cast_ray(
            area,
            prop_vis_grid,
            start_x,
            start_y,
            end_x,
            end_y,
            src_elev + 1,
        )
    } else if dist_squared < area.area.vis_dist_squared {
        cast_ray(
            area,
            prop_vis_grid,
            start_x,
            start_y,
            end_x,
            end_y,
            src_elev,
        )
    } else {
        false
    }
}

fn cast_ray(
    area: &GeneratedArea,
    prop_vis_grid: &[bool],
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    src_elev: u8,
) -> bool {
    if (end_y - start_y).abs() < (end_x - start_x).abs() {
        if start_x > end_x {
            cast_low(
                area,
                prop_vis_grid,
                end_x,
                end_y,
                start_x,
                start_y,
                src_elev,
            )
        } else {
            cast_low(
                area,
                prop_vis_grid,
                start_x,
                start_y,
                end_x,
                end_y,
                src_elev,
            )
        }
    } else if start_y > end_y {
        cast_high(
            area,
            prop_vis_grid,
            end_x,
            end_y,
            start_x,
            start_y,
            src_elev,
        )
    } else {
        cast_high(
            area,
            prop_vis_grid,
            start_x,
            start_y,
            end_x,
            end_y,
            src_elev,
        )
    }
}

fn check(area: &GeneratedArea, prop_vis_grid: &[bool], x: i32, y: i32, src_elev: u8) -> bool {
    let index = (x + y * area.width) as usize;

    prop_vis_grid[index]
        && area.layer_set.is_visible_index(index)
        && area.layer_set.elevation_index(index) <= src_elev
}

fn cast_high(
    area: &GeneratedArea,
    prop_vis_grid: &[bool],
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    src_elev: u8,
) -> bool {
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
        } else if !check(area, prop_vis_grid, x, y, src_elev) {
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

fn cast_low(
    area: &GeneratedArea,
    prop_vis_grid: &[bool],
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
    src_elev: u8,
) -> bool {
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
        } else if !check(area, prop_vis_grid, x, y, src_elev) {
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
