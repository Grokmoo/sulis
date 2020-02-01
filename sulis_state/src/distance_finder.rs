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

use crate::{EntityState, PropState};
use sulis_core::util::Point;
use sulis_module::area::Transition;

pub trait Locatable {
    fn size(&self) -> (f32, f32);
    fn pos(&self) -> (f32, f32);
}

impl Locatable for EntityState {
    fn size(&self) -> (f32, f32) {
        (self.size.width as f32, self.size.height as f32)
    }

    fn pos(&self) -> (f32, f32) {
        (self.location.x as f32, self.location.y as f32)
    }
}

impl Locatable for PropState {
    fn size(&self) -> (f32, f32) {
        (self.prop.size.width as f32, self.prop.size.height as f32)
    }

    fn pos(&self) -> (f32, f32) {
        (self.location.x as f32, self.location.y as f32)
    }
}

impl Locatable for Transition {
    fn size(&self) -> (f32, f32) {
        (self.size.width as f32, self.size.height as f32)
    }

    fn pos(&self) -> (f32, f32) {
        (self.from.x as f32, self.from.y as f32)
    }
}

impl Locatable for Point {
    fn size(&self) -> (f32, f32) {
        (1.0, 1.0)
    }

    fn pos(&self) -> (f32, f32) {
        (self.x as f32, self.y as f32)
    }
}

impl Locatable for (f32, f32) {
    fn size(&self) -> (f32, f32) {
        (1.0, 1.0)
    }

    fn pos(&self) -> (f32, f32) {
        (self.0, self.1)
    }
}

pub fn center_i32<T: Locatable>(target: &T) -> (i32, i32) {
    let (x, y) = center(target);
    (x as i32, y as i32)
}

pub fn center<T: Locatable>(target: &T) -> (f32, f32) {
    let (x, y) = target.pos();
    let (w, h) = target.size();
    (x + w / 2.0, y + h / 2.0)
}

pub fn dist(parent: &impl Locatable, target: &impl Locatable) -> f32 {
    let (cx, cy) = center(parent);

    let (tx, ty) = center(target);
    let (w, h) = target.size();

    // closest distance from cx, cy to axis aligned rect centered
    // on tx, ty

    let mut dx = (cx - tx).abs() - w / 2.0;
    let mut dy = (cy - ty).abs() - h / 2.0;

    if dx < 0.0 {
        dx = 0.0;
    }
    if dy < 0.0 {
        dy = 0.0;
    }

    dx.hypot(dy)
}

pub fn is_within(parent: &impl Locatable, target: &impl Locatable, max_dist: f32) -> bool {
    dist(parent, target) <= max_dist
}

pub fn is_within_attack_dist<T: Locatable>(parent: &EntityState, target: &T) -> bool {
    let dist = parent.actor.stats.attack_distance();
    is_within(parent, target, dist)
}

pub fn is_within_touch_dist<T: Locatable>(parent: &EntityState, target: &T) -> bool {
    let dist = parent.actor.stats.touch_distance();
    is_within(parent, target, dist)
}

pub fn is_threat(attacker: &EntityState, defender: &EntityState) -> bool {
    let a = attacker;
    let d = defender;

    if !a.actor.stats.attack_is_melee() || a.actor.stats.attack_disabled || a.actor.is_dead() {
        return false;
    }

    if !a.is_hostile(d) {
        return false;
    }

    is_within_attack_dist(a, d)
}

pub fn can_attack(attacker: &EntityState, defender: &EntityState) -> bool {
    let a = attacker;
    let d = defender;

    if !a.actor.has_ap_to_attack() || a.actor.stats.attack_disabled || a.actor.is_dead() {
        return false;
    }

    if !a.is_hostile(d) {
        return false;
    }

    is_within_attack_dist(a, d)
}
