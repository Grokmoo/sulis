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
//  You should have received a copy of the GNU General Public License//
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::io::{Error, ErrorKind};
use std::rc::Rc;

use grt::util::Point;
use grt::resource::{Sprite, ResourceBuilder, ResourceSet};
use grt::serde_json;
use grt::serde_yaml;

#[derive(Debug)]
pub struct Tile {
    pub id: String,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub layer: String,
    pub image_display: Rc<Sprite>,
    pub impass: Vec<Point>,
}

impl Tile {
    pub fn new(builder: TileBuilder) -> Result<Tile, Error> {
        let mut impass_points: Vec<Point> = Vec::new();

        for p in builder.impass.into_iter() {
            // allow an empty vector (no impass points)
            if p.len() == 0 { continue; }

            if p.len() != 2 {
                return Err(Error::new(ErrorKind::InvalidData,
                                      "Impass point array length is not equal to 2."));
            }
            let x = *p.get(0).unwrap();
            let y = *p.get(1).unwrap();
            if x >= builder.width || y >= builder.height {
                return Err(
                    Error::new(ErrorKind::InvalidData,
                               format!("Impass point has coordinate greater than size '{}, {}'",
                                       builder.width, builder.height))
                    );
            }

            impass_points.push(Point::new(x as i32, y as i32));
        }

        let sprite = ResourceSet::get_sprite(&builder.image_display)?;

        Ok(Tile {
            id: builder.id,
            name: builder.name,
            layer: builder.layer,
            width: builder.width as i32,
            height: builder.height as i32,
            image_display: sprite,
            impass: impass_points,
        })
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        self.id == other.id
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileBuilder {
    pub id: String,
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub layer: String,
    image_display: String,
    pub impass: Vec<Vec<usize>>,
}

impl ResourceBuilder for TileBuilder {
    fn owned_id(&self) -> String {
        self.id.to_owned()
    }

    fn from_json(data: &str) -> Result<TileBuilder, Error> {
        let resource: TileBuilder = serde_json::from_str(data)?;

        Ok(resource)
    }

    fn from_yaml(data: &str) -> Result<TileBuilder, Error> {
        let resource: Result<TileBuilder, serde_yaml::Error> = serde_yaml::from_str(data);

        match resource {
            Ok(resource) => Ok(resource),
            Err(error) => Err(Error::new(ErrorKind::InvalidData, format!("{}", error)))
        }
    }
}
