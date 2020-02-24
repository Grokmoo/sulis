//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2020 Jared Stephen
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

use sulis_core::util::{gen_rand, Offset};

const CLAMP_SHAKES: u32 = 4;
const TOTAL_SHAKES: u32 = 7;
const SHAKE_MILLIS: u32 = 110;

pub struct ShakeResult {
    pub done: bool,
    pub scroll: Option<Offset>,
}

impl ShakeResult {
    fn done(shake: &ScreenShake) -> ShakeResult {
        ShakeResult {
            done: true,
            scroll: Some(Offset {
                x: -shake.last_scroll.x,
                y: -shake.last_scroll.y,
            })
        }
    }

    fn shake(scroll: Offset) -> ShakeResult {
        ShakeResult {
            done: false,
            scroll: Some(scroll),
        }
    }

    fn none() -> ShakeResult {
        ShakeResult {
            done: false,
            scroll: None,
        }
    }
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ScreenShake {
    total_shakes: u32,
    last_millis: u32,
    last_scroll: Offset,
}

impl ScreenShake {
    pub fn new() -> ScreenShake {
        ScreenShake {
            total_shakes: 0,
            last_millis: 0,
            last_scroll: Offset { x: 1.0, y: 0.0 },
        }
    }

    pub fn shake(&mut self, delta_millis: u32) -> ShakeResult {
        self.last_millis += delta_millis;

        if self.last_millis >= SHAKE_MILLIS {
            if self.total_shakes >= TOTAL_SHAKES {
                return ShakeResult::done(&self);
            }

            self.total_shakes += 1;
            self.last_millis -= SHAKE_MILLIS;

            let mut scroll = Offset {
                x: -1.0 * self.last_scroll.x.signum() * gen_rand(1.0, 1.8) - self.last_scroll.x,
                y: gen_rand(-0.1, 0.1) - self.last_scroll.y,
            };

            if self.total_shakes > CLAMP_SHAKES {
                let clamp_factor = 1.0 - (self.total_shakes - CLAMP_SHAKES) as f32 * 0.2;
                scroll.x *= clamp_factor;
            }

            self.last_scroll = scroll;
            return ShakeResult::shake(scroll);
        }

        ShakeResult::none()
    }
}
