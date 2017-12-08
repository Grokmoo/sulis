mod resource_builder_set;
mod generator;

mod game;
pub use self::game::Game;

mod area;
pub use self::area::Area;

mod terrain;
pub use self::terrain::Terrain;

mod tile;
pub use self::tile::Tile;

mod actor;
pub use self::actor::Actor;

mod size;
pub use self::size::Size;
pub use self::size::SizeIterator;

mod point;
pub use self::point::Point;

use std::collections::HashMap;
use std::rc::Rc;
use std::io::Error;
use std::fmt::Display;
use std::hash::Hash;

use resource::actor::ActorBuilder;
use resource::area::AreaBuilder;
use resource::tile::TileBuilder;
use resource::resource_builder_set::ResourceBuilderSet;

pub struct ResourceSet {
    pub game: Game,
    areas: HashMap<String, Rc<Area>>,
    tiles: HashMap<String, Rc<Tile>>,
    actors: HashMap<String, Rc<Actor>>,
    sizes: HashMap<usize, Rc<Size>>,
}

pub trait ResourceBuilder where Self: Sized {
    fn owned_id(& self) -> String;

    fn new(data: &str) -> Result<Self, Error>;
}

impl ResourceSet {
    pub fn new(root_directory: &str) -> Result<ResourceSet, Error> {
        let builder_set = ResourceBuilderSet::new(root_directory)?;

        debug!("Creating resource set from parsed data.");
        let mut sizes: HashMap<usize, Rc<Size>> = HashMap::new();
        for (_id_str, builder) in builder_set.size_builders {
            insert_if_ok("size", builder.size, Size::new(builder), &mut sizes);
        }

        let mut tiles: HashMap<String, Rc<Tile>> = HashMap::new();
        for (id, builder) in builder_set.tile_builders {
            insert_if_ok("tile", id, Tile::new(builder), &mut tiles);
        }

        let mut actors: HashMap<String, Rc<Actor>> = HashMap::new();
        for (id, builder) in builder_set.actor_builders.into_iter() {
            insert_if_ok("actor", id, Actor::new(builder, &sizes), &mut actors);
        }

        let mut areas: HashMap<String, Rc<Area>> = HashMap::new();
        for (id, builder) in builder_set.area_builders {
            insert_if_ok("area", id, Area::new(builder, &tiles, &sizes), &mut areas);
        }

        Ok(ResourceSet {
            tiles: tiles,
            areas: areas,
            actors: actors,
            game: builder_set.game,
            sizes: sizes,
        })
    }

    pub fn get_area(&self, id: &str) -> Option<Rc<Area>> {
        let area = self.areas.get(id);

        match area {
            None => None,
            Some(a) => Some(Rc::clone(a)),
        }
    }

    pub fn get_actor(&self, id: &str) -> Option<Rc<Actor>> {
        let actor = self.actors.get(id);

        match actor {
            None => None,
            Some(a) => Some(Rc::clone(a)),
        }
    }

    pub fn get_tile(&self, id: &str) -> Option<Rc<Tile>> {
        let tile = self.tiles.get(id);

        match tile {
            None => None,
            Some(t) => Some(Rc::clone(t)),
        }
    }

    pub fn get_size(&self, size: usize) -> Option<Rc<Size>> {
        let size = self.sizes.get(&size);

        match size {
            None => None,
            Some(s) => Some(Rc::clone(s)),
        }
    }
}

fn insert_if_ok<K: Eq + Hash + Display, V>(type_str: &str, key: K, val: Result<V, Error>,
                                           map: &mut HashMap<K, Rc<V>>) {
    trace!("Inserting resource of type {} with key {} into resource set.", type_str, key);
    match val {
        Err(e) => {
            warn!("Error in {} with id '{}'", type_str, key);
            warn!("{}", e);
        },
        Ok(v) => { (*map).insert(key, Rc::new(v)); }
    };
}
