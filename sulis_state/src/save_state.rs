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
use std::cell::RefCell;
use std::u64;
use std::collections::HashMap;

use sulis_core::util::{Point};
use sulis_rules::Slot;
use sulis_module::{actor::{ActorBuilder, RewardBuilder}};

use {ActorState, EntityState, GameState, ItemState, PropState, prop_state::Interactive, Merchant};
use area_state::{TriggerState};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SaveState {
    pub(crate) party: Vec<usize>,
    pub(crate) selected: Vec<usize>,
    pub(crate) current_area: String,
    pub(crate) areas: HashMap<String, AreaSaveState>,
}

impl SaveState {
    pub fn create() -> SaveState {
        let mut areas = HashMap::new();

        for id in GameState::area_state_ids() {
            areas.insert(id.to_string(), AreaSaveState::new(id));
        }

        let area_state = GameState::area_state();
        let current_area = area_state.borrow().area.id.to_string();

        let mut party = Vec::new();
        for entity in GameState::party().iter() {
            party.push(entity.borrow().index);
        }

        let mut selected = Vec::new();
        for entity in GameState::selected().iter() {
            selected.push(entity.borrow().index);
        }

        SaveState {
            areas,
            current_area,
            party,
            selected,
        }
    }

    pub fn load(self) -> Result<(), Error> {
        GameState::load(self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AreaSaveState {
    pub(crate) on_load_fired: bool,
    pub(crate) entities: Vec<EntitySaveState>,
    pub(crate) props: Vec<PropSaveState>,
    pub(crate) triggers: Vec<TriggerSaveState>,
    pub(crate) merchants: Vec<MerchantSaveState>,
    pub(crate) pc_explored: Vec<u64>,
}

impl AreaSaveState {
    pub fn new(id: String) -> AreaSaveState {
        let area_state = GameState::get_area_state(&id).unwrap();
        let area_state = area_state.borrow();

        let mut pc_explored: Vec<u64> = Vec::new();
        let mut mask: u64 = 1;
        let mut cur_buf: u64 = 0;
        for val in area_state.pc_explored.clone() {
            if val {
                cur_buf += mask;
            }

            if mask == u64::MAX / 2 + 1 {
                mask = 1;
                pc_explored.push(cur_buf);
                cur_buf = 0;
            } else {
                mask = mask * 2;
            }
        }
        if mask != 1 {
            pc_explored.push(cur_buf);
        }

        let on_load_fired = area_state.on_load_fired;

        let mut props = Vec::new();
        for prop_state in area_state.prop_iter() {
            props.push(PropSaveState::new(prop_state));
        }

        let mut triggers = Vec::new();
        for trigger in area_state.triggers.iter() {
            triggers.push(TriggerSaveState::new(trigger));
        }

        let mut merchants = Vec::new();
        for merchant in area_state.merchants.iter() {
            merchants.push(MerchantSaveState::new(merchant));
        }

        let mut entities = Vec::new();
        for entity in area_state.entity_iter() {
            entities.push(EntitySaveState::new(entity));
        }

        AreaSaveState {
            pc_explored,
            on_load_fired,
            props,
            triggers,
            merchants,
            entities,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct PropSaveState {
    pub(crate) id: String,
    pub(crate) interactive: PropInteractiveSaveState,
    pub(crate) location: Point,
    pub(crate) active: bool,
    pub(crate) enabled: bool,
}

impl PropSaveState {
    pub fn new(prop_state: &PropState) -> PropSaveState {
        let location = prop_state.location.to_point();

        use self::PropInteractiveSaveState::*;
        let interactive = match prop_state.interactive {
            Interactive::Not => Not,
            Interactive::Container { ref items, ref loot_to_generate, temporary } => {
                let loot_to_generate = match loot_to_generate {
                    None => None,
                    Some(ref loot_list) => Some(loot_list.id.to_string()),
                };

                let items = items.iter().map(|(qty, ref it)| ItemListEntrySaveState::new(*qty, it)).collect();

                Container {
                    loot_to_generate,
                    temporary,
                    items,
                }
            },
            Interactive::Door { open } => Door {
                open,
            },
        };

        PropSaveState {
            id: prop_state.prop.id.to_string(),
            interactive,
            location,
            active: prop_state.is_active(),
            enabled: prop_state.is_enabled(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub enum PropInteractiveSaveState {
    Not,
    Container {
        loot_to_generate: Option<String>,
        temporary: bool,
        items: Vec<ItemListEntrySaveState>,
    },
    Door {
        open: bool,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemListEntrySaveState {
    pub(crate) quantity: u32,
    pub(crate) item: ItemSaveState,
}

impl ItemListEntrySaveState {
    fn new(quantity: u32, item_state: &ItemState) -> ItemListEntrySaveState {
        ItemListEntrySaveState {
            quantity,
            item: ItemSaveState { id: item_state.item.id.to_string() },
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ItemSaveState {
    pub(crate) id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TriggerSaveState {
    pub(crate) fired: bool,
    pub(crate) enabled: bool,
}

impl TriggerSaveState {
    pub fn new(trigger: &TriggerState) -> TriggerSaveState {
        TriggerSaveState {
            fired: trigger.fired,
            enabled: trigger.enabled,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MerchantSaveState {
    pub(crate) id: String,
    pub(crate) buy_frac: f32,
    pub(crate) sell_frac: f32,
    pub(crate) items: Vec<ItemListEntrySaveState>,
}

impl MerchantSaveState {
    pub fn new(merchant: &Merchant) -> MerchantSaveState {
        let items = merchant.items().iter().map(|(q, ref it)| ItemListEntrySaveState::new(*q, it)).collect();

        MerchantSaveState {
            id: merchant.id.to_string(),
            buy_frac: merchant.buy_frac,
            sell_frac: merchant.sell_frac,
            items,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EntitySaveState {
    pub(crate) index: usize,
    pub(crate) actor_base: Option<ActorBuilder>,
    pub(crate) actor: ActorSaveState,
    pub(crate) location: Point,
    pub(crate) size: String,
    pub(crate) custom_flags: Vec<String>,
    pub(crate) ai_group: Option<usize>,
    pub(crate) ai_active: bool,
}

impl EntitySaveState {
    pub fn new(entity: Rc<RefCell<EntityState>>) -> EntitySaveState {
        let entity = entity.borrow();

        let actor_base = if entity.is_party_member() {
            let actor = &entity.actor.actor;

            let mut levels = HashMap::new();
            for (ref class, level) in actor.levels.iter() {
                levels.insert(class.id.to_string(), *level);
            }

            let reward = match actor.reward {
                None => None,
                Some(ref reward) => {
                    Some(RewardBuilder {
                        xp: reward.xp,
                        loot: reward.loot.as_ref().map(|l| l.id.to_string()),
                        loot_chance: Some(reward.loot_chance),
                    })
                }
            };

            let mut abilities: Vec<String> = Vec::new();
            for owned_ability in actor.abilities.iter() {
                for _ in 0..(owned_ability.level + 1) {
                    abilities.push(owned_ability.ability.id.to_string());
                }
            }

            Some(ActorBuilder {
                id: actor.id.to_string(),
                name: actor.name.to_string(),
                race: actor.race.id.to_string(),
                sex: Some(actor.sex),
                portrait: actor.portrait.as_ref().map(|p| p.id().to_string()),
                attributes: actor.attributes,
                conversation: actor.conversation.as_ref().map(|c| c.id.to_string()),
                faction: Some(actor.faction),
                images: actor.builder_images.clone(),
                hue: actor.hue,
                hair_color: actor.hair_color,
                skin_color: actor.skin_color,
                items: Some(actor.items.iter().map(|i| i.id.to_string()).collect()),
                equipped: Some(actor.to_equip.iter().map(|e| *e as u32).collect()),
                coins: Some(actor.coins),
                levels,
                xp: Some(actor.xp),
                reward,
                abilities,
            })
        } else {
            None
        };

        EntitySaveState {
            index: entity.index,
            actor: ActorSaveState::new(&entity.actor),
            location: entity.location.to_point(),
            size: entity.size.id.clone(),
            custom_flags: entity.custom_flags().map(|s| s.to_string()).collect(),
            ai_group: entity.ai_group(),
            ai_active: entity.is_ai_active(),
            actor_base,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ActorSaveState {
    pub(crate) id: String,
    pub(crate) hp: i32,
    pub(crate) ap: u32,
    pub(crate) overflow_ap: i32,
    pub(crate) xp: u32,
    pub(crate) items: Vec<ItemListEntrySaveState>,
    pub(crate) equipped: Vec<Option<usize>>,
    pub(crate) coins: i32,
    pub(crate) ability_states: HashMap<String, AbilitySaveState>,
}

impl ActorSaveState {
    pub fn new(actor_state: &ActorState) -> ActorSaveState {
        // TODO serialize effects
        let mut equipped = Vec::new();
        for slot in Slot::iter() {
            equipped.push(actor_state.inventory().equipped(slot));
        }

        let mut ability_states = HashMap::new();
        for (id, ref ability_state) in actor_state.ability_states.iter() {
            ability_states.insert(id.to_string(), AbilitySaveState {
                remaining_duration: ability_state.remaining_duration(),
            });
        }

        ActorSaveState {
            id: actor_state.actor.id.to_string(),
            hp: actor_state.hp(),
            ap: actor_state.ap(),
            overflow_ap: actor_state.overflow_ap(),
            xp: actor_state.xp(),
            items: actor_state.inventory().items.iter()
                .map(|(q, ref i)| ItemListEntrySaveState::new(*q, i)).collect(),
            equipped,
            coins: actor_state.inventory().coins(),
            ability_states,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AbilitySaveState {
    pub(crate) remaining_duration: u32,
}
