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

extern crate sulis_core;
extern crate sulis_rules;

extern crate rand;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

pub mod actor;
pub use self::actor::Actor;
pub use self::actor::Sex;

pub mod area;
pub use self::area::Area;

pub mod class;
pub use self::class::Class;

pub mod entity_size;
pub use self::entity_size::EntitySize;
pub use self::entity_size::EntitySizeIterator;

pub mod equippable;
pub use self::equippable::Equippable;

pub mod game;
pub use self::game::Game;

mod generator;

pub mod image_layer;
pub use self::image_layer::ImageLayer;
pub use self::image_layer::ImageLayerSet;

pub mod item;
pub use self::item::Item;

pub mod item_adjective;
pub use self::item_adjective::ItemAdjective;

pub mod race;
pub use self::race::Race;

pub mod rules;
pub use self::rules::Rules;

use std::collections::HashMap;
use std::rc::Rc;
use std::io::Error;
use std::cell::RefCell;
use std::fmt::{self, Display};
use std::path::PathBuf;
use std::fs;

use sulis_core::resource::{all_resources, read, read_single_resource, get_resource, insert_if_ok};

use self::area::Tile;
use self::actor::ActorBuilder;
use self::area::AreaBuilder;
use self::class::ClassBuilder;
use self::item::ItemBuilder;
use self::race::RaceBuilder;
use self::entity_size::EntitySizeBuilder;
use self::area::TileBuilder;

thread_local! {
    static MODULE: RefCell<Module> = RefCell::new(Module::default());
}

pub struct Module {
    rules: Option<Rc<Rules>>,
    game: Option<Rc<Game>>,
    actors: HashMap<String, Rc<Actor>>,
    areas: HashMap<String, Rc<Area>>,
    classes: HashMap<String, Rc<Class>>,
    items: HashMap<String, Rc<Item>>,
    item_adjectives: HashMap<String, Rc<ItemAdjective>>,
    races: HashMap<String, Rc<Race>>,
    sizes: HashMap<usize, Rc<EntitySize>>,
    tiles: HashMap<String, Rc<Tile>>,
}

#[derive(Clone)]
pub struct ModuleInfo {
    pub dir: String,
    pub name: String,
}

impl ModuleInfo {
    fn from_dir(path: PathBuf) -> Result<ModuleInfo, Error> {
        let path_str = path.to_string_lossy().to_string();
        debug!("Checking module at '{}'", path_str);

        let game: Game = read_single_resource(&format!("{}/module", path_str))?;

        Ok(ModuleInfo {
            dir: path_str,
            name: game.name,
        })
    }
}

impl Display for ModuleInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Module {
    pub fn get_available_modules(root_dir: &str) -> Vec<ModuleInfo> {
        let mut modules: Vec<ModuleInfo> = Vec::new();
        let path = PathBuf::from(root_dir);

        let dir_entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => {
                warn!("Unable to read directory: {}", root_dir);
                return modules;
            }
        };

        for entry in dir_entries {
            trace!("Found entry {:?}", entry);
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    warn!("Error reading entry: {}", e);
                    continue;
                }
            };

            if entry.path().is_dir() {
                let module = match ModuleInfo::from_dir(entry.path()) {
                    Ok(module) => module,
                    Err(e) => {
                        warn!("Unable to read module from '{:?}'", entry.path());
                        warn!("{}", e);
                        continue;
                    }
                };
                modules.push(module);
            }
        }

        modules
    }

    pub fn init(data_dir: &str, root_dir: &str) -> Result<(), Error> {
        let builder_set = ModuleBuilder::new(data_dir, root_dir);

        debug!("Creating module from parsed data.");

        MODULE.with(|module| {
            let mut module = module.borrow_mut();

            for (id, adj) in builder_set.item_adjectives {
                trace!("Inserting resource of type item_adjective with key {} \
                    into resource set.", id);
                module.item_adjectives.insert(id, Rc::new(adj));
            }

            for (_id_str, builder) in builder_set.size_builders {
                insert_if_ok("size", builder.size, EntitySize::new(builder), &mut module.sizes);
            }

            for (id, builder) in builder_set.tile_builders {
                insert_if_ok("tile", id, Tile::new(builder), &mut module.tiles);
            }

            for (id, builder) in builder_set.item_builders.into_iter() {
                insert_if_ok("item", id, Item::new(builder), &mut module.items);
            }

            for (id, builder) in builder_set.race_builders.into_iter() {
                insert_if_ok("race", id, Race::new(builder, &module), &mut module.races);
            }

            for (id, builder) in builder_set.class_builders.into_iter() {
                insert_if_ok("class", id, Class::new(builder), &mut module.classes);
            }

            for (id, builder) in builder_set.actor_builders.into_iter() {
                insert_if_ok("actor", id, Actor::new(builder, &module), &mut module.actors);
            }

            for (id, builder) in builder_set.area_builders {
                 insert_if_ok("area", id, Area::new(builder, &module), &mut module.areas);
            }
        });

        let game = read_single_resource(&format!("{}/module", root_dir))?;
        let rules = read_single_resource(&format!("{}/rules", root_dir))?;

        MODULE.with(move |m| {
            let mut m = m.borrow_mut();
            m.game = Some(Rc::new(game));
            m.rules = Some(Rc::new(rules));
        });

        Ok(())
    }

    pub fn actor(id: &str) -> Option<Rc<Actor>> {
        MODULE.with(|r| get_resource(id, &r.borrow().actors))
    }

    pub fn all_actors() -> Vec<Rc<Actor>> {
        MODULE.with(|r| all_resources(&r.borrow().actors))
    }

    pub fn area(id: &str) -> Option<Rc<Area>> {
        MODULE.with(|m| get_resource(id, &m.borrow().areas))
    }

    pub fn entity_size(id: usize) -> Option<Rc<EntitySize>> {
        MODULE.with(|r| {
            let r = r.borrow();
            let size = r.sizes.get(&id);

            match size {
                None => None,
                Some(s) => Some(Rc::clone(s)),
            }
        })
    }

    pub fn all_entity_sizes() -> Vec<Rc<EntitySize>> {
        MODULE.with(|r| r.borrow().sizes.iter().map(|ref s| Rc::clone(s.1)).collect())
    }

    pub fn class(id: &str) -> Option<Rc<Class>> {
        MODULE.with(|r| get_resource(id, &r.borrow().classes))
    }

    pub fn all_classes() -> Vec<Rc<Class>> {
        MODULE.with(|r| all_resources(&r.borrow().classes))
    }

    pub fn game() -> Rc<Game> {
        MODULE.with(|m| Rc::clone(m.borrow().game.as_ref().unwrap()))
    }

    pub fn race(id: &str) -> Option<Rc<Race>> {
        MODULE.with(|r| get_resource(id, &r.borrow().races))
    }

    pub fn all_races() -> Vec<Rc<Race>> {
        MODULE.with(|r| all_resources(&r.borrow().races))
    }

    pub fn rules() -> Rc<Rules> {
        MODULE.with(|m| Rc::clone(m.borrow().rules.as_ref().unwrap()))
    }

    pub fn tile(id: &str) -> Option<Rc<Tile>> {
        MODULE.with(|r| get_resource(id, &r.borrow().tiles))
    }

    pub fn all_tiles() -> Vec<Rc<Tile>> {
        MODULE.with(|r| all_resources(&r.borrow().tiles))
    }
}

impl Default for Module {
    fn default() -> Module {
        Module {
            rules: None,
            game: None,
            actors: HashMap::new(),
            areas: HashMap::new(),
            classes: HashMap::new(),
            items: HashMap::new(),
            item_adjectives: HashMap::new(),
            races: HashMap::new(),
            sizes: HashMap::new(),
            tiles: HashMap::new(),
        }
    }
}

struct ModuleBuilder {
    actor_builders: HashMap<String, ActorBuilder>,
    area_builders: HashMap<String, AreaBuilder>,
    class_builders: HashMap<String, ClassBuilder>,
    item_builders: HashMap<String, ItemBuilder>,
    item_adjectives: HashMap<String, ItemAdjective>,
    race_builders: HashMap<String, RaceBuilder>,
    size_builders: HashMap<String, EntitySizeBuilder>,
    tile_builders: HashMap<String, TileBuilder>,
}

impl ModuleBuilder {
    fn new(data_dir: &str, root_dir: &str) -> ModuleBuilder {
        let root_dirs: Vec<&str> = vec![data_dir, root_dir];
        ModuleBuilder {
            actor_builders: read(&root_dirs, "actors"),
            area_builders: read(&root_dirs, "areas"),
            class_builders: read(&root_dirs, "classes"),
            item_builders: read(&root_dirs, "items"),
            item_adjectives: read(&root_dirs, "item_adjectives"),
            race_builders: read(&root_dirs, "races"),
            size_builders: read(&root_dirs, "sizes"),
            tile_builders: read(&root_dirs, "tiles"),
        }
    }
}
