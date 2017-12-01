use resource::Tile;

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

pub fn generate_area(tiles: &HashMap<String, Rc<Tile>>,
                     width: usize, height: usize) -> Result<Vec<Vec<Option<Rc<Tile>>>>, Error> {

    let mut terrain: Vec<Vec<Option<Rc<Tile>>>> = vec![vec![None;width];height];

    let (passable_tiles, impassable_tiles):
        (Vec<Rc<Tile>>, Vec<Rc<Tile>>) = tiles.values().
         map(|tile| Rc::clone(tile)).partition(|tile| tile.passable);

    if passable_tiles.len() == 0 {
        return Err(Error::new(ErrorKind::InvalidData,
                              "Found no passable tiles to generate terrain."))
    }

    if impassable_tiles.len() == 0 {
        return Err(Error::new(ErrorKind::InvalidData,
                              "Found no impassable tiles to generate terrain."))
    }

    // set passable for all tiles
    for y in 0..height {
        for x in 0..width {
            let cell = terrain.get_mut(y).unwrap().get_mut(x).unwrap();
            *cell = Some(Rc::clone(passable_tiles.first().unwrap()));
        }
    }

    // impassable generate borders
    for x in 0..width {
        { let cell = terrain.get_mut(0).unwrap().get_mut(x).unwrap();
        *cell = Some(Rc::clone(impassable_tiles.first().unwrap())); }

        let cell = terrain.get_mut(height - 1).unwrap().get_mut(x).unwrap();
        *cell = Some(Rc::clone(impassable_tiles.first().unwrap()));
    }

    for y in 0..height {
        { let cell = terrain.get_mut(y).unwrap().get_mut(0).unwrap();
        *cell = Some(Rc::clone(impassable_tiles.first().unwrap())); }

        let cell = terrain.get_mut(y).unwrap().get_mut(width - 1).unwrap();
        *cell = Some(Rc::clone(impassable_tiles.first().unwrap()));
    }

    Ok(terrain)
}
