use resource::size::SizeBuilder;
use resource::Game;
use resource::TileBuilder;
use resource::ActorBuilder;
use resource::ResourceBuilder;
use resource::area::AreaBuilder;
use resource::image::SimpleImage;
use resource::image::composed_image::ComposedImageBuilder;
use resource::image::animated_image::AnimatedImageBuilder;
use ui::theme::ThemeBuilder;

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Error};
use std::ffi::OsStr;

#[derive(Debug)]
pub struct ResourceBuilderSet {
    pub game: Game,
    pub theme_builder: ThemeBuilder,
    pub size_builders: HashMap<String, SizeBuilder>,
    pub area_builders: HashMap<String, AreaBuilder>,
    pub tile_builders: HashMap<String, TileBuilder>,
    pub actor_builders: HashMap<String, ActorBuilder>,
    pub simple_images: HashMap<String, SimpleImage>,
    pub composed_builders: HashMap<String, ComposedImageBuilder>,
    pub animated_builders: HashMap<String, AnimatedImageBuilder>,
}

impl ResourceBuilderSet {
    pub fn new(root: &str) -> Result<ResourceBuilderSet, Error> {
        let game_filename = root.to_owned() + "/game.json";
        debug!("Reading top level config from {}", game_filename);
        let game = match ResourceBuilderSet::create_game(&game_filename) {
            Ok(g) => g,
            Err(e) => {
                error!("Unable to load game startup state from {}", game_filename);
                return Err(e);
            }
        };

        let theme_filename = root.to_owned() + "/theme.json";
        debug!("Reading theme from {}", theme_filename);
        let theme_builder = match ResourceBuilderSet::create_theme(&theme_filename) {
            Ok(t) => t,
            Err(e) => {
                error!("Unable to load theme from {}", theme_filename);
                return Err(e);
            }
        };

        Ok(ResourceBuilderSet {
            game,
            theme_builder,
            size_builders: read_resources(&format!("{}/sizes/", root)),
            tile_builders: read_resources(&format!("{}/tiles/", root)),
            actor_builders: read_resources(&format!("{}/actors/", root)),
            area_builders: read_resources(&format!("{}/areas/", root)),
            simple_images: read_resources(&format!("{}/images/simple/", root)),
            composed_builders: read_resources(&format!("{}/images/composed/", root)),
            animated_builders: read_resources(&format!("{}/images/animated/", root)),
        })
    }

    pub fn create_game(filename: &str) -> Result<Game, Error> {
        let mut f = File::open(filename)?;
        let mut file_data = String::new();
        f.read_to_string(&mut file_data)?;
        let game = Game::new(&file_data)?;

        Ok(game)
    }

    pub fn create_theme(filename: &str) -> Result<ThemeBuilder, Error> {
        let mut f = File::open(filename)?;
        let mut file_data = String::new();
        f.read_to_string(&mut file_data)?;
        let theme = ThemeBuilder::new(&file_data)?;

        Ok(theme)
    }
}

fn read_resources<T: ResourceBuilder>(dir: &str) -> HashMap<String, T> {
    debug!("Reading resources from {}", dir);
    let mut resources: HashMap<String, T> = HashMap::new();
    let dir_entries = fs::read_dir(dir);
    let dir_entries = match dir_entries {
        Ok(entries) => entries,
        Err(_) => {
            warn!("Unable to read directory: {}", dir);
            return resources;
        }
    };

    for entry in dir_entries {
        trace!("Found entry {:?}", entry);
        let entry = match entry {
            Ok(e) => e,
            Err(error) => {
                warn!("Error reading file: {}", error);
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
            debug!("Reading file at {}", path_str);
            let f = File::open(path);
            let mut f = match f {
                Ok(file) => file,
                Err(error) => {
                    warn!("Error reading file: {}", error);
                    continue;
                }
            };

            let mut file_data = String::new();
            if f.read_to_string(&mut file_data).is_err() {
                warn!("Error reading file data from file");
                continue;
            }
            trace!("Read file data.");

            let resource = T::new(&file_data);
            let resource = match resource {
                Ok(a) => a,
                Err(error) => {
                    warn!("Error parsing file data: {:?}", path_str);
                    warn!("  {}", error);
                    continue;
                }
            };

            let id = resource.owned_id();

            trace!("Created resource '{}'", id);
            if resources.contains_key(&id) {
                warn!("Duplicate resource key: {} in {}", id, dir);
                continue;
            }

            trace!("Inserted resource.");
            resources.insert(id, resource);
        }
    }

    resources
}
