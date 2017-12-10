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

mod image;
pub use self::image::Image;

use std::collections::HashMap;
use std::rc::Rc;
use std::io::Error;
use std::fmt::Display;
use std::hash::Hash;

use resource::actor::ActorBuilder;
use resource::area::AreaBuilder;
use resource::tile::TileBuilder;
use resource::resource_builder_set::ResourceBuilderSet;
use resource::image::ComposedImage;

pub struct ResourceSet {
    pub game: Game,
    areas: HashMap<String, Rc<Area>>,
    tiles: HashMap<String, Rc<Tile>>,
    actors: HashMap<String, Rc<Actor>>,
    sizes: HashMap<usize, Rc<Size>>,
    images: HashMap<String, Rc<Image>>,
}

pub trait ResourceBuilder where Self: Sized {
    fn owned_id(& self) -> String;

    fn new(data: &str) -> Result<Self, Error>;
}

impl ResourceSet {
    pub fn new(root_directory: &str) -> Result<ResourceSet, Error> {
        let builder_set = ResourceBuilderSet::new(root_directory)?;

        debug!("Creating resource set from parsed data.");

        let mut images: HashMap<String, Rc<Image>> = HashMap::new();
        for (id, image) in builder_set.simple_images {
            insert_if_ok_boxed("image", id, Ok(Rc::new(image) as Rc<Image>),
                &mut images);
        }

        for (id, image) in builder_set.composed_builders {
            insert_if_ok_boxed("image", id, ComposedImage::new(image, &images),
                &mut images);
        }

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
            images: images,
        })
    }


    pub fn get_resource<V: ?Sized>(&self, id: &str,
        map: &HashMap<String, Rc<V>>) -> Option<Rc<V>> {

        let resource = map.get(id);

        match resource {
            None => None,
            Some(r) => Some(Rc::clone(r)),
        }
    }

    pub fn get_image(&self, id: &str) -> Option<Rc<Image>> {
        self.get_resource(id, &self.images)
    }

    pub fn get_area(&self, id: &str) -> Option<Rc<Area>> {
        self.get_resource(id, &self.areas)
    }

    pub fn get_actor(&self, id: &str) -> Option<Rc<Actor>> {
        self.get_resource(id, &self.actors)
    }

    pub fn get_tile(&self, id: &str) -> Option<Rc<Tile>> {
        self.get_resource(id, &self.tiles)
    }

    pub fn get_size(&self, size: usize) -> Option<Rc<Size>> {
        let size = self.sizes.get(&size);

        match size {
            None => None,
            Some(s) => Some(Rc::clone(s)),
        }
    }
}

fn insert_if_ok<K: Eq + Hash + Display, V>(type_str: &str,
    key: K, val: Result<V, Error>, map: &mut HashMap<K, Rc<V>>) {

    trace!("Inserting resource of type {} with key {} into resource set.",
           type_str, key);
    match val {
        Err(e) => warn_on_insert(type_str, key, e),
        Ok(v) => { (*map).insert(key, Rc::new(v)); }
    };
}

fn insert_if_ok_boxed<K: Eq + Hash + Display, V: ?Sized>(type_str: &str,
    key: K, val: Result<Rc<V>, Error>, map: &mut HashMap<K, Rc<V>>) {

    trace!("Inserting resource of type {} with key {} into resource set.",
           type_str, key);
    match val {
        Err(e) => warn_on_insert(type_str, key, e),
        Ok(v) => { (*map).insert(key, Rc::clone(&v)); },
    };
}

fn warn_on_insert<K: Display>(type_str: &str, key: K, error: Error) {
    warn!("Error in {} with id '{}'", type_str, key);
    warn!("{}", error);
}
