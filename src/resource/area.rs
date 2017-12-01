use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use resource::ResourceBuilder;
use resource::Tile;
use resource::generator;

use serde_json;

pub struct Area {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub terrain: Vec<Vec<Option<Rc<Tile>>>>,
}

impl PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.id == other.id
    }
}

impl Area {
    pub fn new(builder: AreaBuilder, tiles: &HashMap<String,
               Rc<Tile>>) -> Result<Area, Error> {
        let terrain = if builder.generate {
            let terrain = generator::generate_area(tiles, builder.width, builder.height);

            match terrain {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Unable to generate area '{}'", builder.id);
                    eprintln!("  {}", e);
                    return Err(e);
                }
            }
        } else {
            let mut terrain: Vec<Vec<Option<Rc<Tile>>>> =
                vec![vec![None;builder.width as usize];builder.height as usize];

            for (terrain_type, locations) in &builder.terrain {
                for point in locations.iter() {
                    let x = point[0];
                    let y = point[1];

                    let row = terrain.get_mut(y).unwrap();
                    let cell = row.get_mut(x).unwrap();

                    let tile_ref = tiles.get(terrain_type);
                    if let Some(tile) = tile_ref {
                        *cell = Some(Rc::clone(tile));
                    }
                }
            }

            terrain
        }; 

        Ok(Area {
            id: builder.id,
            name: builder.name,
            width: builder.width,
            height: builder.height,
            terrain: terrain,
        })
    }
    
    pub fn terrain_display_at(&self, x: usize, y: usize) -> char {
        match self.terrain_at(x, y) {
            Some(ref tile) => {
                (*tile).display
            },
            None => ' '
        }
    }

    pub fn terrain_at(&self, x: usize, y: usize) -> Option<Rc<Tile>> {
        if let Some(row) = self.terrain.get(y) {
            if let Some(cell) = row.get(x) {
                if let Some(ref tile) = *cell {
                    return Some(Rc::clone(tile));
                }
            }
        }

        None
    }

    pub fn coords_valid(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height { return false; }

        true
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AreaBuilder {
    id: String,
    name: String,
    width: usize,
    height: usize,
    terrain: HashMap<String, Vec<Vec<usize>>>,
    generate: bool,
}

impl ResourceBuilder for AreaBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn new(data: &str) -> Result<AreaBuilder, Error> {
        let builder: AreaBuilder = serde_json::from_str(data)?;

        let mut terrain: Vec<Vec<u32>> =
            vec![vec![0;builder.width as usize];builder.height as usize];

        for (_terrain_type, locations) in &builder.terrain {
            for point in locations.iter() {
                let x = point[0];
                let y = point[1];

                let row = terrain.get_mut(y).unwrap();
                let cell = row.get_mut(x).unwrap();

                if *cell > 0 {
                    let msg = format!("Multiple terrain references to cell {},{}", x, y);
                    return Err(Error::new(ErrorKind::AlreadyExists, msg));
                }

                *cell += 1;
            }
        }
        Ok(builder)
    }
}
