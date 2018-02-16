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

use sulis_core::image::Image;
use sulis_core::resource::ResourceSet;
use bonus_list::{AttackBuilder, AttackKindBuilder};
use {Armor, DamageKind, DamageList, StatList};

use AttackKind::*;

pub struct Attack {
    pub damage: DamageList,
    pub kind: AttackKind,
}

impl Attack {
    pub fn new(builder: &AttackBuilder, stats: &StatList) -> Attack {
        let damage = DamageList::new(builder.damage, &stats.bonus_damage);

        let kind = match builder.kind {
            AttackKindBuilder::Melee { reach } =>
                Melee { reach: reach + stats.bonus_reach },
            AttackKindBuilder::Ranged { range, ref projectile } => {
                let image = match ResourceSet::get_image(projectile) {
                    None => {
                        warn!("No image found for projectile '{}'", projectile);
                        ResourceSet::get_empty_image()
                    }, Some(image) => image,
                };
                Ranged { range: range + stats.bonus_range, projectile: image }
            }
        };

        Attack {
            damage,
            kind,
        }
    }

    pub fn is_melee(&self) -> bool {
        match self.kind {
            Melee { .. } => true,
            Ranged { .. } => false,
        }
    }

    pub fn is_ranged(&self) -> bool {
        match self.kind {
            Melee { .. } => false,
            Ranged { .. } => true,
        }
    }

    pub fn get_ranged_projectile(&self) -> Option<Rc<Image>> {
        match self.kind {
            Melee { .. } => None,
            Ranged { ref projectile, .. } => Some(Rc::clone(projectile))
        }
    }

    pub fn mult(&self, multiplier: f32) -> Attack {
        Attack {
            damage: self.damage.mult(multiplier),
            kind: self.kind.clone(),
        }
    }

    pub fn roll_damage(&self, armor: &Armor,
                       multiplier: f32) -> Vec<(DamageKind, u32)> {
        self.damage.roll(armor, multiplier)
    }

    // Returns the distance that this attack can reach
    pub fn distance(&self) -> f32 {
        match self.kind {
            Melee { reach } => reach,
            Ranged { range, .. } => range,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AttackKind {
    Melee { reach: f32 },
    Ranged { range: f32, projectile: Rc<Image> },
}
