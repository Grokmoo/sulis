use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use resource::Tile;
use resource::generator;
use resource::AreaBuilder;

pub struct Terrain {
    pub width: usize,
    pub height: usize,
    layer: Vec<Rc<Tile>>,
}

impl Terrain {
    pub fn new(builder: &AreaBuilder,
               tiles: &HashMap<String, Rc<Tile>>) -> Result<Terrain, Error> {
       let width = builder.width;
       let height = builder.height;

        let layer = if builder.generate {
            let layer = generator::generate_area(tiles, width, height);

            match layer {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Unable to generate terrain for area '{}'", builder.id);
                    eprintln!("  {}", e);
                    return Err(e);
                }
            }
        } else {
            let mut layer: Vec<Option<Rc<Tile>>> =
                vec![None;width * height];

            for (terrain_type, locations) in &builder.terrain {
                for point in locations.iter() {
                    let x = point[0];
                    let y = point[1];

                    let cell = layer.get_mut(x + y * width).unwrap();

                    let tile_ref = tiles.get(terrain_type);
                    if let Some(tile) = tile_ref {
                        *cell = Some(Rc::clone(tile));
                    }
                }
            }

            for (index, tile) in layer.iter().enumerate() {
                if let None = *tile {
                    eprintln!("Unable to generate terrain for '{}'", builder.id);
                    let x = index % width;
                    let y = index / width;
                    return Err(
                        Error::new(ErrorKind::InvalidData,
                                   format!("Terrain is empty at position [{}, {}]", x, y))
                        );
                }
            }

            layer.into_iter().map(|tile| tile.unwrap()).collect() 
        };

        Ok(Terrain {
            layer,
            width,
            height
        })
    }

    pub fn at(&self, x: usize, y: usize) -> Rc<Tile> {
        Rc::clone(self.layer.get(x + y * self.width).unwrap())
    }

    pub fn display_at(&self, x: usize, y: usize) -> char {
        self.layer.get(x + y * self.width).unwrap().display
    }

    pub fn display(&self, index: usize) -> char {
        self.layer.get(index).unwrap().display
    }
}
