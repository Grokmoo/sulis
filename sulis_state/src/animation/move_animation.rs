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

use std::cell::{Cell, RefCell};
use std::cmp;
use std::rc::Rc;

use crate::{animation::particle_generator::Param, animation::Anim, EntityState, GameState};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::animation_state;
use sulis_core::util::{ExtInt, Offset, Point, Rect, Scale};
use sulis_module::ObjectSize;

fn check_immediate_cancel(mover: &Rc<RefCell<EntityState>>, model: &mut MoveAnimModel) -> bool {
    let cur_area_id = mover.borrow().location.area_id.to_string();
    if (model.area_id != cur_area_id) || model.path.is_empty() {
        return true;
    }

    if GameState::is_combat_active() != model.combat_mode {
        return true;
    }

    if mover.borrow().marked_for_removal {
        return true;
    }

    let actor = &mover.borrow().actor;
    if actor.is_dead() || actor.stats.move_disabled {
        return true;
    }

    false
}

pub(in crate::animation) fn update(
    mover: &Rc<RefCell<EntityState>>,
    marked_for_removal: &Rc<Cell<bool>>,
    model: &mut MoveAnimModel,
    millis: u32,
) {
    if check_immediate_cancel(mover, model) {
        marked_for_removal.set(true);
        return;
    }

    let frame_index = cmp::min(
        (millis / model.frame_time_millis) as usize,
        model.path.len() - 1,
    );
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
    let area_state = GameState::get_area_state(&mover.borrow().location.area_id).unwrap();
    if !area_state
        .borrow_mut()
        .move_entity(mover, p.x, p.y, move_ap as u32)
    {
        marked_for_removal.set(true);
        return;
    }

    if frame_index == model.path.len() - 1 {
        marked_for_removal.set(true);
    }
}

pub(in crate::animation) fn draw(
    model: &MoveAnimModel,
    renderer: &mut dyn GraphicsRenderer,
    offset: Offset,
    scale: Scale,
    millis: u32,
) {
    if model.path.is_empty() {
        return;
    }

    let last = &model.path[model.path.len() - 1];

    let rect = Rect {
        x: offset.x + last.x as f32,
        y: offset.y + last.y as f32,
        w: model.owner_size.width as f32,
        h: model.owner_size.height as f32,
    };

    let mut draw_list = DrawList::empty_sprite();
    model.owner_size.selection_image.append_to_draw_list(
        &mut draw_list,
        &animation_state::NORMAL,
        rect,
        millis,
    );
    draw_list.set_scale(scale);
    renderer.draw(draw_list);
}

pub(in crate::animation) fn cleanup(mover: &Rc<RefCell<EntityState>>, model: &mut MoveAnimModel) {
    let old_pos = mover.borrow().location.to_point();

    let area = GameState::get_area_state(&mover.borrow().location.area_id).unwrap();

    let mut first = true;
    let mut target = None;
    let mut index = model.last_frame_index as usize;

    let do_move_back = !mover.borrow().is_party_member() || GameState::is_combat_active();

    if do_move_back {
        loop {
            let p = model.path[index];
            if area
                .borrow()
                .is_passable_for_entity(&mover.borrow(), p.x, p.y)
            {
                if !first {
                    target = Some(p);
                }
                break;
            }

            if index == 0 {
                break;
            }

            index -= 1;
            first = false;
        }
    }

    if let Some(p) = target {
        area.borrow_mut().move_entity(mover, p.x, p.y, 0);
    }

    let new_pos = mover.borrow().location.to_point();
    let dx = (new_pos.x - old_pos.x) as f32 + mover.borrow().sub_pos.0;
    let dy = (new_pos.y - old_pos.y) as f32 + mover.borrow().sub_pos.1;

    trace!("Canceling subpos with {:?}", mover.borrow().sub_pos);
    if new_pos != old_pos {
        let base_time = model.frame_time_millis * 5;
        let frac = 1000.0 / base_time as f32;
        let x = Param::with_speed(-dx, dx * frac);
        let y = Param::with_speed(-dy, dy * frac);
        let anim = Anim::new_entity_subpos(mover, ExtInt::Int(base_time), x, y);
        GameState::add_animation(anim);

        mover.borrow_mut().sub_pos = (-dx, -dy);
    } else {
        mover.borrow_mut().sub_pos = (0.0, 0.0);
    }
}

pub fn new(mover: &Rc<RefCell<EntityState>>, path: Vec<Point>, base_time_millis: u32) -> Anim {
    let frame_time_millis = base_time_millis as f32 / mover.borrow().actor.stats.move_anim_rate;
    let frame_time_millis = frame_time_millis.round() as u32;

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
        }
        if i < path.len() as i32 - 1 {
            // next2 is already set to the final point
            next = path[i as usize + 1];
        } else {
            // next and next2 are already set to the final point
        }

        let avg_x = (prev2.x + prev.x + cur.x + next.x + next2.x) as f32 / 5.0;
        let avg_y = (prev2.y + prev.y + cur.y + next.y + next2.y) as f32 / 5.0;

        smoothed_path.push((avg_x, avg_y));

        prev2 = prev;
        prev = cur;
    }

    let duration_millis = frame_time_millis * path.len() as u32;
    let area_id = mover.borrow().location.area_id.to_string();
    let model = MoveAnimModel {
        combat_mode: GameState::is_combat_active(),
        area_id,
        path,
        last_frame_index: 0,
        frame_time_millis,
        smoothed_path,
        owner_size: Rc::clone(&mover.borrow().size),
    };

    Anim::new_move(mover, duration_millis, model)
}

pub struct MoveAnimModel {
    area_id: String,
    combat_mode: bool, // whether this move was created in or out of combat.  a change in
    // this status will cancel the move
    pub(in crate::animation) path: Vec<Point>,
    pub(in crate::animation) last_frame_index: i32,
    frame_time_millis: u32,
    smoothed_path: Vec<(f32, f32)>,
    owner_size: Rc<ObjectSize>,
}
