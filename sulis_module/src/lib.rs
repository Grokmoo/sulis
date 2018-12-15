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

#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

pub mod ability;
pub use self::ability::Ability;

pub mod ability_list;
pub use self::ability_list::AbilityList;

pub mod actor;
pub use self::actor::Actor;
pub use self::actor::Sex;
pub use self::actor::ActorBuilder;
pub use self::actor::Faction;

pub mod ai;
pub use self::ai::AITemplate;

pub mod area;
pub use self::area::Area;

pub mod class;
pub use self::class::Class;

pub mod conversation;
pub use self::conversation::Conversation;

pub mod cutscene;
pub use self::cutscene::Cutscene;

pub mod object_size;
pub use self::object_size::ObjectSize;
pub use self::object_size::ObjectSizeIterator;

pub mod on_trigger;
pub use self::on_trigger::OnTrigger;
pub use self::on_trigger::MerchantData;

pub mod encounter;
pub use self::encounter::Encounter;

pub mod campaign;
pub use self::campaign::Campaign;
pub use self::campaign::CampaignGroup;

mod generator;

pub mod image_layer;
pub use self::image_layer::ImageLayer;
pub use self::image_layer::ImageLayerSet;

pub mod inventory_builder;
pub use self::inventory_builder::InventoryBuilder;
pub use self::inventory_builder::ItemSaveState;
pub use self::inventory_builder::ItemListEntrySaveState;

pub mod item;
pub use self::item::Item;
pub use self::item::Equippable;
pub use self::item::Usable;

pub mod item_adjective;
pub use self::item_adjective::ItemAdjective;

pub mod loot_list;
pub use self::loot_list::LootList;

pub mod modification;
pub use self::modification::ModificationInfo;

pub mod prereq_list;
pub use self::prereq_list::PrereqList;
pub use self::prereq_list::PrereqListBuilder;

pub mod prop;
pub use self::prop::Prop;

pub mod quest;
pub use self::quest::Quest;

pub mod race;
pub use self::race::Race;

pub mod rules;
pub use self::rules::Rules;

use std::collections::HashMap;
use std::rc::Rc;
use std::io::Error;
use std::cell::RefCell;
use std::fmt::{self, Display};
use std::path::{PathBuf};
use std::fs;
use std::ffi::OsStr;

use sulis_core::config::{self, Config};
use sulis_core::serde_yaml;
use sulis_core::util::{invalid_data_error};
use sulis_core::resource::*;

use self::area::Tile;
use self::ability::AbilityBuilder;
use self::ability_list::AbilityListBuilder;
use self::ai::AITemplateBuilder;
use self::conversation::ConversationBuilder;
use self::cutscene::CutsceneBuilder;
use self::area::AreaBuilder;
use self::class::ClassBuilder;
use self::campaign::CampaignBuilder;
use self::encounter::EncounterBuilder;
use self::item::ItemBuilder;
use self::loot_list::LootListBuilder;
use self::prop::PropBuilder;
use self::race::RaceBuilder;
use self::object_size::ObjectSizeBuilder;
use self::area::{Tileset, tile::{TerrainRules, TerrainKind, WallRules, WallKind}};

thread_local! {
    static MODULE: RefCell<Module> = RefCell::new(Module::default());
}

pub struct Module {
    rules: Option<Rc<Rules>>,
    campaign: Option<Rc<Campaign>>,
    abilities: HashMap<String, Rc<Ability>>,
    ability_lists: HashMap<String, Rc<AbilityList>>,
    actors: HashMap<String, Rc<Actor>>,
    ai_templates: HashMap<String, Rc<AITemplate>>,
    areas: HashMap<String, Rc<Area>>,
    classes: HashMap<String, Rc<Class>>,
    conversations: HashMap<String, Rc<Conversation>>,
    cutscenes: HashMap<String, Rc<Cutscene>>,
    encounters: HashMap<String, Rc<Encounter>>,
    items: HashMap<String, Rc<Item>>,
    item_adjectives: HashMap<String, Rc<ItemAdjective>>,
    loot_lists: HashMap<String, Rc<LootList>>,
    props: HashMap<String, Rc<Prop>>,
    quests: HashMap<String, Rc<Quest>>,
    races: HashMap<String, Rc<Race>>,
    sizes: HashMap<String, Rc<ObjectSize>>,
    tiles: HashMap<String, Rc<Tile>>,
    scripts: HashMap<String, String>,

    terrain_rules: Option<TerrainRules>,
    terrain_kinds: Vec<TerrainKind>,
    wall_rules: Option<WallRules>,
    wall_kinds: Vec<WallKind>,

    root_dir: Option<String>,
    init: bool,
}

#[derive(Clone)]
pub struct ModuleInfo {
    pub id: String,
    pub dir: String,
    pub name: String,
    pub description: String,
    pub group: CampaignGroup,
}

impl ModuleInfo {
    fn from_dir(path: PathBuf) -> Result<ModuleInfo, Error> {
        let path_str = path.to_string_lossy().to_string();
        debug!("Checking module at '{}'", path_str);

        let campaign: CampaignBuilder = read_single_resource(&format!("{}/campaign", path_str))?;

        let group = match campaign.group {
            None => CampaignGroup {
                id: campaign.id.to_string(),
                name: campaign.name.to_string(),
                position: 0
            },
            Some(group) => group,
        };

        Ok(ModuleInfo {
            id: campaign.id,
            dir: path_str,
            name: campaign.name,
            description: campaign.description,
            group,
        })
    }
}

impl Display for ModuleInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

macro_rules! getters {
    ($($name:ident, $plural:ident, $kind:ty);*) => {
        $(
            pub fn $name(id: &str) -> Option<Rc<$kind>> {
                MODULE.with(|m| get_resource(id, &m.borrow().$plural))
            }
         )*
    }
}

impl Module {
    pub fn get_available_modules() -> Vec<ModuleInfo> {
        let root_dir = Config::resources_config().campaigns_directory;
        let mut user_dir = config::USER_DIR.clone();
        user_dir.push(&root_dir);

        let mut modules: Vec<ModuleInfo> = Vec::new();

        let mut dirs = Vec::new();
        match subdirs(&root_dir) {
            Ok(mut subdirs) => dirs.append(&mut subdirs),
            Err(e) => warn!("Unable to read modules from '{}': {}", root_dir, e),
        };

        match subdirs(&user_dir) {
            Ok(mut user_dirs) => dirs.append(&mut user_dirs),
            Err(e) => warn!("Unable to read modules from '{:?}': {}", user_dir, e),
        }

        for dir in dirs {
            match ModuleInfo::from_dir(dir.clone()) {
                Ok(module) => modules.push(module),
                Err(e) => warn!("Error reading module from '{:?}': {}", dir, e),
            }
        }

        modules
    }

    pub fn delete_character(id: &str) {
        // TODO don't assume ID = filename
        let mut path = config::USER_DIR.clone();
        path.push("characters");
        path.push(id);
        path.set_extension("yml");
        info!("Deleting character at '{:?}'", path);

        match std::fs::remove_file(path.as_path()) {
            Err(e) => {
                warn!("Unable to delete file {:?}: {}", path, e);
            }, Ok(()) => (),
        }
    }

    pub fn get_available_characters() -> Vec<Actor> {
        let mut path = config::USER_DIR.clone();
        path.push("characters");

        let mut actors = Vec::new();

        if let Err(_) = fs::create_dir_all(&path) {
            warn!("Unable to create directory: '{}'", path.to_string_lossy());
            return actors;
        }

        debug!("Reading list of available characters");
        let dir_entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => {
                warn!("Unable to read directory: '{}'", path.to_string_lossy());
                return actors;
            }
        };

        for entry in dir_entries {
            trace!("Found entry {:?}", entry);
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => { warn!("Error reading entry: {}", e); continue; }
            };

            if !entry.path().is_file() { continue; }

            let extension: String = OsStr::to_str(entry.path().extension()
                                                  .unwrap_or(OsStr::new(""))).unwrap_or("").to_string();

            if extension != "yml" { continue; }

            let mut path = entry.path().to_path_buf();
            path.set_extension("");
            let actor_builder: ActorBuilder = match read_single_resource(&path
                                                        .to_string_lossy().to_string()) {
                Ok(entry) => entry,
                Err(e) => { warn!("Error reading actor: {}", e); continue; }
            };

            let actor = MODULE.with(|module| {
                let module = module.borrow();
                match Actor::new(actor_builder, &module) {
                    Err(e) => { warn!("Error reading actor: {}", e); None },
                    Ok(actor) => Some(actor),
                }
            });

            if let Some(actor) = actor {
                actors.push(actor);
            }
        }

        actors
    }

    pub fn load_resources(mut yaml: YamlResourceSet, dirs: Vec<String>) -> Result<(), Error> {
        assert!(dirs.len() > 1);
        debug!("Creating module from parsed data.");

        let file_key = serde_yaml::Value::String(yaml_resource_set::FILE_VAL_STR.to_string());

        let top_level = yaml.resources.remove(&YamlResourceKind::TopLevel);
        let (rules_yaml, campaign_yaml) = match top_level {
            None => return invalid_data_error("No rules or campaign files defined"),
            Some(mut map) => {
                let rules_yaml = match map.remove("rules") {
                    None => return invalid_data_error("No rules file defined"),
                    Some(yaml) => yaml,
                };

                let mut campaign_yaml = None;
                for (id, yaml) in map {
                    if let serde_yaml::Value::Mapping(ref map) = yaml {
                        if let Some(serde_yaml::Value::Sequence(files)) = map.get(&file_key) {
                            if files.iter().all(|file| {
                                if let serde_yaml::Value::String(file) = file {
                                    !file.ends_with("campaign.yml")
                                } else {
                                    false
                                }
                            }) { continue; }
                        }
                    }

                    if campaign_yaml.is_some() {
                        return invalid_data_error(&format!("Multiple potential campaign files \
                            detected at top level: '{}'", id));
                    }
                    campaign_yaml = Some(yaml);
                }

                if campaign_yaml.is_none() {
                    return invalid_data_error("No campaign file found at top level");
                }

                (rules_yaml, campaign_yaml.unwrap())
            }
        };

        let rules: Rules = read_builder(rules_yaml)?;
        rules.validate()?;

        let campaign_builder: CampaignBuilder = read_builder(campaign_yaml)?;

        let builder_set = ModuleBuilder::from_yaml(&mut yaml)?;
        MODULE.with(|module| {
            let mut module = module.borrow_mut();
            module.abilities.clear();
            module.ability_lists.clear();
            module.actors.clear();
            module.ai_templates.clear();
            module.areas.clear();
            module.classes.clear();
            module.conversations.clear();
            module.cutscenes.clear();
            module.encounters.clear();
            module.items.clear();
            module.item_adjectives.clear();
            module.loot_lists.clear();
            module.quests.clear();
            module.props.clear();
            module.races.clear();
            module.sizes.clear();
            module.tiles.clear();
            module.scripts.clear();

            module.rules = Some(Rc::new(rules));
            module.scripts = read_to_string(&dirs, "scripts");

            module.root_dir = Some(dirs[1].to_string());

            for (id, adj) in builder_set.item_adjectives {
                trace!("Inserting resource of type item_adjective with key {} \
                    into resource set.", id);
                module.item_adjectives.insert(id, Rc::new(adj));
            }

            for (id, quest) in builder_set.quests {
                trace!("Inserting resource of type quest with key {} \
                    into module.", id);
                module.quests.insert(id, Rc::new(quest));
            }

            for (id, builder) in builder_set.size_builders {
                insert_if_ok("size", id, ObjectSize::new(builder), &mut module.sizes);
            }

            for (_, mut tiles_list) in builder_set.tile_builders {
                tiles_list.move_tiles();

                module.terrain_rules = Some(tiles_list.terrain_rules);
                module.terrain_kinds = tiles_list.terrain_kinds;
                module.wall_rules = Some(tiles_list.wall_rules);
                module.wall_kinds = tiles_list.wall_kinds;

                for (id, tile_builder) in tiles_list.tiles {
                    insert_if_ok("tile", id.to_string(), Tile::new(id, tile_builder), &mut module.tiles);
                }
            }

            for (id, builder) in builder_set.ai_builders {
                insert_if_ok("ai_template", id, AITemplate::new(builder, &module),
                    &mut module.ai_templates);
            }

            for (id, builder) in builder_set.ability_builders {
                insert_if_ok("ability", id, Ability::new(builder, &module), &mut module.abilities);
            }

            for (id, builder) in builder_set.ability_list_builders {
                insert_if_ok("ability_list", id, AbilityList::new(builder, &module), &mut module.ability_lists);
            }

            for (id, builder) in builder_set.item_builders.into_iter() {
                insert_if_ok("item", id, Item::new(builder, &module), &mut module.items);
            }

            for (id, builder) in builder_set.loot_builders.into_iter() {
                insert_if_ok("loot list", id, LootList::new(builder, &module), &mut module.loot_lists);
            }

            for (id, builder) in builder_set.prop_builders {
                insert_if_ok("prop", id, Prop::new(builder, &module), &mut module.props);
            }

            for (id, builder) in builder_set.race_builders.into_iter() {
                insert_if_ok("race", id, Race::new(builder, &module), &mut module.races);
            }

            for (id, builder) in builder_set.class_builders.into_iter() {
                insert_if_ok("class", id, Class::new(builder, &module), &mut module.classes);
            }

            for (id, builder) in builder_set.conversation_builders.into_iter() {
                insert_if_ok("conversation", id, Conversation::new(builder, &module), &mut module.conversations);
            }

            for (id, builder) in builder_set.actor_builders.into_iter() {
                insert_if_ok("actor", id, Actor::new(builder, &module), &mut module.actors);
            }

            for (id, builder) in builder_set.encounter_builders.into_iter() {
                insert_if_ok("encounter", id, Encounter::new(builder, &module), &mut module.encounters);
            }

            for (id, builder) in builder_set.cutscene_builders {
                insert_if_ok("cutscene", id, Cutscene::new(builder, &module), &mut module.cutscenes);
            }

            for (id, builder) in builder_set.area_builders {
                 insert_if_ok("area", id, Area::new(builder, &module), &mut module.areas);
            }
        });

        let campaign = Campaign::new(campaign_builder)?;

        MODULE.with(move |m| {
            let mut m = m.borrow_mut();
            m.campaign= Some(Rc::new(campaign));
            m.init = true;
        });

        Ok(())
    }

    pub fn module_dir() -> Option<String> {
        MODULE.with(|m| {
            match m.borrow().root_dir {
                None => None,
                Some(ref dir) => Some(dir.to_string()),
            }
        })
    }

    pub fn is_initialized() -> bool {
        MODULE.with(|m| { m.borrow_mut().init })
    }

    pub fn load_actor(builder: ActorBuilder) -> Result<Actor, Error> {
        MODULE.with(|module| {
            let module = module.borrow();
            Actor::new(builder, &module)
        })
    }

    pub fn add_actor_to_resources(builder: ActorBuilder) {
        let result: Result<(), Error> = MODULE.with(|module| {
            let mut module = module.borrow_mut();
            let actor = Actor::new(builder, &module)?;
            let id = actor.id.to_string();
            module.actors.insert(id, Rc::new(actor));
            Ok(())
        });

        if let Err(e) = result {
            warn!("Error loading and inserting actor");
            warn!("{}", e);
        }
    }

    pub fn campaign() -> Rc<Campaign> {
        MODULE.with(|m| Rc::clone(m.borrow().campaign.as_ref().unwrap()))
    }

    pub fn rules() -> Rc<Rules> {
        MODULE.with(|m| Rc::clone(m.borrow().rules.as_ref().unwrap()))
    }

    pub fn wall_rules() -> WallRules {
        MODULE.with(|m| m.borrow().wall_rules.as_ref().unwrap().clone())
    }

    pub fn wall_kinds() -> Vec<WallKind> {
        MODULE.with(|m| m.borrow().wall_kinds.clone())
    }

    pub fn terrain_rules() -> TerrainRules {
        MODULE.with(|m| m.borrow().terrain_rules.as_ref().unwrap().clone())
    }

    pub fn terrain_kinds() -> Vec<TerrainKind> {
        MODULE.with(|m| m.borrow().terrain_kinds.clone())
    }

    pub fn create_get_item(id: &str, adjectives: &Vec<String>) -> Option<Rc<Item>> {
        if adjectives.len() == 0 { return Module::item(id); }

        MODULE.with(|m| {
            let mut module = m.borrow_mut();

            let mut new_id = String::new();
            new_id.push_str(id);
            new_id.push_str("__ADJ__");
            for adj in adjectives.iter() {
                new_id.push_str(&adj);
            }

            if let Some(ref item) = module.items.get(&new_id) {
                return Some(Rc::clone(item));
            }

            let base_item = match module.items.get(id) {
                None => return None,
                Some(ref item) => Rc::clone(item),
            };

            let mut adjs = Vec::new();
            for adj_id in adjectives.iter() {
                let adjective = match module.item_adjectives.get(adj_id) {
                    None => return None,
                    Some(ref adj) => Rc::clone(adj),
                };
                adjs.push(adjective);
            }

            let item = Rc::new(Item::clone_with_adjectives(&base_item, adjs, new_id.clone()));

            module.items.insert(new_id, Rc::clone(&item));
            Some(item)
        })
    }

    getters!(
        ability, abilities, Ability;
        ability_list, ability_lists, AbilityList;
        actor, actors, Actor;
        ai_template, ai_templates, AITemplate;
        area, areas, Area;
        class, classes, Class;
        conversation, conversations, Conversation;
        cutscene, cutscenes, Cutscene;
        encounter, encounters, Encounter;
        item, items, Item;
        item_adjective, item_adjectives, ItemAdjective;
        loot_list, loot_lists, LootList;
        object_size, sizes, ObjectSize;
        quest, quests, Quest;
        prop, props, Prop;
        race, races, Race;
        tile, tiles, Tile
        );

    pub fn script(id: &str) -> Option<String> {
        MODULE.with(|r| {
            let module = r.borrow();
            match module.scripts.get(id) {
                None => None,
                Some(ref script) => Some(script.to_string())
            }
        })
    }

    pub fn all_actors() -> Vec<Rc<Actor>> {
        MODULE.with(|r| all_resources(&r.borrow().actors))
    }

    pub fn all_object_sizes() -> Vec<Rc<ObjectSize>> {
        MODULE.with(|r| r.borrow().sizes.iter().map(|ref s| Rc::clone(s.1)).collect())
    }

    pub fn all_classes() -> Vec<Rc<Class>> {
        MODULE.with(|r| all_resources(&r.borrow().classes))
    }

    pub fn all_encounters() -> Vec<Rc<Encounter>> {
        MODULE.with(|r| all_resources(&r.borrow().encounters))
    }

    pub fn all_props() -> Vec<Rc<Prop>> {
        MODULE.with(|r| all_resources(&r.borrow().props))
    }

    pub fn all_quests() -> Vec<Rc<Quest>> {
        MODULE.with(|r| all_resources(&r.borrow().quests))
    }

    pub fn all_races() -> Vec<Rc<Race>> {
        MODULE.with(|r| all_resources(&r.borrow().races))
    }

    pub fn all_tiles() -> Vec<Rc<Tile>> {
        MODULE.with(|r| all_resources(&r.borrow().tiles))
    }
}

impl Default for Module {
    fn default() -> Module {
        Module {
            root_dir: None,
            rules: None,
            campaign: None,
            abilities: HashMap::new(),
            ability_lists: HashMap::new(),
            actors: HashMap::new(),
            ai_templates: HashMap::new(),
            areas: HashMap::new(),
            classes: HashMap::new(),
            conversations: HashMap::new(),
            cutscenes: HashMap::new(),
            items: HashMap::new(),
            encounters: HashMap::new(),
            props: HashMap::new(),
            quests: HashMap::new(),
            item_adjectives: HashMap::new(),
            loot_lists: HashMap::new(),
            races: HashMap::new(),
            sizes: HashMap::new(),
            tiles: HashMap::new(),
            scripts: HashMap::new(),
            terrain_rules: None,
            terrain_kinds: Vec::new(),
            wall_rules: None,
            wall_kinds: Vec::new(),
            init: false,
        }
    }
}

struct ModuleBuilder {
    ability_builders: HashMap<String, AbilityBuilder>,
    ability_list_builders: HashMap<String, AbilityListBuilder>,
    actor_builders: HashMap<String, ActorBuilder>,
    ai_builders: HashMap<String, AITemplateBuilder>,
    area_builders: HashMap<String, AreaBuilder>,
    class_builders: HashMap<String, ClassBuilder>,
    cutscene_builders: HashMap<String, CutsceneBuilder>,
    conversation_builders: HashMap<String, ConversationBuilder>,
    encounter_builders: HashMap<String, EncounterBuilder>,
    item_builders: HashMap<String, ItemBuilder>,
    loot_builders: HashMap<String, LootListBuilder>,
    prop_builders: HashMap<String, PropBuilder>,
    race_builders: HashMap<String, RaceBuilder>,
    size_builders: HashMap<String, ObjectSizeBuilder>,
    tile_builders: HashMap<String, Tileset>,

    item_adjectives: HashMap<String, ItemAdjective>,
    quests: HashMap<String, Quest>,
}

impl ModuleBuilder {
    fn from_yaml(resources: &mut YamlResourceSet) -> Result<ModuleBuilder, Error> {
        use self::YamlResourceKind::*;
        Ok(ModuleBuilder {
            ability_builders: read_builders(resources, Ability)?,
            ability_list_builders: read_builders(resources, AbilityList)?,
            actor_builders: read_builders(resources, Actor)?,
            ai_builders: read_builders(resources, AiTemplate)?,
            area_builders: read_builders(resources, Area)?,
            class_builders: read_builders(resources, Class)?,
            conversation_builders: read_builders(resources, Conversation)?,
            cutscene_builders: read_builders(resources, Cutscene)?,
            encounter_builders: read_builders(resources, Encounter)?,
            item_builders: read_builders(resources, Item)?,
            item_adjectives: read_builders(resources, ItemAdjective)?,
            loot_builders: read_builders(resources, LootList)?,
            prop_builders: read_builders(resources, Prop)?,
            quests: read_builders(resources, Quest)?,
            race_builders: read_builders(resources, Race)?,
            size_builders: read_builders(resources, Size)?,
            tile_builders: read_builders(resources, Tile)?,
        })
    }
}
