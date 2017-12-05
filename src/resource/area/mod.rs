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
    pub width: usize,
    pub height: usize,
    pub terrain: Terrain,
    path_grids: HashMap<usize, PathFinderGrid>,
}

impl PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.id == other.id
    }
}

impl Area {
    pub fn new(builder: AreaBuilder, tiles: &HashMap<String, Rc<Tile>>,
               sizes: &HashMap<usize, Rc<Size>>) -> Result<Area, Error> {
        let terrain = Terrain::new(&builder, tiles);
        let terrain = match terrain {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Unable to generate terrain for area '{}'", builder.id);
                return Err(e);
            }
        };
        
        let mut path_grids: HashMap<usize, PathFinderGrid> = HashMap::new();
        for size in sizes.values() {
            path_grids.insert(size.size, PathFinderGrid::new(Rc::clone(size), &terrain));
        }

        Ok(Area {
            id: builder.id,
            name: builder.name,
            width: builder.width,
            height: builder.height,
            terrain: terrain,
            path_grids: path_grids,
        })
    }

    pub fn coords_valid(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height { return false; }

        true
    }

    pub fn get_path_grid(&self, size: usize) -> &PathFinderGrid {
        self.path_grids.get(&size).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
