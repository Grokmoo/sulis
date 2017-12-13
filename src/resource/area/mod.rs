mod path_finder_grid;
use self::path_finder_grid::PathFinderGrid;

use std::collections::HashMap;
use std::io::Error;
use std::rc::Rc;

use resource::ResourceBuilder;
use resource::Tile;
use resource::Terrain;
use resource::Size;

use serde_json;

pub struct Area {
    pub id: String,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub terrain: Terrain,
    path_grids: HashMap<i32, PathFinderGrid>,
}

impl PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.id == other.id
    }
}

impl Area {
    pub fn new(builder: AreaBuilder, tiles: &HashMap<String, Rc<Tile>>,
               sizes: &HashMap<usize, Rc<Size>>) -> Result<Area, Error> {
        debug!("Creating area {}", builder.id);
        let terrain = Terrain::new(&builder, tiles);
        let terrain = match terrain {
            Ok(l) => l,
            Err(e) => {
                warn!("Unable to generate terrain for area '{}'", builder.id);
                return Err(e);
            }
        };

        let mut path_grids: HashMap<i32, PathFinderGrid> = HashMap::new();
        for size in sizes.values() {
            let path_grid = PathFinderGrid::new(Rc::clone(size), &terrain);
            trace!("Generated path grid of size {}", size.size);
            path_grids.insert(size.size, path_grid);
        }

        Ok(Area {
            id: builder.id,
            name: builder.name,
            width: builder.width as i32,
            height: builder.height as i32,
            terrain: terrain,
            path_grids: path_grids,
        })
    }

    pub fn coords_valid(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 { return false; }
        if x >= self.width || y >= self.height { return false; }

        true
    }

    pub fn get_path_grid(&self, size: i32) -> &PathFinderGrid {
        self.path_grids.get(&size).unwrap()
    }
}

#[derive(Deserialize, Debug)]
pub struct AreaBuilder {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub terrain: HashMap<String, Vec<Vec<usize>>>,
    pub generate: bool,
}

impl ResourceBuilder for AreaBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<AreaBuilder, Error> {
        let builder: AreaBuilder = serde_json::from_str(data)?;

        Ok(builder)
    }
}
