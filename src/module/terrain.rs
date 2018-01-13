use std::io::{Error, ErrorKind};
use std::rc::Rc;

use grt::resource::{ResourceSet, Sprite, Spritesheet};
use grt::util::invalid_data_error;

use module::area::AreaBuilder;
use module::{Module, Tile, generator};

pub struct Terrain {
    pub width: i32,
    pub height: i32,
    text_display: Vec<char>,
    image_display: Vec<Option<Rc<Sprite>>>,
    tiles: Vec<Option<Rc<Tile>>>,
    passable: Vec<bool>,
    spritesheet_id: String,
}

impl Terrain {
    pub fn new(builder: &AreaBuilder, module: &Module) -> Result<Terrain, Error> {
        let width = builder.width as i32;
        let height = builder.height as i32;
        let layer = if builder.generate {
            let layer = generator::generate_area(width, height, module);

            match layer {
                Ok(l) => l,
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            if let Err(e) = Terrain::validate_tiles(builder, module) {
                return Err(e);
            }

            let mut layer: Vec<Option<Rc<Tile>>> = vec![None;(width * height) as usize];

            for (terrain_type, locations) in &builder.terrain {
                let tile_ref = module.tiles.get(terrain_type).unwrap();

                for point in locations.iter() {
                    *layer.get_mut(point[0] + point[1] * width as usize).unwrap() =
                        Some(Rc::clone(&tile_ref));
                }
            }

            layer
        };

        if let Err(e) = Terrain::validate_layer(&layer, width, height) {
            return Err(e);
        }

        let mut spritesheet_id: Option<String> = None;
        let mut tiles: Vec<Option<Rc<Tile>>> = vec![None;(width * height) as usize];
        let mut image_display: Vec<Option<Rc<Sprite>>> = vec![None;(width * height) as usize];
        let mut text_display = vec![' ';(width * height) as usize];
        let mut passable = vec![true;(width * height) as usize];
        for (index, tile) in layer.iter().enumerate() {
            let index = index as i32;
            if let None = *tile { continue; }

            let tile = match tile {
                &None => continue,
                &Some(ref t) => Rc::clone(&t),
            };

            match spritesheet_id {
                None => spritesheet_id = Some(tile.image_display.id.to_string()),
                Some(ref id) => {
                    if id != &tile.image_display.id {
                        return invalid_data_error(&format!("All tiles in a layer must be from the \
                                                          same spritesheet: '{}' vs '{}'", id, tile.id));
                    }
                }
            }

            let base_x = index % width;
            let base_y = index / width;

            *tiles.get_mut((base_x + base_y * width) as usize).unwrap() =
                Some(Rc::clone(&tile));
            *image_display.get_mut((base_x + base_y * width) as usize).unwrap() =
                Some(Rc::clone(&tile.image_display));

            for y in 0..tile.height {
                for x in 0..tile.width {
                    *text_display.get_mut((base_x + x + (base_y + y) * width) as usize).unwrap() =
                        tile.get_text_display(x, y);
                }
            }

            for p in tile.impass.iter() {
                *passable.get_mut((base_x + p.x + (base_y + p.y) * width) as usize).unwrap() = false;
            }
        }

        let spritesheet_id = match spritesheet_id {
            None => return invalid_data_error("Empty terrain"),
            Some(id) => id,
        };

        Ok(Terrain {
            width,
            height,
            tiles,
            text_display,
            image_display,
            passable,
            spritesheet_id,
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

    fn validate_tiles(builder: &AreaBuilder, module: &Module) -> Result<(), Error> {
        for (tile_id, locations) in &builder.terrain {
            let tile_ref = module.tiles.get(tile_id);
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

    pub fn get_spritesheet(&self) -> Rc<Spritesheet> {
        ResourceSet::get_spritesheet(&self.spritesheet_id).unwrap()
    }

    pub fn is_passable(&self, x: i32, y: i32) -> bool {
        *self.passable.get((x + y * self.width) as usize).unwrap()
    }

    pub fn tile_at(&self, x: i32, y: i32) -> &Option<Rc<Tile>> {
        self.tiles.get((x + y * self.width) as usize).unwrap()
    }

    pub fn image_at(&self, x: i32, y: i32) -> &Option<Rc<Sprite>> {
        self.image_display.get((x + y * self.width) as usize).unwrap()
    }

    pub fn display_at(&self, x: i32, y: i32) -> char {
        *self.text_display.get((x + y * self.width) as usize).unwrap()
    }

    pub fn display(&self, index: usize) -> char {
        *self.text_display.get(index).unwrap()
    }
}
