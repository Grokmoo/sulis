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

use std::io::{Error};
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::rc::Rc;

use sulis_core::util::unable_to_create_error;
use sulis_core::image::Image;
use sulis_core::resource::{ResourceSet};
use sulis_rules::{ItemKind, bonus::AttackBuilder, BonusList, Slot};

use crate::{Actor, ImageLayer, Module, ability::{AIData, Duration}, PrereqList, PrereqListBuilder, ItemAdjective};

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Equippable {
    pub slot: Slot,
    pub alternate_slot: Option<Slot>,
    pub blocks_slot: Option<Slot>,
    pub bonuses: BonusList,
    pub attack: Option<AttackBuilder>,
}

#[derive(Debug, Clone)]
pub struct Usable {
    pub script: String,
    pub ap: u32,
    pub duration: Duration,
    pub consumable: bool,
    pub short_description: String,
    pub ai: AIData,
    pub use_in_slot: bool,
}

#[derive(Debug)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub kind: ItemKind,
    pub icon: Rc<Image>,
    pub equippable: Option<Equippable>,
    pub prereqs: Option<PrereqList>,
    image: Option<HashMap<ImageLayer, Rc<Image>>>,
    alternate_image: Option<HashMap<ImageLayer, Rc<Image>>>,
    pub value: i32,
    pub weight: i32,
    pub usable: Option<Usable>,

    // original values from before any adjectives are applied
    pub original_id: String,
    original_value: i32,
    original_equippable: Option<Equippable>,

    // adjectives that were present in the item builder when this
    // item is defined
    builder_adjectives: Vec<Rc<ItemAdjective>>,

    // adjectives that were not in the item builder but were created
    // dynamically
    pub added_adjectives: Vec<Rc<ItemAdjective>>,
}

fn build_hash_map(id: &str, input: Option<HashMap<ImageLayer, String>>)
    -> Result<Option<HashMap<ImageLayer, Rc<Image>>>, Error> {
    match input {
        Some(input_images) => {
            let mut output = HashMap::new();
            for (layer, image_str) in input_images {
                let image = match ResourceSet::get_image(&image_str) {
                    None => {
                        warn!("No image found for image '{}'", image_str);
                        return unable_to_create_error("item", id);
                    }, Some(image) => image,
                };

                output.insert(layer, image);
            }

            Ok(Some(output))
        }, None => Ok(None),
    }
}

impl Item {
    pub fn clone_with_adjectives(item: &Rc<Item>, mut adjectives: Vec<Rc<ItemAdjective>>,
                                 new_id: String) -> Item {
        assert!(adjectives.len() > 0);

        let mut added_adjectives = item.added_adjectives.clone();
        added_adjectives.append(&mut adjectives);

        let mut all_adjectives = item.builder_adjectives.clone();
        all_adjectives.extend_from_slice(&added_adjectives);

        let (equippable, value) = apply_adjectives(&item.original_equippable,
                                                   item.original_value, &all_adjectives);

        let mut name = String::new();
        for adj in all_adjectives.iter() {
            if let Some(ref prefix) = adj.name_prefix {
                name.push_str(prefix);
            }
        }
        name.push_str(&item.name);
        for adj in all_adjectives.iter() {
            if let Some(ref postfix) = adj.name_postfix {
                name.push_str(postfix);
            }
        }

        Item {
            id: new_id,
            icon: Rc::clone(&item.icon),
            image: item.image.clone(),
            kind: item.kind,
            alternate_image: item.alternate_image.clone(),
            name,
            equippable,
            value,
            weight: item.weight,
            usable: item.usable.clone(),
            prereqs: item.prereqs.clone(),
            original_id: item.original_id.clone(),
            original_value: item.original_value,
            original_equippable: item.original_equippable.clone(),
            builder_adjectives: item.builder_adjectives.clone(),
            added_adjectives,
        }
    }

    pub fn new(builder: ItemBuilder, module: &Module) -> Result<Item, Error> {
        let icon = match ResourceSet::get_image(&builder.icon) {
            None => {
                warn!("No image found for icon '{}'", builder.icon);
                return unable_to_create_error("item", &builder.id);
            },
            Some(icon) => icon
        };


        if let &Some(ref equippable) = &builder.equippable {
            if let Some(ref attack) = equippable.attack {
                if attack.damage.kind.is_none() {
                    warn!("Kind must be specified for attack damage.");
                    return unable_to_create_error("item", &builder.id);
                }
            }
        }

        let images = build_hash_map(&builder.id, builder.image)?;
        let alt_images = build_hash_map(&builder.id, builder.alternate_image)?;

        let usable = match builder.usable {
            None => None,
            Some(usable) => {
                let script = match module.scripts.get(&usable.script) {
                    None => {
                        warn!("No script found with id '{}'", usable.script);
                        return unable_to_create_error("item", &builder.id);
                    }, Some(ref script) => script.to_string(),
                };

                Some(Usable {
                    script,
                    consumable: usable.consumable,
                    duration: usable.duration,
                    ap: usable.ap,
                    short_description: usable.short_description,
                    ai: usable.ai,
                    use_in_slot: usable.use_in_slot,
                })
            }
        };

        let prereqs = match builder.prereqs {
            None => None,
            Some(list) => Some(PrereqList::new(list, module)?),
        };

        let mut adjectives = Vec::new();
        for adj_id in builder.adjectives {
            let adjective = match module.item_adjectives.get(&adj_id) {
                None => {
                    warn!("No item adjective found with id '{}'", adj_id);
                    return unable_to_create_error("item", &builder.id);
                }, Some(ref adj) => Rc::clone(adj),
            };
            adjectives.push(adjective);
        }

        let (equippable, value) = apply_adjectives(&builder.equippable, builder.value as i32, &adjectives);

        Ok(Item {
            id: builder.id.clone(),
            icon: icon,
            image: images,
            kind: builder.kind.unwrap_or(ItemKind::Other),
            alternate_image: alt_images,
            name: builder.name,
            equippable,
            value,
            weight: builder.weight as i32,
            usable,
            prereqs,
            original_id: builder.id,
            original_value: builder.value as i32,
            original_equippable: builder.equippable,
            builder_adjectives: adjectives,
            added_adjectives: Vec::new(),
        })
    }

    pub fn adjective_icons(&self) -> Vec<String> {
        self.builder_adjectives.iter().chain(self.added_adjectives.iter())
            .map(|adj| adj.item_status_icon.id()).collect()
    }

    pub fn meets_prereqs(&self, actor: &Rc<Actor>) -> bool {
        match self.prereqs {
            None => true,
            Some(ref prereqs) => prereqs.meets(actor),
        }
    }

    pub fn alt_image_iter(&self) -> Option<Iter<ImageLayer, Rc<Image>>> {
        match self.alternate_image {
            None => None,
            Some(ref image) => Some(image.iter()),
        }
    }

    pub fn image_iter(&self) -> Option<Iter<ImageLayer, Rc<Image>>> {
        match self.image {
            None => None,
            Some(ref image) => Some(image.iter()),
        }
    }

    pub fn is_armor(&self) -> bool {
        match self.kind {
            ItemKind::Armor { .. } => true,
            _ => false,
        }
    }

    pub fn is_weapon(&self) -> bool {
        match self.kind {
            ItemKind::Weapon { .. } => true,
            _ => false,
        }
    }
}

fn apply_adjectives(equippable: &Option<Equippable>, value: i32,
                    adjectives: &Vec<Rc<ItemAdjective>>) -> (Option<Equippable>, i32) {
    // first, add bonuses from adjectives and accumulate modifiers
    let mut value_modifier = 1.0;
    let mut bonus_modifier = 1.0;
    let mut penalty_modifier = 1.0;
    let mut attack_damage_mod = 1.0;
    let mut attack_bonus_mod = 1.0;
    let mut attack_penalty_mod = 1.0;

    let mut equippable = equippable.clone();
    for adjective in adjectives.iter() {
        value_modifier += adjective.value_modifier.unwrap_or(1.0) - 1.0;
        bonus_modifier += adjective.bonus_modifier.unwrap_or(1.0) - 1.0;
        penalty_modifier += adjective.penalty_modifier.unwrap_or(1.0) - 1.0;
        attack_damage_mod += adjective.attack_damage_modifier.unwrap_or(1.0) - 1.0;
        attack_bonus_mod += adjective.attack_bonus_modifier.unwrap_or(1.0) - 1.0;
        attack_penalty_mod += adjective.attack_penalty_modifier.unwrap_or(1.0) - 1.0;

        if let Some(ref mut equippable) = equippable {
            for bonus in adjective.bonuses.iter() {
                equippable.bonuses.add(bonus.clone());
            }

            if let Some(ref mut attack) = equippable.attack {
                attack.bonuses.add(&adjective.attack_bonuses);
            }
        }
    }

    // now apply accumulated bonuses and penalties
    if let Some(ref mut equippable) = equippable {
        equippable.bonuses.merge_duplicates();

        if let Some(ref mut attack) = equippable.attack {
            if attack_bonus_mod != 1.0 || attack_penalty_mod != 1.0 {
                attack.bonuses.apply_modifier(attack_penalty_mod, attack_bonus_mod);
            }

            if attack_damage_mod != 1.0 {
                let damage = attack.damage.mult_f32(attack_damage_mod);
                attack.damage = damage;
            }
        }

        equippable.bonuses.apply_modifiers(penalty_modifier, bonus_modifier);
    }

    let value = (value as f32 * value_modifier).round() as i32;

    (equippable, value)
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UsableBuilder {
    pub script: String,
    pub ap: u32,
    pub duration: Duration,
    pub consumable: bool,
    pub short_description: String,
    pub ai: AIData,
    #[serde(default="bool_true")]
    pub use_in_slot: bool,
}

fn bool_true() -> bool { true }

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemBuilder {
    pub id: String,
    pub name: String,
    kind: Option<ItemKind>,
    pub icon: String,
    pub equippable: Option<Equippable>,
    pub image: Option<HashMap<ImageLayer, String>>,
    pub alternate_image: Option<HashMap<ImageLayer, String>>,
    prereqs: Option<PrereqListBuilder>,
    value: u32,
    weight: u32,
    usable: Option<UsableBuilder>,
    #[serde(default)]
    adjectives: Vec<String>,
}

pub fn format_item_value(value: i32) -> String {
    let display_factor = Module::rules().item_value_display_factor;

    let value_f = value as f32 / display_factor;
    format!("{:.*}", 1, value_f)
}

pub fn format_item_weight(weight: i32) -> String {
    let display_factor = Module::rules().item_weight_display_factor;

    let weight_f = weight as f32 / display_factor;
    format!("{:.*}", 2, weight_f)
}
