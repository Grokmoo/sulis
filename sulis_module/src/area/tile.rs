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

use std::collections::{HashMap, HashSet};
use std::io::Error;
use std::rc::Rc;

use crate::Module;
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::util::{gen_rand, invalid_data_error, unable_to_create_error, Point, Size};

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FeatureBuilder {
    pub entries: Vec<FeatureEntry>,
    pub size: Size,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct FeatureEntry {
    weight: u32,
    tiles: HashMap<String, Point>,
}

pub struct Feature {
    pub id: String,
    pub size: Size,
    entries: Vec<(Vec<(Rc<Tile>, Point)>, u32)>,
    pub preview: Vec<(Rc<Tile>, Point)>,
}

impl Feature {
    pub fn new(id: String, builder: FeatureBuilder, module: &Module) -> Result<Feature, Error> {
        let mut entries = Vec::new();
        for entry in builder.entries {
            let mut tiles = Vec::new();

            for (tile_id, p) in entry.tiles {
                let tile = match module.tiles.get(&tile_id) {
                    None => {
                        warn!("No tile '{}' found for feature '{}'", tile_id, id);
                        return unable_to_create_error("feature", &id);
                    }
                    Some(tile) => Rc::clone(tile),
                };

                tiles.push((tile, p));
            }

            entries.push((tiles, entry.weight));
        }

        if entries.is_empty() {
            warn!("Feature '{}' must have 1 or more entries.", id);
            return unable_to_create_error("feature", &id);
        }

        let preview = entries[0].0.clone();

        Ok(Feature {
            id,
            size: builder.size,
            entries,
            preview,
        })
    }

    pub fn rand_entry(&self) -> &[(Rc<Tile>, Point)] {
        if self.entries.len() == 1 {
            &self.entries[0].0
        } else {
            let index = gen_rand(0, self.entries.len());
            &self.entries[index].0
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct UniformSet {
    pub size: [usize; 2],
    pub layer: String,
    pub sprite_prefix: String,
    pub impass: Vec<Vec<usize>>,
    pub invis: Vec<Vec<usize>>,
    pub tiles: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NonUniformSet {
    pub size: [usize; 2],
    pub layer: String,
    pub sprite_prefix: String,
    pub tiles: HashMap<String, ImpassInvis>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ImpassInvis {
    pub impass: Option<Vec<Vec<usize>>>,
    pub invis: Option<Vec<Vec<usize>>>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TileBuilder {
    pub size: [usize; 2],
    pub layer: String,
    pub sprite: String,
    pub impass: Option<Vec<Vec<usize>>>,
    pub invis: Option<Vec<Vec<usize>>>,
    pub pass: Option<Vec<Vec<usize>>>,
    pub vis: Option<Vec<Vec<usize>>>,
    pub override_impass: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct WallRules {
    pub grid_width: u32,
    pub grid_height: u32,
    pub up_layers: Vec<String>,
    pub down_layers: Vec<String>,
    pub prefix: String,
    pub edges: EdgeRules,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct EdgeRules {
    pub inner_edge_postfix: String,
    pub outer_edge_postfix: String,
    pub n_postfix: String,
    pub s_postfix: String,
    pub e_postfix: String,
    pub w_postfix: String,
    pub ne_postfix: String,
    pub se_postfix: String,
    pub nw_postfix: String,
    pub sw_postfix: String,
    pub all_postfix: String,
    pub ne_sw_postfix: String,
    pub nw_se_postfix: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct WallKind {
    pub id: String,
    pub base_tile: String,
    pub fill_tile: Option<String>,
    pub extended: Vec<String>,
    pub interior_border: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TerrainRules {
    pub grid_width: u32,
    pub grid_height: u32,
    pub base_layer: String,
    pub border_layer: String,
    pub prefix: String,
    pub base_postfix: String,
    pub variant_postfix: String,
    pub base_weight: u32,
    pub edges: EdgeRules,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TerrainKind {
    pub id: String,
    pub variants: Vec<i32>,

    #[serde(default)]
    pub borders: HashMap<String, String>,

    pub base_weight: Option<u32>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Tileset {
    pub id: String,

    #[serde(default)]
    pub tiles: HashMap<String, TileBuilder>,

    #[serde(default)]
    pub uniform_sets: HashMap<String, UniformSet>,

    #[serde(default)]
    pub non_uniform_sets: HashMap<String, NonUniformSet>,

    #[serde(default)]
    pub terrain_kinds: Vec<TerrainKind>,

    pub terrain_rules: Option<TerrainRules>,
    pub wall_rules: Option<WallRules>,

    #[serde(default)]
    pub wall_kinds: Vec<WallKind>,

    #[serde(default)]
    pub features: HashMap<String, FeatureBuilder>,
}

impl Tileset {
    pub fn move_tiles(&mut self) {
        self.move_uniform();
        self.move_non_uniform();
    }

    fn move_uniform(&mut self) {
        for (_, ref uniform) in self.uniform_sets.iter() {
            let size = uniform.size;
            let layer = &uniform.layer;
            let prefix = &uniform.sprite_prefix;
            let impass = if uniform.impass.is_empty() {
                None
            } else {
                Some(uniform.impass.clone())
            };
            let invis = if uniform.invis.is_empty() {
                None
            } else {
                Some(uniform.invis.clone())
            };
            for tile_id in uniform.tiles.iter() {
                let id = format!("{}{}", prefix, tile_id);
                let tile = TileBuilder {
                    size,
                    layer: layer.to_string(),
                    sprite: id.to_string(),
                    impass: impass.clone(),
                    invis: invis.clone(),
                    pass: None,
                    vis: None,
                    override_impass: None,
                };

                self.tiles.insert(id, tile);
            }
        }
    }

    fn move_non_uniform(&mut self) {
        for (_, ref non_uniform) in self.non_uniform_sets.iter() {
            let size = non_uniform.size;
            let layer = &non_uniform.layer;
            let prefix = &non_uniform.sprite_prefix;
            for (tile_id, impass_invis) in non_uniform.tiles.iter() {
                let id = format!("{}{}", prefix, tile_id);
                let tile = TileBuilder {
                    size,
                    layer: layer.to_string(),
                    sprite: id.to_string(),
                    impass: impass_invis.impass.clone(),
                    invis: impass_invis.invis.clone(),
                    pass: None,
                    vis: None,
                    override_impass: None,
                };

                self.tiles.insert(id, tile);
            }
        }
    }
}

#[derive(Debug)]
pub struct Tile {
    pub id: String,
    pub width: i32,
    pub height: i32,
    pub layer: String,
    pub image_display: Rc<Sprite>,
    pub impass: Vec<Point>,
    pub invis: Vec<Point>,
    pub override_impass: bool,
}

impl Tile {
    pub fn new(id: String, builder: TileBuilder) -> Result<Tile, Error> {
        if builder.impass.is_some() && builder.pass.is_some() {
            warn!("Cannot specify both pass and impass for a tile");
            return unable_to_create_error("tile", &id);
        }

        if builder.invis.is_some() && builder.vis.is_some() {
            warn!("Cannot specify both vis and invis for a tile");
            return unable_to_create_error("tile", &id);
        }

        let (width, height) = (builder.size[0], builder.size[1]);
        let mut impass_points: Vec<Point> = Vec::new();
        let mut invis_points: Vec<Point> = Vec::new();

        if let Some(impass) = builder.impass {
            for p in impass {
                let (x, y) = verify_point("impass", width, height, p)?;
                impass_points.push(Point::new(x, y));
            }
        } else if let Some(pass) = builder.pass {
            let mut pass_points = HashSet::new();
            for p in pass {
                let (x, y) = verify_point("pass", width, height, p)?;
                pass_points.insert(Point::new(x, y));
            }

            for x in 0..width {
                for y in 0..height {
                    let p = Point::new(x as i32, y as i32);
                    if pass_points.contains(&p) {
                        continue;
                    }

                    impass_points.push(p);
                }
            }
        }

        if let Some(invis) = builder.invis {
            for p in invis {
                let (x, y) = verify_point("invis", width, height, p)?;
                invis_points.push(Point::new(x, y));
            }
        } else if let Some(vis) = builder.vis {
            let mut vis_points = HashSet::new();
            for p in vis {
                let (x, y) = verify_point("vis", width, height, p)?;
                vis_points.insert(Point::new(x, y));
            }

            for x in 0..width {
                for y in 0..height {
                    let p = Point::new(x as i32, y as i32);
                    if vis_points.contains(&p) {
                        continue;
                    }

                    invis_points.push(p);
                }
            }
        }

        let sprite = ResourceSet::sprite(&builder.sprite)?;

        Ok(Tile {
            id,
            layer: builder.layer,
            width: builder.size[0] as i32,
            height: builder.size[1] as i32,
            image_display: sprite,
            impass: impass_points,
            invis: invis_points,
            override_impass: builder.override_impass.unwrap_or(false),
        })
    }
}

pub fn verify_point(
    kind: &str,
    width: usize,
    height: usize,
    p: Vec<usize>,
) -> Result<(i32, i32), Error> {
    if p.len() != 2 {
        return invalid_data_error(&format!("{} point array length is not equal to 2", kind));
    }

    let x = p[0];
    let y = p[1];
    if x > width || y >= height {
        return invalid_data_error(&format!(
            "{} point has coordiantes greater than size {},{}",
            kind, width, height
        ));
    }
    Ok((x as i32, y as i32))
}

impl PartialEq for Tile {
    fn eq(&self, other: &Tile) -> bool {
        self.id == other.id
    }
}
