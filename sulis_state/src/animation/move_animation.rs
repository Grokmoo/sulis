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

use std::cmp;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use {animation::Anim, EntityState, GameState};
use sulis_core::util::{Point};

pub (in animation) fn update(mover: &Rc<RefCell<EntityState>>, marked_for_removal: &Rc<Cell<bool>>,
                             model: &mut MoveAnimModel, millis: u32) {
    let cur_area_id = mover.borrow().location.area_id.to_string();
    if (model.area_id != cur_area_id) || model.path.is_empty()  || mover.borrow().actor.is_dead() {
        marked_for_removal.set(true);
        return;
    }

    let frame_index = cmp::min((millis / model.frame_time_millis) as usize, model.path.len() - 1);
    let frame_frac = (millis % model.frame_time_millis) as f32 / model.frame_time_millis as f32;

    if frame_index != model.path.len() - 1 {
        let dx = model.smoothed_path[frame_index + 1].0 - model.smoothed_path[frame_index].0;
        let dy = model.smoothed_path[frame_index + 1].1 - model.smoothed_path[frame_index].1;

        let dx = dx * frame_frac;
        let dy = dy * frame_frac;

        let offset_x = model.smoothed_path[frame_index].0 - model.path[frame_index].x as f32;
        let offset_y = model.smoothed_path[frame_index].1 - model.path[frame_index].y as f32;

        mover.borrow_mut().sub_pos = (dx + offset_x, dy + offset_y);
    }

    if frame_index as i32 == model.last_frame_index {
        return;
    }
    let move_ap = frame_index as i32 - model.last_frame_index;
    model.last_frame_index = frame_index as i32;

    let p = model.path[frame_index];
    let area_state = GameState::area_state();
    if !area_state.borrow_mut().move_entity(mover, p.x, p.y, move_ap as u32) {
        marked_for_removal.set(true);
        return;
    }

    if frame_index == model.path.len() - 1 {
        marked_for_removal.set(true);
    }
}

pub (in animation) fn cleanup(owner: &Rc<RefCell<EntityState>>) {
    owner.borrow_mut().sub_pos = (0.0, 0.0);
}

pub fn new(mover: &Rc<RefCell<EntityState>>, path: Vec<Point>, frame_time_millis: u32) -> Anim {
    let mut smoothed_path = Vec::new();
    let mut prev2 = path[0];
    let mut prev = path[0];
    let mut next = path[path.len() - 1];
    let mut next2 = path[path.len() - 1];

    for i in 0..(path.len() as i32) {
        let cur: Point = path[i as usize];
        if i < path.len() as i32 - 2 {
            next = path[i as usize + 1];
            next2 = path[i as usize + 2];
        } if i < path.len() as i32 - 1 {
            next = path[i as usize + 1];
            // next2 is already set to the final point
        } else {
            // next and next2 are already set to the final point
        }

        let mut avg_x = (prev2.x + prev.x + cur.x + next.x + next2.x) as f32 / 5.0;
        let mut avg_y = (prev2.y + prev.y + cur.y + next.y + next2.y) as f32 / 5.0;

        smoothed_path.push((avg_x, avg_y));

        prev2 = prev;
        prev = cur;
    }

    let duration_millis = frame_time_millis * path.len() as u32;
    let area_id = mover.borrow().location.area_id.to_string();
    let model = MoveAnimModel {
        area_id,
        path,
        last_frame_index: 0,
        frame_time_millis,
        smoothed_path
    };

    Anim::new_move(mover, duration_millis, model)
}

pub struct MoveAnimModel {
    area_id: String,
    pub (in animation) path: Vec<Point>,
    pub (in animation) last_frame_index: i32,
    frame_time_millis: u32,
    smoothed_path: Vec<(f32, f32)>,
}
