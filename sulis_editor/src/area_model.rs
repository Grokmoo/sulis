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

use std::rc::Rc;
use std::cmp;
use std::slice::Iter;
use std::collections::HashMap;

use sulis_core::config::CONFIG;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::resource::{ResourceSet, read_single_resource, write_to_file};
use sulis_core::ui::animation_state;
use sulis_core::util::Point;
use sulis_module::{Actor, Module, Prop};
use sulis_module::area::*;

pub (crate) const MAX_AREA_SIZE: i32 = 128;

pub struct AreaModel {
    tiles: Vec<(String, Vec<(Point, Rc<Tile>)>)>,
    actors: Vec<(Point, Rc<Actor>)>,
    props: Vec<PropData>,
    transitions: Vec<Transition>,
    elevation: Vec<u8>,

    id: String,
    name: String,
    filename: String,
}

impl AreaModel {
    pub fn new() -> AreaModel {
        let mut tiles = Vec::new();
        for ref layer_id in CONFIG.editor.area.layers.iter() {
            tiles.push((layer_id.to_string(), Vec::new()));
        }

        let elevation = vec![0;(MAX_AREA_SIZE * MAX_AREA_SIZE) as usize];

        AreaModel {
            tiles,
            elevation,
            actors: Vec::new(),
            props: Vec::new(),
            transitions: Vec::new(),
            id: CONFIG.editor.area.id.clone(),
            name: CONFIG.editor.area.name.clone(),
            filename: CONFIG.editor.area.filename.clone(),
        }
    }

    pub fn id(&self) -> &str { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn filename(&self) -> &str { &self.filename }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_string();
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn set_filename(&mut self, filename: &str) {
        self.filename = filename.to_string();
    }

    pub fn elevation(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 { return 0; }

        self.elevation[(x + y * MAX_AREA_SIZE) as usize]
    }

    pub fn set_elevation(&mut self, elev: u8, x: i32, y: i32) {
        if x < 0 || y < 0 { return; }

        self.elevation[(x + y * MAX_AREA_SIZE) as usize] = elev;
    }

    pub fn add_tile(&mut self, tile: Rc<Tile>, x: i32, y: i32) {
        if x < 0 || y < 0 { return; }

        let index = self.create_layer_if_missing(&tile.layer);
        self.tiles[index].1.push((Point::new(x, y), tile));
    }

    pub fn remove_tiles_within(&mut self, layer_id: &str, x: i32, y: i32, width: i32, height: i32) {
        for &mut (ref cur_layer_id, ref mut tiles) in self.tiles.iter_mut() {
            if layer_id != cur_layer_id { continue; }

            tiles.retain(|&(pos, ref tile)| {
                !is_removal(pos, tile.width, tile.height, x, y, width, height)
            });
        }
    }

    pub fn tiles_within(&mut self, layer_id: &str, x: i32, y: i32,
                        width: i32, height: i32) -> Vec<(Point, Rc<Tile>)> {
        let mut within = Vec::new();

        for &(ref cur_layer_id, ref tiles) in self.tiles.iter() {
            if layer_id != cur_layer_id { continue; }

            for &(pos, ref tile) in tiles {
                if !is_removal(pos, tile.width, tile.height, x, y, width, height) { continue; }

                within.push((pos, Rc::clone(tile)));
            }
        }

        within
    }

    pub fn shift_tiles(&mut self, delta_x: i32, delta_y: i32) {
        for &mut (_, ref mut layer) in self.tiles.iter_mut() {
            for &mut (ref mut point, ref tile) in layer.iter_mut() {
                if point.x + delta_x < 0
                    || point.y + delta_y < 0
                    || point.x + delta_x + tile.width > MAX_AREA_SIZE
                    || point.y + delta_y + tile.height > MAX_AREA_SIZE {
                        warn!("Invalid tile shift parameters: {},{}", delta_x, delta_y);
                        return;
                }
            }
        }

        for &mut (_, ref mut layer) in self.tiles.iter_mut() {
            for &mut (ref mut point, _) in layer.iter_mut() {
                point.x += delta_x;
                point.y += delta_y;
            }
        }
    }

    pub fn add_actor(&mut self, actor: Rc<Actor>, x: i32, y: i32) {
        if x < 0 || y < 0 { return; }

        self.actors.push((Point::new(x, y), actor));
    }

    pub fn remove_actors_within(&mut self, x: i32, y: i32, size: i32) {
        self.actors.retain(|&(pos, ref actor)| {
            let pos_s = actor.race.size.size;
            !is_removal(pos, pos_s, pos_s, x, y, size, size)
        });
    }

    pub fn actors_within(&self, x: i32, y: i32, size: i32) -> Vec<(Point, Rc<Actor>)> {
        let mut actors = Vec::new();
        for &(pos, ref actor) in self.actors.iter() {
            let s = actor.race.size.size;
            if !is_removal(pos, s, s, x, y, size, size) { continue; }

            actors.push((pos, Rc::clone(actor)));
        }

        actors
    }

    pub fn add_prop(&mut self, prop: Rc<Prop>, x: i32, y: i32) {
        if x < 0 || y < 0 { return; }

        let prop_data = PropData {
            prop,
            location: Point::new(x, y),
            items: Vec::new(),
        };
        self.props.push(prop_data);
    }

    pub fn remove_props_within(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.props.retain(|prop_data| {
            let w = prop_data.prop.width as i32;
            let h = prop_data.prop.height as i32;
            !is_removal(prop_data.location, w, h, x, y, width, height)
        });
    }

    pub fn props_within(&self, x: i32, y: i32, width: i32, height: i32) -> Vec<(Point, Rc<Prop>)>{
        let mut within = Vec::new();
        for prop_data in self.props.iter() {
            let prop = &prop_data.prop;
            let pos = prop_data.location;
            if !is_removal(pos, prop.width as i32, prop.height as i32, x, y, width, height) { continue; }

            within.push((pos, Rc::clone(prop)));
        }

        within
    }

    pub fn new_transition(&mut self) -> Option<usize> {
        let sprite = match ResourceSet::get_image(&CONFIG.editor.transition_image) {
            None => {
                warn!("No image with ID {} found.", CONFIG.editor.transition_image);
                return None;
            }, Some(image) => image,
        };

        self.transitions.push(Transition {
            from: Point::new(1, 1),
            to: Point::new(1, 1),
            to_area: None,
            size: CONFIG.editor.transition_size,
            image_display: sprite,
        });

        Some(self.transitions.len() - 1)
    }

    pub fn delete_transition(&mut self, index: usize) {
        self.transitions.remove(index);
    }

    pub fn transitions_iter(&self) -> Iter<Transition> {
        self.transitions.iter()
    }

    pub fn transition(&self, index: usize) -> &Transition {
        &self.transitions[index]
    }

    pub fn transition_mut(&mut self, index: usize) -> &mut Transition {
        &mut self.transitions[index]
    }

    pub fn draw(&self, renderer: &mut GraphicsRenderer, x: f32, y: f32, scale_x: f32, scale_y: f32,
                millis: u32) {
        for &(_, ref tiles) in self.tiles.iter() {
            let mut draw_list = DrawList::empty_sprite();
            for &(pos, ref tile) in tiles {
                let sprite = &tile.image_display;
                draw_list.append(&mut DrawList::from_sprite_f32(sprite, x + pos.x as f32, y + pos.y as f32,
                                                                tile.width as f32, tile.height as f32));
            }
            if !draw_list.is_empty() {
                draw_list.set_scale(scale_x, scale_y);
                renderer.draw(draw_list);
            }
        }

        for prop_data in self.props.iter() {
            let x = prop_data.location.x as f32 + x;
            let y = prop_data.location.y as f32 + y;
            let mut draw_list = DrawList::empty_sprite();
            prop_data.prop.append_to_draw_list(&mut draw_list, &animation_state::NORMAL,
                                               x, y, millis);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        for &(pos, ref actor) in self.actors.iter() {
            actor.draw(renderer, scale_x, scale_y, pos.x as f32 + x, pos.y as f32 + y, millis);
        }

        for ref transition in self.transitions.iter() {
            let image = &transition.image_display;
            let x = transition.from.x as f32 + x;
            let y = transition.from.y as f32 + y;
            let w = transition.size.width;
            let h = transition.size.height;
            let mut draw_list = DrawList::empty_sprite();
            image.append_to_draw_list(&mut draw_list, &animation_state::NORMAL,
                                     x, y, w as f32, h as f32, millis);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }
    }

    pub fn load(&mut self, filename_prefix: &str, filename: &str) {
        let path = format!("{}/{}", filename_prefix, filename);
        debug!("Loading area state from {}", filename);

        let area_builder: AreaBuilder = match read_single_resource(&path) {
            Err(e) => {
                warn!("Unable to load area from {}", path);
                warn!("{}", e);
                return;
            }, Ok(builder) => builder,
        };

        self.id = area_builder.id;
        self.name = area_builder.name;
        self.filename = filename.to_string();

        trace!("Loading area terrain.");
        for &mut (_, ref mut vec) in self.tiles.iter_mut() {
            vec.clear();
        }
        for (tile_id, positions) in area_builder.terrain {
            let tile = match Module::tile(&tile_id) {
                None => {
                    warn!("No tile with ID {} found", tile_id);
                    continue;
                }, Some(tile) => tile,
            };

            for position in positions {
                if position.len() != 2 {
                    warn!("tile position vector is not length 2.");
                    continue;
                }

                let p = Point::new(position[0] as i32, position[1] as i32);

                let index = self.create_layer_if_missing(&tile.layer);
                self.tiles[index].1.push((p, Rc::clone(&tile)));
            }
        }

        trace!("Loading area actors.");
        self.actors.clear();
        for actor_data in area_builder.actors {
            let actor = match Module::actor(&actor_data.id) {
                None => {
                    warn!("No actor with ID {} found", actor_data.id);
                    continue;
                }, Some(actor) => actor,
            };

            self.actors.push((actor_data.location, actor));
        }

        trace!("Loading area props.");
        self.props.clear();
        for prop_builder in area_builder.props {
            let prop = match Module::prop(&prop_builder.id) {
                None => {
                    warn!("No prop with ID {} found", prop_builder.id);
                    continue;
                }, Some(prop) => prop,
            };

            let mut items = Vec::new();
            if let Some(ref builder_items) = prop_builder.items.as_ref() {
                for item_id in builder_items.iter() {
                    let item = match Module::item(item_id) {
                        None => {
                            warn!("No item with ID '{}' found", item_id);
                            continue;
                        }, Some(item) => item,
                    };
                    items.push(item);
                }
            }

            let prop_data = PropData {
                prop,
                location: prop_builder.location,
                items,
            };

            self.props.push(prop_data);
        }

        trace!("Loading area transitions.");
        self.transitions.clear();
        for transition_builder in area_builder.transitions {
            let image = match ResourceSet::get_image(&transition_builder.image_display) {
                None => {
                    warn!("No image with ID {} found.", transition_builder.image_display);
                    continue;
                }, Some(image) => image,
            };

            self.transitions.push(Transition {
                from: transition_builder.from,
                to: transition_builder.to,
                size: transition_builder.size,
                to_area: transition_builder.to_area,
                image_display: image,
            });
        }

        trace!("Loading area elevation.");
        if let Some(ref elev) = area_builder.elevation {
            for y in 0..area_builder.height {
                for x in 0..area_builder.width {
                    let val = elev[x + y * area_builder.width];
                    self.elevation[x + y * MAX_AREA_SIZE as usize] = val;
                }
            }
        } else {
            self.elevation = vec![0;(MAX_AREA_SIZE * MAX_AREA_SIZE) as usize];
        }
    }

    pub fn save(&self, filename_prefix: &str) {
        let filename = format!("{}/{}.yml", filename_prefix, self.filename);
        debug!("Saving current area state to {}", filename);
        let visibility_tile = CONFIG.editor.area.visibility_tile.clone();
        let explored_tile = CONFIG.editor.area.explored_tile.clone();

        let mut width = 0;
        let mut height = 0;
        let mut layers: Vec<String> = Vec::new();
        let mut terrain: HashMap<String, Vec<Vec<usize>>> = HashMap::new();

        trace!("Saving terrain.");
        for &(ref layer_id, ref tiles) in self.tiles.iter() {
            layers.push(layer_id.to_string());
            for &(position, ref tile) in tiles.iter() {
                width = cmp::max(width, position.x + tile.width);
                height = cmp::max(height, position.y + tile.height);

                let tiles_vec = terrain.entry(tile.id.to_string()).or_insert(Vec::new());
                tiles_vec.push(vec![position.x as usize, position.y as usize]);
            }
        }
        let entity_layer = CONFIG.editor.area.entity_layer;

        trace!("Saving actors.");
        let mut actors: Vec<ActorData> = Vec::new();
        for &(pos, ref actor) in self.actors.iter() {
            actors.push(ActorData { id: actor.id.to_string(), location: pos });
        }

        trace!("Saving props.");
        let mut props: Vec<PropDataBuilder> = Vec::new();
        for prop_data in self.props.iter() {
            let mut items = Vec::new();
            for item in prop_data.items.iter() {
                items.push(item.id.to_string());
            }

            let builder = PropDataBuilder {
                id: prop_data.prop.id.to_string(),
                location: prop_data.location,
                items: Some(items),
            };
            props.push(builder);
        }

        trace!("Saving transitions");
        let mut transitions: Vec<TransitionBuilder> = Vec::new();
        for ref transition in self.transitions.iter() {
            transitions.push(TransitionBuilder {
                from: transition.from,
                size: transition.size,
                to: transition.to,
                to_area: transition.to_area.clone(),
                image_display: CONFIG.editor.transition_image.clone(),
            });
        }

        trace!("Saving elevation");
        let mut elevation = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let val = self.elevation[(x + y * MAX_AREA_SIZE) as usize];
                elevation.push(val);
            }
        }

        let area_builder = AreaBuilder {
            id: self.id.clone(),
            name: self.name.clone(),
            elevation: Some(elevation),
            terrain,
            layers,
            visibility_tile,
            explored_tile,
            width: width as usize,
            height: height as usize,
            generate: false,
            entity_layer,
            actors,
            props,
            transitions,
            max_vis_distance: 20,
            max_vis_up_one_distance: 6,
        };

        trace!("Writing to file {}", filename);
        match write_to_file(&filename, &area_builder) {
            Err(e) => {
                error!("Unable to save area state to file {}", filename);
                error!("{}", e);
            },
            Ok(()) => {},
        }
    }

    fn create_layer_if_missing(&mut self, layer_id: &str) -> usize {
        for (index, &(ref id, _)) in self.tiles.iter().enumerate() {
            if id == layer_id {
                return index;
            }
        }

        self.tiles.push((layer_id.to_string(), Vec::new()));
        self.tiles.len() - 1
    }
}

fn is_removal(pos: Point, pos_w: i32, pos_h: i32, x: i32, y: i32, w: i32, h: i32) -> bool {
    // if one rectangle is on left side of the other
    if pos.x >= x + w || x >= pos.x + pos_w {
        return false;
    }

    // if one rectangle is above the other
    if pos.y >= y + h || y >= pos.y + pos_h {
        return false
    }

    true
}
