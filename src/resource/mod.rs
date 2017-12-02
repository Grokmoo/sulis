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
use std::hash::Hash;
use std::rc::Rc;
use std::io::Error;

use resource::actor::ActorBuilder;
use resource::area::AreaBuilder;
use resource::resource_builder_set::ResourceBuilderSet;

pub struct ResourceSet {
    pub game: Game,
    pub areas: HashMap<String, Area>,
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

        let sizes: HashMap<usize, Rc<Size>> = builder_set.size_builders.into_iter().
            map(|(_id_str, size)| (size.size, Rc::new(Size::new(size)))).collect();

        let tiles = create_rc_hashmap(builder_set.tiles);

        let mut areas: HashMap<String, Area> = HashMap::new();
        for (id, area_builder) in builder_set.area_builders {
            let area = Area::new(area_builder, &tiles);

            match area {
                Ok(a) => { areas.insert(id, a); }
                Err(e) => { eprintln!("{}", e); }
            }
        }

        let mut actors: HashMap<String, Rc<Actor>> = HashMap::new();
        for (id, builder) in builder_set.actor_builders.into_iter() {
            let actor = Actor::new(builder, &sizes);

            match actor {
                Ok(a) => { actors.insert(id, Rc::new(a)); }
                Err(e) => { eprintln!("{}", e); }
            }
        }

        Ok(ResourceSet {
            tiles: tiles,
            areas: areas,
            actors: actors,
            game: builder_set.game,
            sizes: sizes,
        })
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

fn create_rc_hashmap<K: Eq + Hash, V>(data: HashMap<K, V>) -> HashMap<K, Rc<V>> {
    data.into_iter().map(|(id, entry)| (id, Rc::new(entry))).collect()
}
