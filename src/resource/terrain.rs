use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use resource::Tile;
use resource::generator;
use resource::AreaBuilder;

pub struct Terrain {
    pub width: usize,
    pub height: usize,
    _layer: Vec<Option<Rc<Tile>>>,
    text_display: Vec<char>,
    passable: Vec<bool>,
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
                    return Err(e);
                }
            }
        } else {
            let mut layer: Vec<Option<Rc<Tile>>> =
                vec![None;width * height];

            for (terrain_type, locations) in &builder.terrain {
                let tile_ref = tiles.get(terrain_type);
                let tile_ref = match tile_ref {
                    Some(t) => t,
                    None => {
                        return Err(Error::new(ErrorKind::InvalidData,
                                    format!("Tile not found '{}'", terrain_type)));
                    }
                };
                
                for point in locations.iter() {
                    if point.len() != 2 {
                        return Err(
                            Error::new(ErrorKind::InvalidData,
                                format!("Point array is not 2 coordinates in '{}'", terrain_type))
                            );
                    }

                    let x = point[0];
                    let y = point[1];
                    *layer.get_mut(x + y * width).unwrap() = Some(Rc::clone(tile_ref));
                }
            }
            
            layer
        };

        let mut text_display = vec![' ';width * height];
        let mut passable = vec![true;width * height];
        for (index, tile) in layer.iter().enumerate() {
            if let None = *tile { continue; }

            let tile = match tile {
                &None => continue,
                &Some(ref t) => Rc::clone(&t),
            };

            let base_x = index % width;
            let base_y = index / width;

            for y in 0..tile.height {
                for x in 0..tile.width {
                    *text_display.get_mut(base_x + x + (base_y + y) * width).unwrap() =
                        tile.get_text_display(x, y);
                }
            }

            for p in tile.impass.iter() {
                *passable.get_mut(base_x + p.x + (base_y + p.y) * width).unwrap() = false;
            }
        }

        Ok(Terrain {
            _layer: layer,
            width,
            height,
            text_display,
            passable,
        })
    }

    pub fn is_passable(&self, x: usize, y: usize) -> bool {
        *self.passable.get(x + y * self.width).unwrap()
    }

    pub fn display_at(&self, x: usize, y: usize) -> char {
        *self.text_display.get(x + y * self.width).unwrap()
    }

    pub fn display(&self, index: usize) -> char {
        *self.text_display.get(index).unwrap()
    }
}
