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
use std::f32;
use std::rc::Rc;

use crate::{EntityState, GameState};
use sulis_core::util;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Formation {
    positions: Vec<(f32, f32)>,
}

impl Default for Formation {
    fn default() -> Self {
        Formation {
            positions: vec![
                (-2.0, 0.0),
                (2.0, 0.0),
                (-2.0, 3.0),
                (2.0, 3.0),
                (-2.0, 6.0),
                (2.0, 6.0),
            ],
        }
    }
}

fn center_of_group(entities: &Vec<Rc<RefCell<EntityState>>>) -> (f32, f32) {
    let mut x = 0.0;
    let mut y = 0.0;

    for entity in entities.iter() {
        let entity = entity.borrow();
        x += entity.location.x as f32;
        y += entity.location.y as f32;
    }

    x /= entities.len() as f32;
    y /= entities.len() as f32;

    (x, y)
}

impl Formation {
    pub fn positions_iter(&self) -> impl Iterator<Item = &(f32, f32)> {
        self.positions.iter()
    }

    pub fn set_position(&mut self, index: usize, pos: (f32, f32)) {
        if index >= self.positions.len() {
            return;
        }

        self.positions[index] = pos;
    }

    pub fn move_group(
        &self,
        entities_to_move: &Vec<Rc<RefCell<EntityState>>>,
        entities_to_ignore: Vec<usize>,
        x_base: f32,
        y_base: f32,
    ) {
        if entities_to_move.len() == 1 {
            GameState::move_towards_point(
                &entities_to_move[0],
                entities_to_ignore,
                x_base,
                y_base,
                0.6,
                None,
            );
            return;
        }

        let (cur_x, cur_y) = center_of_group(entities_to_move);
        let (dir_x, dir_y) = (x_base - cur_x, y_base - cur_y);

        let angle = dir_y.atan2(dir_x);
        let (sin, cos) = angle.sin_cos();

        let to_move = if entities_to_move.len() > self.positions.len() {
            &entities_to_move[0..self.positions.len()]
        } else {
            entities_to_move
        };

        let start_time = ::std::time::Instant::now();
        for index in 0..to_move.len() {
            // first rotate the stored positions by 90 degrees
            let xi = -self.positions[index].1;
            let yi = self.positions[index].0;

            let x = (xi * cos - yi * sin).round() + x_base;
            let y = (xi * sin + yi * cos).round() + y_base;

            for dist_increase in 0..3 {
                let dist = 0.6 + dist_increase as f32 * 1.0;
                if !GameState::can_move_towards_point(
                    &to_move[index],
                    entities_to_ignore.clone(),
                    x,
                    y,
                    dist,
                ) {
                    continue;
                }

                GameState::move_towards_point(
                    &to_move[index],
                    entities_to_ignore.clone(),
                    x,
                    y,
                    dist,
                    None,
                );
                break;
            }
        }

        debug!(
            "Formation move complete in {} secs",
            util::format_elapsed_secs(start_time.elapsed())
        );
    }
}
