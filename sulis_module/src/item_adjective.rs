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

use std::io::Error;
use std::rc::Rc;

use crate::rules::{AttackBonuses, BonusList};
use crate::{PrereqList, PrereqListBuilder};
use sulis_core::image::Image;
use sulis_core::resource::deserialize_image;

/// An adjective is a modifier that affects the stats of
/// an item in a given way.  Items can have zero, one, or
/// many adjectives.
#[derive(Debug)]
pub struct ItemAdjective {
    pub id: String,
    pub name: String,

    pub item_status_icon: Rc<dyn Image>,

    pub name_prefix: Option<String>,
    pub name_postfix: Option<String>,
    pub value_modifier: Option<f32>,
    pub value_add: Option<i32>,
    pub bonus_modifier: Option<f32>,
    pub penalty_modifier: Option<f32>,
    pub attack_damage_modifier: Option<f32>,
    pub attack_bonus_modifier: Option<f32>,
    pub attack_penalty_modifier: Option<f32>,

    pub bonuses: BonusList,

    pub attack_bonuses: AttackBonuses,

    pub prereqs: Option<PrereqList>
}

impl ItemAdjective {
    pub fn new(builder: ItemAdjectiveBuilder) -> Result<ItemAdjective, Error> {
        let prereqs = match builder.prereqs {
            None => None,
            Some(prereqs) => Some(PrereqList::new(prereqs)?),
        };

        Ok(ItemAdjective {
            id: builder.id,
            name: builder.name,
            item_status_icon: builder.item_status_icon,
            name_prefix: builder.name_prefix,
            name_postfix: builder.name_postfix,
            value_modifier: builder.value_modifier,
            value_add: builder.value_add,
            bonus_modifier: builder.bonus_modifier,
            penalty_modifier: builder.penalty_modifier,
            attack_damage_modifier: builder.attack_damage_modifier,
            attack_bonus_modifier: builder.attack_bonus_modifier,
            attack_penalty_modifier: builder.attack_penalty_modifier,
            bonuses: builder.bonuses,
            attack_bonuses: builder.attack_bonuses,
            prereqs,
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemAdjectiveBuilder {
    pub id: String,
    pub name: String,

    #[serde(deserialize_with = "deserialize_image")]
    pub item_status_icon: Rc<dyn Image>,

    pub name_prefix: Option<String>,
    pub name_postfix: Option<String>,
    pub value_modifier: Option<f32>,
    pub value_add: Option<i32>,
    pub bonus_modifier: Option<f32>,
    pub penalty_modifier: Option<f32>,
    pub attack_damage_modifier: Option<f32>,
    pub attack_bonus_modifier: Option<f32>,
    pub attack_penalty_modifier: Option<f32>,

    #[serde(default)]
    pub bonuses: BonusList,

    #[serde(default)]
    pub attack_bonuses: AttackBonuses,

    pub prereqs: Option<PrereqListBuilder>
}

impl PartialEq for ItemAdjective {
    fn eq(&self, other: &ItemAdjective) -> bool {
        self.id == other.id
    }
}
