use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use resource::ResourceBuilder;
use resource::Tile;
use resource::Terrain;

use serde_json;

pub struct Area {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub terrain: Terrain,
}

impl PartialEq for Area {
    fn eq(&self, other: &Area) -> bool {
        self.id == other.id
    }
}

impl Area {
    pub fn new(builder: AreaBuilder, tiles: &HashMap<String,
               Rc<Tile>>) -> Result<Area, Error> {
        let terrain = Terrain::new(&builder, tiles);
        let terrain = match terrain {
            Ok(l) => l,
            Err(e) => {
                return Err(e);
            }
        };
        
        Ok(Area {
            id: builder.id,
            name: builder.name,
            width: builder.width,
            height: builder.height,
            terrain: terrain,
        })
    }

    pub fn coords_valid(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height { return false; }

        true
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
