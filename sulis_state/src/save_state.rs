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

use std::u64;
use std::collections::HashMap;

use sulis_core::util::Point;

use {GameState, ItemState, PropState, prop_state::Interactive, Merchant};
use area_state::{TriggerState};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveState {
    areas: HashMap<String, AreaSaveState>,
    current_area: String,
}

impl SaveState {
    pub fn create() -> SaveState {
        let mut areas = HashMap::new();

        for id in GameState::area_state_ids() {
            areas.insert(id.to_string(), AreaSaveState::new(id));
        }

        let area_state = GameState::area_state();
        let current_area = area_state.borrow().area.id.to_string();

        SaveState {
            areas,
            current_area,
        }
    }

    pub fn load(&self) {

    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AreaSaveState {
    on_load_fired: bool,
    props: Vec<PropSaveState>,
    triggers: Vec<TriggerSaveState>,
    merchants: Vec<MerchantSaveState>,
    pc_explored: Vec<u64>,
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

        AreaSaveState {
            pc_explored,
            on_load_fired,
            props,
            triggers,
            merchants,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropSaveState {
    id: String,
    interactive: PropInteractiveSaveState,
    location: Point,
    active: bool,
    enabled: bool,
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemListEntrySaveState {
    quantity: u32,
    item: ItemSaveState,
}

impl ItemListEntrySaveState {
    fn new(quantity: u32, item_state: &ItemState) -> ItemListEntrySaveState {
        ItemListEntrySaveState {
            quantity,
            item: ItemSaveState { id: item_state.item.id.to_string() },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemSaveState {
    id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TriggerSaveState {
    fired: bool,
    enabled: bool,
}

impl TriggerSaveState {
    pub fn new(trigger: &TriggerState) -> TriggerSaveState {
        TriggerSaveState {
            fired: trigger.fired,
            enabled: trigger.enabled,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MerchantSaveState {
    id: String,
    buy_frac: f32,
    sell_frac: f32,
    items: Vec<ItemListEntrySaveState>,
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
