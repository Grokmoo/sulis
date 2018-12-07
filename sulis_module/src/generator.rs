//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::io::{Error, ErrorKind};
use std::rc::Rc;

use crate::Module;
use crate::area::Tile;

pub fn generate_area(width: i32, height: i32, module: &Module) ->
        Result<(String, Vec<Vec<Rc<Tile>>>), Error> {
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

    let terrain: Vec<Vec<Rc<Tile>>> = vec![Vec::new();width * height];

    // TODO implement area generate

    let id = passable_tiles[0].layer.to_string();

    trace!("Done generation of area.");

    Ok((id, terrain))
}
