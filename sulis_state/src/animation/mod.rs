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

use std::time::Duration;
use std::cmp;
use std::rc::Rc;
use std::cell::RefCell;

mod entity_subpos_animation;
pub use self::entity_subpos_animation::EntitySubposAnimation;

mod melee_attack_animation;
pub use self::melee_attack_animation::MeleeAttackAnimation;

mod move_animation;
pub use self::move_animation::MoveAnimation;

pub mod particle_generator;
pub use self::particle_generator::ParticleGenerator;

mod ranged_attack_animation;
pub use self::ranged_attack_animation::RangedAttackAnimation;

mod wait_animation;
pub use self::wait_animation::WaitAnimation;

use {EntityState, ScriptCallback};

use sulis_core::io::GraphicsRenderer;
use sulis_core::ui::Widget;
use sulis_core::util;

pub trait Animation {
    fn update(&mut self, root: &Rc<RefCell<Widget>>) -> bool;

    fn mark_for_removal(&mut self) {}

    fn get_owner(&self) -> &Rc<RefCell<EntityState>>;

    fn draw_graphics_mode(&self, _renderer: &mut GraphicsRenderer, _offset_x: f32, _offset_y: f32,
                          _scale_x: f32, _scale_y: f32, _millis: u32) { }

    fn set_callback(&mut self, callback: Option<Box<ScriptCallback>>);
}

/// Helper function to return the number of frames elapsed for the 'elapsed'
/// duration.  This is equal to the duration divided by 'frame_time'.  The
/// value is capped by 'max_index'
pub fn get_current_frame(elapsed: Duration, frame_time:
                         u32, max_index: usize) -> usize {
    let millis = util::get_elapsed_millis(elapsed);

    let frame_index = (millis / frame_time) as usize;

    cmp::min(frame_index, max_index)
}
