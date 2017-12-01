mod generator;

mod game;
pub use self::game::Game;

mod area;
pub use self::area::Area;

mod terrain;
pub use self::terrain::Terrain;

mod tile;
pub use self::tile::Tile;

mod entity;
pub use self::entity::Entity;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Error};
use std::ffi::OsStr;

use resource::tile::TileBuilder;
use resource::area::AreaBuilder;
use resource::entity::EntityBuilder;

use std::rc::Rc;

pub struct ResourceSet {
    pub game: Game,
    pub areas: HashMap<String, Area>,
    pub tiles: HashMap<String, Rc<Tile>>,
    pub entities: HashMap<String, Entity>,
}

impl ResourceSet {
    pub fn new(root_directory: &str) -> Result<ResourceSet, Error> {
        let builder_set = ResourceBuilderSet::new(root_directory)?;

        let tiles: HashMap<String, Rc<Tile>> = builder_set.tile_builders.into_iter()
            .map(|(id, builder)| (id, Rc::new(Tile::new(builder)))).collect();


        let mut areas: HashMap<String, Area> = HashMap::new();
        for (id, area_builder) in builder_set.area_builders {
            let area = Area::new(area_builder, &tiles);

            match area {
                Ok(a) => { areas.insert(id, a); }
                Err(e) => { eprintln!("{}", e); }
            }
        }

        let entities: HashMap<String, Entity> = builder_set.entity_builders.into_iter()
            .map(|(id, builder)| (id, Entity::new(builder))).collect();

        Ok(ResourceSet {
            tiles: tiles,
            areas: areas,
            entities: entities,
            game: builder_set.game,
        })
    }
}

pub trait ResourceBuilder where Self: Sized {
    fn owned_id(& self) -> String;

    fn new(data: &str) -> Result<Self, Error>;
}

#[derive(Debug)]
pub struct ResourceBuilderSet {
    pub game: Game,
    pub area_builders: HashMap<String, AreaBuilder>,
    pub tile_builders: HashMap<String, TileBuilder>,
    pub entity_builders: HashMap<String, EntityBuilder>,
}

impl ResourceBuilderSet {
    pub fn new(root: &str) -> Result<ResourceBuilderSet, Error> {
        let game_filename = root.to_owned() + "/game.json";
        let game = match ResourceBuilderSet::create_game(&game_filename) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Unable to load game startup state from {}", game_filename);
                return Err(e);
            }
        };

        Ok(ResourceBuilderSet {
            game,
            tile_builders: read_resources(&format!("{}/tiles/", root)),
            area_builders: read_resources(&format!("{}/areas/", root)),
            entity_builders: read_resources(&format!("{}/entities/", root)),
        })
    }

    pub fn create_game(filename: &str) -> Result<Game, Error> {
        let mut f = File::open(filename)?;
        let mut file_data = String::new();
        f.read_to_string(&mut file_data)?;
        let game = Game::new(&file_data)?;

        Ok(game)
    }
}

fn read_resources<T: ResourceBuilder>(dir: &str) -> HashMap<String, T> {
    let mut resources: HashMap<String, T> = HashMap::new();
    let dir_entries = fs::read_dir(dir);
    let dir_entries = match dir_entries {
        Ok(entries) => entries,
        Err(_) => {
            eprintln!("Unable to read directory: {}", dir);
            return resources;
        }
    };

    for entry in dir_entries {
        let entry = match entry {
            Ok(e) => e,
            Err(error) => {
                eprintln!("Error reading file: {}", error);
                continue;
            }
        };

        let path = entry.path();
        let path2 = path.clone();
        let extension: &str = match path2.extension() {
            Some(ext) => match OsStr::to_str(ext) {
                Some(str) => str,
                None => ""
            },
            None => ""
        };
        if path.is_file() && extension == "json" {
            let path_str = path.to_string_lossy().into_owned();
            let f = File::open(path);
            let mut f = match f {
                Ok(file) => file,
                Err(error) => {
                    eprintln!("Error reading file: {}", error);
                    continue;
                }
            };

            let mut file_data = String::new();
            if f.read_to_string(&mut file_data).is_err() {
                eprintln!("Error reading file data from file");
                continue;
            }

            let resource = T::new(&file_data); 
            let resource = match resource {
                Ok(a) => a,
                Err(error) => {
                    eprintln!("Error parsing file data: {:?}", path_str);
                    eprintln!("  {}", error);
                    continue;
                }
            };

            let id = resource.owned_id();
            if resources.contains_key(&id) {
                eprintln!("Error: duplicate resource key: {}", id);
                continue;
            }


            resources.insert(id, resource);
        }
    }

    resources
}
