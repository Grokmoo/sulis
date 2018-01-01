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

pub mod item;
pub use self::item::Item;

mod spritesheet;
pub use self::spritesheet::Spritesheet;
pub use self::spritesheet::Sprite;

mod item_adjective;
pub use self::item_adjective::ItemAdjective;

mod size;
pub use self::size::Size;
pub use self::size::SizeIterator;

use ui::Theme;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::io::Error;
use std::fmt::Display;
use std::hash::Hash;

use resource::actor::ActorBuilder;
use resource::area::AreaBuilder;
use resource::tile::TileBuilder;
use resource::item::ItemBuilder;
use resource::resource_builder_set::ResourceBuilderSet;
use image::{Image, SimpleImage, AnimatedImage, ComposedImage};

thread_local! {
    static RESOURCE_SET: RefCell<ResourceSet> = RefCell::new(ResourceSet::new());
}

#[derive(Debug, PartialEq)]
pub enum BuilderType {
    JSON,
    YAML,
}

pub trait ResourceBuilder where Self: Sized {
    fn owned_id(& self) -> String;

    fn from_json(data: &str) -> Result<Self, Error>;

    fn from_yaml(data: &str) -> Result<Self, Error>;
}

pub struct ResourceSet {
    game: Option<Rc<Game>>,
    theme: Option<Rc<Theme>>,
    areas: HashMap<String, Rc<Area>>,
    tiles: HashMap<String, Rc<Tile>>,
    actors: HashMap<String, Rc<Actor>>,
    item_adjectives: HashMap<String, Rc<ItemAdjective>>,
    items: HashMap<String, Rc<Item>>,
    sizes: HashMap<usize, Rc<Size>>,
    images: HashMap<String, Rc<Image>>,
    pub spritesheets: HashMap<String, Rc<Spritesheet>>,
}

impl ResourceSet {
    pub fn new() -> ResourceSet {
        ResourceSet {
            game: None,
            theme: None,
            areas: HashMap::new(),
            tiles: HashMap::new(),
            actors: HashMap::new(),
            sizes: HashMap::new(),
            images: HashMap::new(),
            items: HashMap::new(),
            item_adjectives: HashMap::new(),
            spritesheets: HashMap::new(),
        }
    }

    pub fn init(root_directory: &str) -> Result<(), Error> {
        let builder_set = ResourceBuilderSet::new(root_directory)?;

        debug!("Creating resource set from parsed data.");

        RESOURCE_SET.with(|resource_set| {
            let mut resource_set = resource_set.borrow_mut();

            resource_set.game = Some(Rc::new(builder_set.game));
            resource_set.theme = Some(Rc::new(Theme::new("", builder_set.theme_builder)));

            let sheets_dir = &builder_set.spritesheets_dir;
            for (id, sheet) in builder_set.spritesheet_builders {
                insert_if_ok_boxed("spritesheet", id, Spritesheet::new(sheets_dir, sheet),
                    &mut resource_set.spritesheets);
            }

            for (id, image) in builder_set.simple_builders {
                insert_if_ok_boxed("image", id, SimpleImage::new(image, &resource_set),
                    &mut resource_set.images);
            }

            for (id, image) in builder_set.composed_builders {
                insert_if_ok_boxed("image", id, ComposedImage::new(image,
                    &resource_set.images), &mut resource_set.images);
            }

            for (id, image) in builder_set.animated_builders {
                insert_if_ok_boxed("image", id, AnimatedImage::new(image,
                    &resource_set.images), &mut resource_set.images);
            }

            for (id, adj) in builder_set.item_adjectives {
                trace!("Inserting resource of type item_adjective with key {} \
                    into resource set.", id);
                resource_set.item_adjectives.insert(id, Rc::new(adj));
            }

            for (_id_str, builder) in builder_set.size_builders {
                insert_if_ok("size", builder.size, Size::new(builder),
                    &mut resource_set.sizes);
            }

            for (id, builder) in builder_set.tile_builders {
                insert_if_ok("tile", id, Tile::new(builder),
                    &mut resource_set.tiles);
            }

            for (id, builder) in builder_set.item_builders.into_iter() {
                insert_if_ok("item", id, Item::new(builder, &resource_set.images),
                    &mut resource_set.items);
            }

            for (id, builder) in builder_set.actor_builders.into_iter() {
                insert_if_ok("actor", id, Actor::new(builder, &resource_set),
                    &mut resource_set.actors);
            }

            for (id, builder) in builder_set.area_builders {
                insert_if_ok("area", id, Area::new(builder, &resource_set),
                    &mut resource_set.areas);
            }
        });

        Ok(())
    }

    fn get_resource<V: ?Sized>(&self, id: &str,
        map: &HashMap<String, Rc<V>>) -> Option<Rc<V>> {

        let resource = map.get(id);

        match resource {
            None => None,
            Some(r) => Some(Rc::clone(r)),
        }
    }

    pub fn get_theme() -> Rc<Theme> {
        RESOURCE_SET.with(|r| Rc::clone(r.borrow().theme.as_ref().unwrap()))
    }

    pub fn get_game() -> Rc<Game> {
        RESOURCE_SET.with(|r| Rc::clone(r.borrow().game.as_ref().unwrap()))
    }

    pub fn get_actor(id: &str) -> Option<Rc<Actor>> {
        RESOURCE_SET.with(|r| r.borrow().get_resource(id, &r.borrow().actors))
    }

    pub fn get_area(id: &str) -> Option<Rc<Area>> {
        RESOURCE_SET.with(|r| r.borrow().get_resource(id, &r.borrow().areas))
    }

    pub fn get_spritesheet(id: &str) -> Option<Rc<Spritesheet>> {
        RESOURCE_SET.with(|r| r.borrow().get_resource(id, &r.borrow().spritesheets))
    }

    pub fn get_image(id: &str) -> Option<Rc<Image>> {
        RESOURCE_SET.with(|r| r.borrow().get_resource(id, &r.borrow().images))
    }

    pub fn get_size(id: usize) -> Option<Rc<Size>> {
        RESOURCE_SET.with(|r| {
            let r = r.borrow();
            let size = r.sizes.get(&id);

            match size {
                None => None,
                Some(s) => Some(Rc::clone(s)),
            }
        })
    }

    pub fn get_tile(id: &str) -> Option<Rc<Tile>> {
        RESOURCE_SET.with(|r| r.borrow().get_resource(id, &r.borrow().tiles))
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
