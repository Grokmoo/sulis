use std::io::{Error, ErrorKind};
use std::rc::Rc;

use module::{Module, Tile};

pub fn generate_area(width: i32, height: i32, module: &Module) -> Result<Vec<Option<Rc<Tile>>>, Error> {
    debug!("Generating area with size {},{}", width, height);
    let width = width as usize;
    let height = height as usize;
    // filter down to only tiles of size one
    let tiles: Vec<Rc<Tile>> = module.tiles.iter().filter(|&(_, tile)| {
        tile.width == 1 && tile.height == 1
    }).map(|(_, tile)| Rc::clone(tile)).collect();

    let (passable_tiles, impassable_tiles):
        (Vec<Rc<Tile>>, Vec<Rc<Tile>>) = tiles.into_iter().partition(|ref tile| tile.impass.len() == 0);
    trace!("Got passable tiles {:?}", passable_tiles);
    trace!("Got impassable tiles {:?}", impassable_tiles);

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

    trace!("Done generation of area.");

    Ok(terrain)
}
