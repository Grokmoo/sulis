use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use resource::{AreaBuilder, Sprite, Tile};
use resource::generator;
use util::Point;
use ui::Size;

pub struct Terrain {
    pub width: i32,
    pub height: i32,
    text_display: Vec<char>,
    image_display: Vec<Rc<Sprite>>,
    passable: Vec<bool>,
}

impl Terrain {
    pub fn new(builder: &AreaBuilder,
               tiles: &HashMap<String, Rc<Tile>>) -> Result<Terrain, Error> {
        let width = builder.width as i32;
        let height = builder.height as i32;
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

            let mut layer: Vec<Option<Rc<Tile>>> = vec![None;(width * height) as usize];

            for (terrain_type, locations) in &builder.terrain {
                let tile_ref = tiles.get(terrain_type).unwrap();

                for point in locations.iter() {
                    *layer.get_mut(point[0] + point[1] * width as usize).unwrap() =
                        Some(Rc::clone(tile_ref));
                }
            }

            layer
        };

        if let Err(e) = Terrain::validate_layer(&layer, width, height) {
            return Err(e);
        }

        let empty = Rc::new(Sprite {
            id: String::new(),
            position: Point::as_zero(),
            size: Size::as_zero(),
            tex_coords: [0.0;8],
        });

        let mut image_display: Vec<Rc<Sprite>> = vec![Rc::clone(&empty);(width * height) as usize];
        let mut text_display = vec![' ';(width * height) as usize];
        let mut passable = vec![true;(width * height) as usize];
        for (index, tile) in layer.iter().enumerate() {
            let index = index as i32;
            if let None = *tile { continue; }

            let tile = match tile {
                &None => continue,
                &Some(ref t) => Rc::clone(&t),
            };

            let base_x = index % width;
            let base_y = index / width;

            for y in 0..tile.height {
                for x in 0..tile.width {
                    *text_display.get_mut((base_x + x + (base_y + y) * width) as usize).unwrap() =
                        tile.get_text_display(x, y);

                   *image_display.get_mut((base_x + x + (base_y + y) * width) as usize).unwrap() =
                        Rc::clone(&tile.image_display);
                }
            }

            for p in tile.impass.iter() {
                *passable.get_mut((base_x + p.x + (base_y + p.y) * width) as usize).unwrap() = false;
            }
        }

        Ok(Terrain {
            width,
            height,
            text_display,
            image_display,
            passable,
        })
    }

    fn validate_layer(layer: &Vec<Option<Rc<Tile>>>, width: i32,
                      height: i32) -> Result<(), Error> {

        let mut refed_by_tile = vec![false;(width * height) as usize];

        for (index, tile) in layer.iter().enumerate() {
            let index = index as i32;
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
                    let already_used = refed_by_tile.get_mut((x + y *width) as usize).unwrap();

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
                let refed = refed_by_tile.get((x + y * width) as usize).unwrap();

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

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        *self.passable.get((x + y * self.width) as usize).unwrap()
    }

    pub fn image_at(&self, x: i32, y: i32) -> &Rc<Sprite> {
        self.image_display.get((x + y * self.width) as usize).unwrap()
    }

    pub fn display_at(&self, x: i32, y: i32) -> char {
        *self.text_display.get((x + y * self.width) as usize).unwrap()
    }

    pub fn display(&self, index: usize) -> char {
        *self.text_display.get(index).unwrap()
    }
}
