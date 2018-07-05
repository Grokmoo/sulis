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

use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::io::Error;

use sulis_rules::BonusList;
use sulis_core::image::Image;
use sulis_core::resource::{ResourceBuilder, ResourceSet};
use sulis_core::util::{invalid_data_error, unable_to_create_error};
use sulis_core::serde_yaml;

use {Actor, Module, PrereqList, PrereqListBuilder};

#[derive(Debug)]
pub struct Active {
    pub script: String,
    pub ap: u32,
    pub duration: Duration,
    pub group: String,
    pub cooldown: u32,
    pub short_description: String,
}

#[derive(Debug)]
pub struct Ability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Rc<Image>,
    pub active: Option<Active>,
    pub bonuses: BonusList,
    pub prereqs: Option<PrereqList>,
    pub upgrades: Vec<String>,
}

impl Eq for Ability { }

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
        let icon = match ResourceSet::get_image(&builder.icon) {
            None => {
                warn!("No image found for icon '{}'", builder.icon);
                return unable_to_create_error("ability", &builder.id);
            },
            Some(icon) => icon
        };

        let active = match builder.active {
            None => None,
            Some(active) => {
                let script = match module.scripts.get(&active.script) {
                    None => {
                        warn!("No script found with id '{}'", active.script);
                        return unable_to_create_error("ability", &builder.id);
                    }, Some(ref script) => script.to_string(),
                };

                let cooldown = match active.cooldown {
                    None => {
                        match active.duration {
                            Duration::Rounds(c) => c,
                            Duration::Mode | Duration::Instant => 0,
                        }
                    }, Some(c) => c,
                };

                Some(Active {
                    script,
                    ap: active.ap,
                    duration: active.duration,
                    cooldown,
                    group: active.group,
                    short_description: active.short_description,
                })
            },
        };

        let prereqs = match builder.prereqs {
            None => None,
            Some(prereqs) => Some(PrereqList::new(prereqs, module)?),
        };

        Ok(Ability {
            id: builder.id,
            name: builder.name,
            description: builder.description,
            icon,
            active,
            bonuses: builder.bonuses.unwrap_or_default(),
            prereqs,
            upgrades: builder.upgrades.unwrap_or_default(),
        })
    }

    pub fn meets_prereqs(&self, actor: &Rc<Actor>) -> bool {
        match self.prereqs {
            None => true,
            Some(ref prereqs) => prereqs.meets(actor),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum Duration {
    Rounds(u32),
    Mode,
    Instant,
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
    pub upgrades: Option<Vec<String>>,
}

impl ResourceBuilder for AbilityBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_yaml(data: &str) -> Result<AbilityBuilder, Error> {
        let resource: Result<AbilityBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => invalid_data_error(&format!("{}", error))
        }
    }
}
