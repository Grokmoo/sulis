use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use resource::Tile;
use resource::generator;
use resource::AreaBuilder;

pub struct Terrain {
    pub width: usize,
    pub height: usize,
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
            if let Err(e) = Terrain::validate_tiles(builder, tiles) {
                return Err(e);
            }

            let mut layer: Vec<Option<Rc<Tile>>> = vec![None;width * height];

            for (terrain_type, locations) in &builder.terrain {
                let tile_ref = tiles.get(terrain_type).unwrap();

                for point in locations.iter() {
                    *layer.get_mut(point[0] + point[1] * width).unwrap() = Some(Rc::clone(tile_ref));
                }
            }

            layer
        };
        
        if let Err(e) = Terrain::validate_layer(&layer, width, height) {
            return Err(e);
        }

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
            width,
            height,
            text_display,
            passable,
        })
    }

    fn validate_layer(layer: &Vec<Option<Rc<Tile>>>, width: usize,
                      height: usize) -> Result<(), Error> {

        let mut refed_by_tile = vec![false;width * height];
        
        for (index, tile) in layer.iter().enumerate() {
            if let None = *tile { continue; }

            let tile = match tile {
                &None => continue,
                &Some(ref t) => Rc::clone(&t),
            };

            let tile_x = index % width;
            let tile_y = index / width;

            if tile_x + tile.width > width || tile_y + tile.height > height {
                return Err(
                    Error::new(ErrorKind::InvalidData,
                               format!("Tile '{}' at [{}, {}] extends past area boundary.",
                                       tile.id, tile_x, tile_y))
                    );
            }

            for x in tile_x..(tile_x + tile.width) {
                for y in tile_y..(tile_y + tile.height) {
                    let already_used = refed_by_tile.get_mut(x + y *width).unwrap();
                    if *already_used {
                        return Err(
                            Error::new(ErrorKind::InvalidData,
                                       format!("Tile '{}' at [{}, {}] uses point [{}, {}], but that point has already been used by another tile.", tile.id, tile_x, tile_y, x, y))
                            );
                    }

                    *already_used = true;
                }
            }
        }

        for y in 0..height {
            for x in 0..width {
                let refed = refed_by_tile.get(x + y * width).unwrap();

                if !refed {
                    return Err(
                        Error::new(ErrorKind::InvalidData,
                                   format!("Point at [{}, {}] is empty.", x, y))
                        );
                }
            }
        }

        Ok(())
    }

    fn validate_tiles(builder: &AreaBuilder,
                      tiles: &HashMap<String, Rc<Tile>>) -> Result<(), Error> {
        for (tile_id, locations) in &builder.terrain {
            let tile_ref = tiles.get(tile_id);
            match tile_ref {
                Some(t) => t,
                None => {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          format!("Tile not found '{}'", tile_id)));
                }
            };

            for point in locations.iter() {
                if point.len() != 2 {
                    return Err(
                        Error::new(ErrorKind::InvalidData,
                                   format!("Point array is not 2 coordinates in '{}'", tile_id))
                        );
                }
            }

        }

        Ok(())
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
