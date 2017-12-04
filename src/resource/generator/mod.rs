use resource::Tile;

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

pub fn generate_area(tiles: &HashMap<String, Rc<Tile>>,
                     width: usize, height: usize) -> Result<Vec<Option<Rc<Tile>>>, Error> {

    // filter down to only tiles of size one
    let tiles: HashMap<String, Rc<Tile>> = tiles.iter().filter_map( |(id, tile)| {
        if tile.width == 1 && tile.height == 1 {
            Some((id.clone(), Rc::clone(tile)))
        } else {
            None
        }
    }).collect();

    let (passable_tiles, impassable_tiles):
        (Vec<Rc<Tile>>, Vec<Rc<Tile>>) = tiles.values().
         map(|tile| Rc::clone(tile)).partition(|tile| tile.impass.len() == 0);

    if passable_tiles.len() == 0 {
        return Err(Error::new(ErrorKind::InvalidData,
                              "Found no passable tiles to generate terrain."))
    }

    if impassable_tiles.len() == 0 {
        return Err(Error::new(ErrorKind::InvalidData,
                              "Found no impassable tiles to generate terrain."))
    }

    // set up terrain array as passable
    let mut terrain: Vec<Option<Rc<Tile>>> =
        vec![Some(Rc::clone(passable_tiles.first().unwrap()));width * height];

    // generate impassable borders
    let impass = impassable_tiles.first().unwrap();
    for x in 0..width {
        { let cell = terrain.get_mut(x + 0 * width).unwrap();
        *cell = Some(Rc::clone(impass)); }

        let cell = terrain.get_mut(x + (height - 1) * width).unwrap();
        *cell = Some(Rc::clone(impass));
    }

    for y in 0..height {
        { let cell = terrain.get_mut(0 + y * width).unwrap();
        *cell = Some(Rc::clone(impass)); }

        let cell = terrain.get_mut(width - 1 + y * width).unwrap();
        *cell = Some(Rc::clone(impass));
    }

    Ok(terrain)
}
