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

use crate::animation::particle_generator::Param;
use crate::{EntityState, RcRfc};

pub(in crate::animation) fn update(
    x: &mut Param,
    y: &mut Param,
    owner: &RcRfc<EntityState>,
    millis: u32,
) {
    let secs = millis as f32 / 1000.0;
    let v_term = secs;
    let a_term = secs * secs;
    let j_term = secs * secs * secs;

    x.update(v_term, a_term, j_term);
    y.update(v_term, a_term, j_term);

    owner.borrow_mut().sub_pos = (x.value, y.value);
}

pub(in crate::animation) fn cleanup(owner: &RcRfc<EntityState>) {
    owner.borrow_mut().sub_pos = (0.0, 0.0);
}
