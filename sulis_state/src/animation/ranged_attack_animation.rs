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
use std::time::Instant;

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::image::Image;
use sulis_core::ui::{animation_state, Widget};
use sulis_core::util;
use {AreaState, EntityState};
use animation::Animation;

pub struct RangedAttackAnimation {
    attacker: Rc<RefCell<EntityState>>,
    defender: Rc<RefCell<EntityState>>,

    vec: (f32, f32),
    start_pos: (f32, f32),
    cur_pos: (f32, f32),

    total_time_millis: f32,
    start_time: Instant,
    marked_for_removal: bool,
    has_attacked: bool,

    projectile: Option<Rc<Image>>,
}

impl RangedAttackAnimation {
    pub fn new(attacker: &Rc<RefCell<EntityState>>, defender: &Rc<RefCell<EntityState>>,
               base_time_millis: u32) -> RangedAttackAnimation {
        let mut start_pos = ((attacker.borrow().location.x + attacker.borrow().size.size / 2) as f32,
            (attacker.borrow().location.y + attacker.borrow().size.size / 2) as f32);
        let x = (defender.borrow().location.x + defender.borrow().size.size / 2) as f32 - start_pos.0;
        let y = (defender.borrow().location.y + defender.borrow().size.size / 2) as f32 - start_pos.1;
        let dist = (x * x + y * y).sqrt();

        let projectile = attacker.borrow().actor.stats.get_ranged_projectile();
        if let Some(ref projectile) = projectile {
            start_pos.0 -= projectile.get_width_f32() / 2.0;
            start_pos.1 -= projectile.get_height_f32() / 2.0;
        }


        RangedAttackAnimation {
            attacker: Rc::clone(attacker),
            defender: Rc::clone(defender),
            marked_for_removal: false,
            start_time: Instant::now(),
            start_pos,
            cur_pos: (0.0, 0.0),
            vec: (x, y),
            total_time_millis: base_time_millis as f32 * dist,
            has_attacked: false,
            projectile,
        }
    }
}

impl Animation for RangedAttackAnimation {
    fn draw_graphics_mode(&self, renderer: &mut GraphicsRenderer, offset_x: f32, offset_y: f32,
                          scale_x: f32, scale_y: f32, millis: u32) {
        if let Some(ref projectile) = self.projectile {
            let mut draw_list = DrawList::empty_sprite();
            projectile.append_to_draw_list(&mut draw_list, &animation_state::NORMAL,
                                           self.cur_pos.0 + offset_x,
                                           self.cur_pos.1 + offset_y,
                                           projectile.get_width_f32(),
                                           projectile.get_height_f32(), millis);
            draw_list.set_scale(scale_x, scale_y);
            // TODO rotate the projectile
            renderer.draw(draw_list);
        }
    }

    fn update(&mut self, area_state: &mut AreaState, _root: &Rc<RefCell<Widget>>) -> bool {
        if self.marked_for_removal {
            return false;
        }

        let millis = util::get_elapsed_millis(self.start_time.elapsed());
        let frac = millis as f32 / self.total_time_millis;

        if frac > 1.0 {
            if !self.has_attacked {
                let (text, color) = self.attacker.borrow_mut().actor.attack(&self.defender);

                let scale = 1.2;
                area_state.add_feedback_text(text, &self.defender, scale, color);
                self.has_attacked = true;
            }
            false
        } else {
            self.cur_pos = (frac * self.vec.0 + self.start_pos.0,
                            frac * self.vec.1 + self.start_pos.1);
            true
        }
    }

    fn check(&mut self, entity: &Rc<RefCell<EntityState>>) {
        if *self.attacker.borrow() == *entity.borrow() {
            self.marked_for_removal = true;
        }
    }

    fn get_owner(&self) -> &Rc<RefCell<EntityState>> {
        &self.attacker
    }
}
