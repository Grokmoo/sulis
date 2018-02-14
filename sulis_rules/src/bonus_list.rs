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

use {Armor, Damage};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BonusList {
    pub armor: Option<Armor>,
    pub bonus_damage: Option<Damage>,
    pub base_damage: Option<Damage>,
    pub base_reach: Option<f32>,
    pub bonus_reach: Option<f32>,
    pub initiative: Option<i32>,
    pub hit_points: Option<i32>,
    pub accuracy: Option<i32>,
    pub defense: Option<i32>,
    pub fortitude: Option<i32>,
    pub reflex: Option<i32>,
    pub will: Option<i32>,
}

