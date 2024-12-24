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

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Error;
use std::rc::Rc;

use serde::Deserialize;

use crate::rules::{BonusList, StatList};
use sulis_core::image::Image;
use sulis_core::resource::ResourceSet;
use sulis_core::util::unable_to_create_error;

use crate::{Actor, Module, PrereqList, PrereqListBuilder};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbilityGroup {
    pub index: usize,
}

impl AbilityGroup {
    pub fn new(module: &Module, group_id: &str) -> Option<AbilityGroup> {
        for (index, other_group_id) in module
            .rules
            .as_ref()
            .unwrap()
            .ability_groups
            .iter()
            .enumerate()
        {
            if other_group_id == group_id {
                return Some(AbilityGroup { index });
            }
        }
        None
    }

    pub fn name(&self) -> String {
        Module::rules().ability_groups[self.index].clone()
    }
}

#[derive(Debug)]
pub struct Active {
    pub script: String,
    pub ap: u32,
    pub duration: Duration,
    pub group: AbilityGroup,
    pub cooldown: u32,
    pub short_description: String,
    pub ai: AIData,
    pub range: Range,
    pub range_increases_with: Option<RangeIncreaseWith>,
    pub class_stats: HashMap<String, HashMap<String, u32>>,
    pub combat_only: bool,
    pub requires_melee: bool,
    pub requires_shield: bool,
    pub requires_ranged: bool,
    pub requires_active_mode: Vec<String>,
}

#[derive(Debug)]
pub struct Ability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Rc<dyn Image>,
    pub active: Option<Active>,
    pub bonuses: BonusList,
    pub prereqs: Option<PrereqList>,
    pub upgrades: Vec<Upgrade>,
}

impl Eq for Ability {}

impl Hash for Ability {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Ability {
    fn eq(&self, other: &Ability) -> bool {
        self.id == other.id
    }
}

impl Ability {
    pub fn new(builder: AbilityBuilder, module: &Module) -> Result<Ability, Error> {
        let icon = match ResourceSet::image(&builder.icon) {
            None => {
                warn!("No image found for icon '{}'", builder.icon);
                return unable_to_create_error("ability", &builder.id);
            }
            Some(icon) => icon,
        };

        let active = match builder.active {
            None => None,
            Some(active) => {
                if !module.scripts.contains_key(&active.script) {
                    warn!("No script found with id '{}'", active.script);
                    return unable_to_create_error("ability", &builder.id);
                }

                let cooldown = match active.cooldown {
                    None => match active.duration {
                        Duration::Rounds(c) => c,
                        Duration::Mode | Duration::Instant | Duration::Permanent => 0,
                    },
                    Some(c) => c,
                };

                let group = match AbilityGroup::new(module, &active.group) {
                    None => {
                        warn!("Unable to find ability group '{}'", active.group);
                        return unable_to_create_error("ability", &builder.id);
                    }
                    Some(group) => group,
                };

                Some(Active {
                    script: active.script,
                    ap: active.ap,
                    duration: active.duration,
                    cooldown,
                    group,
                    short_description: active.short_description,
                    ai: active.ai,
                    range: active.range,
                    range_increases_with: active.range_increases_with,
                    class_stats: active.class_stats,
                    combat_only: active.combat_only,
                    requires_melee: active.requires_melee,
                    requires_shield: active.requires_shield,
                    requires_ranged: active.requires_ranged,
                    requires_active_mode: active.requires_active_mode,
                })
            }
        };

        let prereqs = match builder.prereqs {
            None => None,
            Some(prereqs) => Some(PrereqList::new(prereqs)?),
        };

        let mut bonuses = builder.bonuses.unwrap_or_default();
        bonuses.merge_duplicates();

        Ok(Ability {
            id: builder.id,
            name: builder.name,
            description: builder.description,
            icon,
            active,
            bonuses,
            prereqs,
            upgrades: builder.upgrades.unwrap_or_default(),
        })
    }

    pub fn add_bonuses_to(&self, level: u32, stats: &mut StatList) {
        stats.add(&self.bonuses);
        let mut index = 1;
        for upgrade in self.upgrades.iter() {
            if index > level {
                break;
            }

            if let Some(ref bonuses) = upgrade.bonuses {
                stats.add(bonuses);
            }

            index += 1;
        }
    }

    pub fn meets_prereqs(&self, actor: &Rc<Actor>) -> bool {
        match self.prereqs {
            None => true,
            Some(ref prereqs) => prereqs.meets(actor),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum Duration {
    Rounds(u32),
    Mode,
    Instant,
    Permanent,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Upgrade {
    pub description: String,
    pub bonuses: Option<BonusList>,

    #[serde(default)]
    pub range_increase: f32,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RangeIncreaseWith {
    pub ability: String,
    pub amount: f32,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActiveBuilder {
    script: String,
    ap: u32,
    duration: Duration,
    group: String,
    cooldown: Option<u32>,
    short_description: String,

    #[serde(default = "range_none")]
    range: Range,

    range_increases_with: Option<RangeIncreaseWith>,

    ai: AIData,

    #[serde(default)]
    class_stats: HashMap<String, HashMap<String, u32>>,

    #[serde(default)]
    combat_only: bool,

    #[serde(default)]
    requires_melee: bool,

    #[serde(default)]
    requires_ranged: bool,

    #[serde(default)]
    requires_shield: bool,

    #[serde(default)]
    requires_active_mode: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AIData {
    pub priority: u32,

    #[serde(default = "default_target")]
    pub target: AITarget,
    pub kind: AIKind,
    pub group: AIGroup,
    pub range: AIRange,
    pub on_activate_fn: Option<String>,
}

impl AIData {
    pub fn target(&self) -> String {
        format!("{:?}", self.target)
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn kind(&self) -> String {
        format!("{:?}", self.kind)
    }

    pub fn group(&self) -> String {
        format!("{:?}", self.group)
    }

    pub fn range(&self) -> String {
        format!("{:?}", self.range)
    }
}

fn default_target() -> AITarget { AITarget::Entity }

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum AITarget {
    Entity,
    EmptyGround,
    AnyGround,
}

impl AITarget {
    pub fn unwrap_from_str(s: &str) -> AITarget {
        match s {
            "Entity" => AITarget::Entity,
            "EmptyGround" => AITarget::EmptyGround,
            "AnyGround" => AITarget::AnyGround,
            _ => {
                warn!("Invalid AI target string '{}'", s);
                AITarget::Entity
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum AIKind {
    Damage,
    Heal,
    Buff,
    Debuff,
    Summon,
    Special,
}

impl AIKind {
    pub fn unwrap_from_str(s: &str) -> AIKind {
        match s {
            "Damage" => AIKind::Damage,
            "Heal" => AIKind::Heal,
            "Buff" => AIKind::Buff,
            "Debuff" => AIKind::Debuff,
            "Summon" => AIKind::Summon,
            "Special" => AIKind::Special,
            _ => {
                warn!("Invalid AI kind string '{}'", s);
                AIKind::Damage
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum AIGroup {
    Single,
    Multiple,
}

impl AIGroup {
    pub fn unwrap_from_str(s: &str) -> AIGroup {
        match s {
            "Single" => AIGroup::Single,
            "Multiple" => AIGroup::Multiple,
            _ => {
                warn!("Invalid AI group string '{}'", s);
                AIGroup::Single
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum AIRange {
    Personal,
    Touch,
    Attack,
    Short,
    Visible,
}

impl AIRange {
    pub fn unwrap_from_str(s: &str) -> AIRange {
        match s {
            "Personal" => AIRange::Personal,
            "Touch" => AIRange::Touch,
            "Attack" => AIRange::Attack,
            "Short" => AIRange::Short,
            "Visible" => AIRange::Visible,
            _ => {
                warn!("Invalid AI range string '{}'", s);
                AIRange::Personal
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AbilityBuilder {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub active: Option<ActiveBuilder>,
    pub bonuses: Option<BonusList>,
    pub prereqs: Option<PrereqListBuilder>,
    pub upgrades: Option<Vec<Upgrade>>,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Range {
    None,
    Personal,
    Touch,
    Attack,
    Radius(f32),
    Visible,
}

fn range_none() -> Range {
    Range::None
}
