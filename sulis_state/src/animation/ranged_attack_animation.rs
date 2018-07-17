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
use std::cell::RefCell;

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::image::Image;
use sulis_core::ui::{animation_state};
use {ActorState, EntityState, GameState, ScriptCallback, script::ScriptEntitySet};
use animation::{Anim};

pub (in animation) fn update(attacker: &Rc<RefCell<EntityState>>, model: &mut RangedAttackAnimModel, frac: f32) {
    if frac > 1.0 {
        if !model.has_attacked {
            let cb_targets = ScriptEntitySet::new(&model.defender, &vec![Some(Rc::clone(&attacker))]);
            for cb in model.callbacks.iter() {
                cb.before_attack(&cb_targets);
            }

            let area_state = GameState::area_state();

            let defender_cbs = model.defender.borrow().callbacks();
            let attacker_cbs = attacker.borrow().callbacks();

            attacker_cbs.iter().for_each(|cb| cb.before_attack(&cb_targets));
            defender_cbs.iter().for_each(|cb| cb.before_defense(&cb_targets));

            let (hit_kind, damage, text, color) =
                ActorState::weapon_attack(attacker, &model.defender);
            area_state.borrow_mut().add_feedback_text(text, &model.defender, color, 3.0);
            model.has_attacked = true;

            for cb in model.callbacks.iter() {
                cb.after_attack(&cb_targets, hit_kind, damage);
            }

            attacker_cbs.iter().for_each(|cb| cb.after_attack(&cb_targets, hit_kind, damage));
            defender_cbs.iter().for_each(|cb| cb.after_defense(&cb_targets, hit_kind, damage));
        }
    } else {
        model.cur_pos = (frac * model.vec.0 + model.start_pos.0,
                        frac * model.vec.1 + model.start_pos.1);
    }
}

pub (in animation) fn draw(model: &RangedAttackAnimModel, renderer: &mut GraphicsRenderer,
                           offset_x: f32, offset_y: f32,
                           scale_x: f32, scale_y: f32, millis: u32) {
    if let Some(ref projectile) = model.projectile {
        let mut draw_list = DrawList::empty_sprite();
        projectile.append_to_draw_list(&mut draw_list, &animation_state::NORMAL,
                                       model.cur_pos.0 + offset_x,
                                       model.cur_pos.1 + offset_y,
                                       projectile.get_width_f32(),
                                       projectile.get_height_f32(), millis);
        draw_list.set_scale(scale_x, scale_y);
        draw_list.rotate(model.angle);
        renderer.draw(draw_list);
    }
}

pub fn new(attacker: &Rc<RefCell<EntityState>>, defender: &Rc<RefCell<EntityState>>,
           callbacks: Vec<Box<ScriptCallback>>, duration_millis: u32) -> Anim {
    let mut start_pos = ((attacker.borrow().location.x + attacker.borrow().size.width / 2) as f32,
    (attacker.borrow().location.y + attacker.borrow().size.height / 2) as f32);
    let x = (defender.borrow().location.x + defender.borrow().size.width / 2) as f32 - start_pos.0;
    let y = (defender.borrow().location.y + defender.borrow().size.height / 2) as f32 - start_pos.1;
    let dist = (x * x + y * y).sqrt();

    let projectile = attacker.borrow().actor.stats.get_ranged_projectile();
    if let Some(ref projectile) = projectile {
        start_pos.0 -= projectile.get_width_f32() / 2.0;
        start_pos.1 -= projectile.get_height_f32() / 2.0;
    }

    let angle = y.atan2(x);

    let model = RangedAttackAnimModel {
        defender: Rc::clone(defender),
        angle,
        vec: (x, y),
        start_pos,
        cur_pos: (0.0, 0.0),
        has_attacked: false,
        projectile,
        callbacks,
    };

    let millis = (duration_millis as f32 * dist) as u32;
    Anim::new_ranged_attack(attacker, millis, model)
}

pub (in animation) struct RangedAttackAnimModel {
    defender: Rc<RefCell<EntityState>>,
    angle: f32,
    vec: (f32, f32),
    start_pos: (f32, f32),
    cur_pos: (f32, f32),
    pub (in animation) has_attacked: bool,
    projectile: Option<Rc<Image>>,
    callbacks: Vec<Box<ScriptCallback>>,
}
