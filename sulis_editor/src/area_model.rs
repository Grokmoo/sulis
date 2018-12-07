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

use sulis_core::config::{Config, EditorConfig};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::resource::{Sprite, ResourceSet, read_single_resource, write_to_file};
use sulis_core::ui::{animation_state, LineRenderer};
use sulis_core::util::{Point, Size};
use sulis_module::{Actor, Encounter, Module, Prop};
use sulis_module::area::*;

use crate::wall_picker::WallTiles;
use crate::terrain_picker::TerrainTiles;

pub struct AreaModel {
    pub config: EditorConfig,

    tiles: Vec<(String, Vec<(Point, Rc<Tile>)>)>,
    actors: Vec<(Point, Rc<Actor>)>,
    props: Vec<PropData>,
    encounters: Vec<EncounterData>,
    transitions: Vec<Transition>,
    triggers: Vec<TriggerBuilder>,
    elevation: Vec<u8>,

    encounter_sprite: Option<Rc<Sprite>>,
    font_renderer: Option<LineRenderer>,

    id: String,
    name: String,
    filename: String,
    max_vis_distance: i32,
    max_vis_up_one_distance: i32,
    world_map_location: Option<String>,
    on_rest: OnRest,

    terrain_kinds: Vec<TerrainTiles>,
    terrain: Vec<Option<usize>>,

    wall_kinds: Vec<WallTiles>,
    walls: Vec<(i8, Option<usize>)>,
}

impl AreaModel {
    pub fn new() -> AreaModel {
        let config = Config::editor_config();

        let encounter_sprite = match ResourceSet::get_sprite(&config.area.encounter_tile) {
            Ok(sprite) => Some(sprite),
            Err(_) => {
                warn!("Encounter tile '{}' not found", config.area.encounter_tile);
                None
            },
        };

        let font_renderer = match ResourceSet::get_font(&Config::default_font()) {
            None => {
                warn!("Font '{}' not found", Config::default_font());
                None
            }, Some(font) => Some(LineRenderer::new(&font)),
        };

        let mut tiles = Vec::new();
        for ref layer_id in config.area.layers.iter() {
            tiles.push((layer_id.to_string(), Vec::new()));
        }

        let elevation = vec![0;(MAX_AREA_SIZE * MAX_AREA_SIZE) as usize];

        let terrain_rules = Module::terrain_rules();
        let mut terrain_kinds = Vec::new();

        let all_kinds = Module::terrain_kinds();
        for kind in Module::terrain_kinds() {
            match TerrainTiles::new(&terrain_rules, kind, &all_kinds) {
                Err(_) => continue,
                Ok(terrain) => terrain_kinds.push(terrain),
            }
        }

        let wall_rules = Module::wall_rules();
        let mut wall_kinds = Vec::new();
        for kind in Module::wall_kinds() {
            match WallTiles::new(kind, &wall_rules) {
                Err(_) => continue,
                Ok(wall) => wall_kinds.push(wall),
            }
        }

        let id = config.area.id.clone();
        let name = config.area.name.clone();
        let filename = config.area.filename.clone();

        AreaModel {
            config,
            tiles,
            elevation,
            terrain_kinds,
            terrain: vec![None;(MAX_AREA_SIZE * MAX_AREA_SIZE) as usize],
            wall_kinds,
            walls: vec![(0, None);(MAX_AREA_SIZE * MAX_AREA_SIZE) as usize],
            actors: Vec::new(),
            props: Vec::new(),
            encounters: Vec::new(),
            transitions: Vec::new(),
            triggers: Vec::new(),
            encounter_sprite,
            font_renderer,
            id,
            name,
            filename,
            world_map_location: None,
            max_vis_distance: 20,
            max_vis_up_one_distance: 6,
            on_rest: OnRest::Disabled { message: "<PLACEHOLDER>".to_string() },
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

        let index = (x + y * MAX_AREA_SIZE) as usize;
        if index >= (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize { return 0; }

        self.elevation[index]
    }

    pub fn set_elevation(&mut self, elev: u8, x: i32, y: i32) {
        if x < 0 || y < 0 { return; }

        let index = (x + y * MAX_AREA_SIZE) as usize;
        if index >= (MAX_AREA_SIZE * MAX_AREA_SIZE) as usize { return; }

        self.elevation[index] = elev;
    }

    pub fn wall_kind(&self, index: usize) -> &WallTiles {
        &self.wall_kinds[index]
    }

    pub fn wall_kinds(&self) -> &Vec<WallTiles> {
        &self.wall_kinds
    }

    pub fn wall_at(&self, x: i32, y: i32) -> (i8, Option<usize>) {
        self.walls[(x + y * MAX_AREA_SIZE) as usize]
    }

    pub fn set_wall(&mut self, x: i32, y: i32, elev: i8, index: Option<usize>) {
        self.walls[(x + y * MAX_AREA_SIZE) as usize] = (elev, index);
    }

    pub fn terrain_index_at(&self, x: i32, y: i32) -> Option<usize> {
        self.terrain[(x + y * MAX_AREA_SIZE) as usize]
    }

    pub fn terrain_kind(&self, index: usize) -> &TerrainTiles {
        &self.terrain_kinds[index]
    }

    pub fn terrain_kinds(&self) -> &Vec<TerrainTiles> {
        &self.terrain_kinds
    }

    pub fn set_terrain_index(&mut self, x: i32, y: i32, index: Option<usize>) {
        self.terrain[(x + y * MAX_AREA_SIZE) as usize] = index;
    }

    pub fn all_tiles(&self) -> impl Iterator<Item = &(Point, Rc<Tile>)> {
        self.tiles.iter().map(|ref v| &v.1).flat_map(|ref t| t.iter())
    }

    pub fn add_tile(&mut self, tile: &Option<Rc<Tile>>, x: i32, y: i32) {
        let tile = match tile {
            &None => return,
            &Some(ref tile) => tile,
        };

        if x < 0 || y < 0 { return; }

        let index = self.create_layer_if_missing(&tile.layer);

        // check if the tile already exists
        for &(p, ref other_tile) in self.tiles[index].1.iter() {
            if p.x == x && p.y == y && Rc::ptr_eq(other_tile, tile) { return; }
        }
        self.tiles[index].1.push((Point::new(x, y), Rc::clone(tile)));
    }

    pub fn remove_all_tiles(&mut self, x: i32, y: i32, width: i32, height: i32) {
        for &mut (_, ref mut tiles) in self.tiles.iter_mut() {
            tiles.retain(|&(pos, ref tile)| {
                !is_removal(pos, tile.width, tile.height, x, y, width, height)
            });
        }
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

    pub fn add_trigger(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if x < 0 || y < 0 { return; }

        let location = Point::new(x, y);
        let size = Size::new(w, h);

        self.triggers.push(TriggerBuilder {
            kind: TriggerKind::OnPlayerEnter {
                location,
                size,
            },
            on_activate: Vec::new(),
            initially_enabled: true,
        });
    }

    pub fn add_encounter(&mut self, encounter: Rc<Encounter>, x: i32, y: i32, w: i32, h: i32) {
        if x < 0 || y < 0 { return; }

        self.encounters.push(EncounterData {
            encounter,
            location: Point::new(x, y),
            size: Size::new(w, h),
            triggers: Vec::new(),
        });
    }

    pub fn add_actor(&mut self, actor: Rc<Actor>, x: i32, y: i32) {
        if x < 0 || y < 0 { return; }

        self.actors.push((Point::new(x, y), actor));
    }

    pub fn remove_actors_within(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.actors.retain(|&(pos, ref actor)| {
            !is_removal(pos, actor.race.size.width, actor.race.size.height, x, y, width, height)
        });
    }

    pub fn actors_within(&self, x: i32, y: i32, width: i32, height: i32) -> Vec<(Point, Rc<Actor>)> {
        let mut actors = Vec::new();
        for &(pos, ref actor) in self.actors.iter() {
            if !is_removal(pos, actor.race.size.width, actor.race.size.height, x, y, width, height) {
                continue;
            }

            actors.push((pos, Rc::clone(actor)));
        }

        actors
    }

    pub fn add_prop(&mut self, prop: Rc<Prop>, x: i32, y: i32) {
        if x < 0 || y < 0 { return; }

        let prop_data = PropData {
            prop,
            enabled: true,
            location: Point::new(x, y),
            items: Vec::new(),
        };
        self.props.push(prop_data);
    }

    pub fn remove_props_within(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.props.retain(|prop_data| {
            let w = prop_data.prop.size.width;
            let h = prop_data.prop.size.height;
            !is_removal(prop_data.location, w, h, x, y, width, height)
        });
    }

    pub fn props_within(&self, x: i32, y: i32, width: i32, height: i32) -> Vec<(Point, Rc<Prop>)>{
        let mut within = Vec::new();
        for prop_data in self.props.iter() {
            let prop = &prop_data.prop;
            let pos = prop_data.location;
            if !is_removal(pos, prop.size.width, prop.size.height, x, y, width, height) { continue; }

            within.push((pos, Rc::clone(prop)));
        }

        within
    }

    pub fn remove_triggers_within(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.triggers.retain(|trig| {
            match trig.kind {
                TriggerKind::OnPlayerEnter { location, size } => {
                    !is_removal(location, size.width, size.height, x, y, width, height)
                },
                _ => true,
            }
        });
    }

    pub fn remove_encounters_within(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.encounters.retain(|enc_data| {
            let w = enc_data.size.width;
            let h = enc_data.size.height;
            !is_removal(enc_data.location, w, h, x, y, width, height)
        });
    }

    pub fn new_transition(&mut self) -> Option<usize> {
        let sprite = match ResourceSet::get_image(&self.config.transition_image) {
            None => {
                warn!("No image with ID {} found.", self.config.transition_image);
                return None;
            }, Some(image) => image,
        };

        let size = match Module::object_size(&self.config.transition_size) {
            None => {
                warn!("No size with ID '{}' found.", self.config.transition_size);
                return None;
            }, Some(ref size) => Rc::clone(size),
        };

        self.transitions.push(Transition {
            from: Point::new(1, 1),
            to: ToKind::WorldMap,
            hover_text: "<<PLACEHOLDER>>".to_string(),
            size,
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
            let w = actor.race.size.width as f32 / 2.0;
            let h = actor.race.size.height as f32 / 2.0;
            actor.draw(renderer, scale_x, scale_y, pos.x as f32 + x - w,
                       pos.y as f32 + y - h, millis);
        }

        let encounter_sprite = match self.encounter_sprite {
            None => return,
            Some(ref sprite) => sprite,
        };

        let font_renderer = match self.font_renderer {
            None => return,
            Some(ref font) => font,
        };

        for ref transition in self.transitions.iter() {
            let x = transition.from.x as f32 + x;
            let y = transition.from.y as f32 + y;
            let w = transition.size.width as f32;
            let h = transition.size.height as f32;
            let mut draw_list = DrawList::from_sprite_f32(encounter_sprite, x, y, w, h);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);

            let text = match transition.to {
                ToKind::CurArea { .. } => {
                    "to Current Area".to_string()
                },
                ToKind::Area { ref id, .. } => {
                    format!("to {}", id)
                },
                ToKind::WorldMap => {
                    "to World Map".to_string()
                }
            };

            let mut draw_list = font_renderer.get_draw_list(&text, x, y, 1.0);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        for ref encounter_data in self.encounters.iter() {
            let x = encounter_data.location.x as f32 + x;
            let y = encounter_data.location.y as f32 + y;
            let w = encounter_data.size.width as f32;
            let h = encounter_data.size.height as f32;
            let mut draw_list = DrawList::from_sprite_f32(encounter_sprite, x, y, w, h);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);

            let text = format!("{}", encounter_data.encounter.id);
            let mut draw_list = font_renderer.get_draw_list(&text, x, y, 1.0);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        for ref trigger_data in self.triggers.iter() {
            let (loc, size) = match trigger_data.kind {
                TriggerKind::OnPlayerEnter { location, size } => (location, size),
                _ => continue,
            };

            let x = loc.x as f32 + x;
            let y = loc.y as f32 + y;
            let w = size.width as f32;
            let h = size.height as f32;
            let mut draw_list = DrawList::from_sprite_f32(encounter_sprite, x, y, w, h);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);

            let mut draw_list = font_renderer.get_draw_list("Trigger", x, y, 1.0);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }
    }

    pub fn load(&mut self, filename_prefix: &str, filename: &str) {
        let path = format!("{}/{}", filename_prefix, filename);
        debug!("Loading area state from {}", filename);

        let mut area_builder: AreaBuilder = match read_single_resource(&path) {
            Err(e) => {
                warn!("Unable to load area from {}", path);
                warn!("{}", e);
                return;
            }, Ok(builder) => builder,
        };

        self.id = area_builder.id;
        self.name = area_builder.name;
        self.filename = filename.to_string();
        self.max_vis_distance = area_builder.max_vis_distance;
        self.max_vis_up_one_distance = area_builder.max_vis_up_one_distance;
        self.world_map_location = area_builder.world_map_location.clone();
        self.on_rest = area_builder.on_rest.clone();

        trace!("Loading terrain");
        let width = area_builder.width as i32;
        let (mut x, mut y) = (0, 0);
        for id in area_builder.terrain {
            if x + y * MAX_AREA_SIZE >= MAX_AREA_SIZE * MAX_AREA_SIZE { break; }

            match id {
                None => self.terrain[(x + y * MAX_AREA_SIZE) as usize] = None,
                Some(id) => {
                    for (index, kind) in self.terrain_kinds.iter().enumerate() {
                        if kind.id == id {
                            self.terrain[(x + y * MAX_AREA_SIZE) as usize] = Some(index);
                            break;
                        }
                    }
                }
            }

            x += 1;
            if x == width {
                x = 0;
                y += 1;
            }
        }

        trace!("Loading walls");
        let (mut x, mut y) = (0, 0);
        for (elev, id) in area_builder.walls {
            if x + y * MAX_AREA_SIZE >= MAX_AREA_SIZE * MAX_AREA_SIZE { break; }

            match id {
                None => self.walls[(x + y * MAX_AREA_SIZE) as usize] = (elev, None),
                Some(id) => {
                    for (index, kind) in self.wall_kinds.iter().enumerate() {
                        if kind.id == id {
                            self.walls[(x + y * MAX_AREA_SIZE) as usize] = (elev, Some(index));
                            break;
                        }
                    }
                }
            }

            x += 1;
            if x == width {
                x = 0;
                y += 1;
            }
        }

        trace!("Loading area layer_set.");
        for &mut (_, ref mut vec) in self.tiles.iter_mut() {
            vec.clear();
        }
        for (tile_id, positions) in area_builder.layer_set {
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

        trace!("Loading area encounters.");
        self.encounters.clear();
        for enc_builder in area_builder.encounters {
            let enc = match Module::encounter(&enc_builder.id) {
                None => {
                    warn!("No encounter '{}' found", enc_builder.id);
                    continue;
                }, Some(encounter) => encounter,
            };

            let enc_data = EncounterData {
                encounter: enc,
                location: enc_builder.location,
                size: enc_builder.size,
                triggers: Vec::new(),
            };
            self.encounters.push(enc_data);
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

            let prop_data = PropData {
                prop,
                enabled: prop_builder.enabled.unwrap_or(true),
                location: prop_builder.location,
                items: prop_builder.items,
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

            let size = match Module::object_size(&transition_builder.size) {
                None => {
                    warn!("No size with ID '{}' found.", transition_builder.size);
                    continue;
                }, Some(ref size) => Rc::clone(size),
            };

            self.transitions.push(Transition {
                from: transition_builder.from,
                to: transition_builder.to,
                size,
                hover_text: transition_builder.hover_text,
                image_display: image,
            });
        }

        trace!("Loading area triggers.");
        self.triggers.clear();
        self.triggers.append(&mut area_builder.triggers);

        trace!("Loading area elevation.");
        let elev = &area_builder.elevation;
        if elev.len() != area_builder.height * area_builder.width {
            warn!("Invalid elevation array in {}", path);
            self.elevation = vec![0;(MAX_AREA_SIZE * MAX_AREA_SIZE) as usize];
        } else {
            for y in 0..area_builder.height {
                for x in 0..area_builder.width {
                    let val = elev[x + y * area_builder.width];
                    self.elevation[x + y * MAX_AREA_SIZE as usize] = val;
                }
            }
        }
    }

    pub fn save(&self, filename_prefix: &str) {
        let filename = format!("{}/{}.yml", filename_prefix, self.filename);
        debug!("Saving current area state to {}", filename);
        let visibility_tile = self.config.area.visibility_tile.clone();
        let explored_tile = self.config.area.explored_tile.clone();

        let mut width = 0;
        let mut height = 0;
        let mut layers: Vec<String> = Vec::new();
        let mut layer_set: HashMap<String, Vec<Vec<usize>>> = HashMap::new();

        trace!("Saving layer_set.");
        for &(ref layer_id, ref tiles) in self.tiles.iter() {
            layers.push(layer_id.to_string());
            for &(position, ref tile) in tiles.iter() {
                width = cmp::max(width, position.x + tile.width);
                height = cmp::max(height, position.y + tile.height);

                if position.x >= MAX_AREA_SIZE || position.y >= MAX_AREA_SIZE { continue; }
                let tiles_vec = layer_set.entry(tile.id.to_string()).or_insert(Vec::new());
                tiles_vec.push(vec![position.x as usize, position.y as usize]);
            }
        }
        width = cmp::min(width, MAX_AREA_SIZE);
        height = cmp::min(height, MAX_AREA_SIZE);
        let entity_layer = self.config.area.entity_layer;

        trace!("Saving actors.");
        let mut actors: Vec<ActorData> = Vec::new();
        for &(pos, ref actor) in self.actors.iter() {
            let unique_id = None;
            actors.push(ActorData { id: actor.id.to_string(), unique_id, location: pos });
        }

        trace!("Saving props.");
        let mut props: Vec<PropDataBuilder> = Vec::new();
        for prop_data in self.props.iter() {
            let builder = PropDataBuilder {
                id: prop_data.prop.id.to_string(),
                enabled: Some(prop_data.enabled),
                location: prop_data.location,
                items: prop_data.items.clone(),
            };
            props.push(builder);
        }

        trace!("Saving encounters.");
        let mut encounters: Vec<EncounterDataBuilder> = Vec::new();
        for enc_data in self.encounters.iter() {
            let builder = EncounterDataBuilder {
                id: enc_data.encounter.id.to_string(),
                location: enc_data.location,
                size: enc_data.size,
            };
            encounters.push(builder);
        }

        trace!("Saving transitions");
        let mut transitions: Vec<TransitionBuilder> = Vec::new();
        for ref transition in self.transitions.iter() {
            transitions.push(TransitionBuilder {
                from: transition.from,
                size: transition.size.id.to_string(),
                to: transition.to.clone(),
                hover_text: transition.hover_text.to_string(),
                image_display: self.config.transition_image.clone(),
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

        trace!("Saving terrain");
        let mut terrain = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let index = match self.terrain[(x + y * MAX_AREA_SIZE) as usize] {
                    None => {
                        terrain.push(None);
                        continue;
                    }, Some(index) => index,
                };

                let tiles = &self.terrain_kinds[index];
                terrain.push(Some(tiles.id.to_string()));
            }
        }

        trace!("Saving walls");
        let mut walls = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let (elev, index) = match self.walls[(x + y * MAX_AREA_SIZE) as usize] {
                    (elev, None) => {
                        walls.push((elev, None));
                        continue;
                    }, (elev, Some(index)) => (elev, index),
                };

                let cur_wall = &self.wall_kinds[index];
                walls.push((elev, Some(cur_wall.id.to_string())));
            }
        }

        let area_builder = AreaBuilder {
            id: self.id.clone(),
            name: self.name.clone(),
            elevation: elevation,
            terrain,
            walls,
            layer_set,
            layers,
            visibility_tile,
            explored_tile,
            width: width as usize,
            height: height as usize,
            generate: false,
            entity_layer,
            actors,
            props,
            encounters,
            transitions,
            triggers: self.triggers.clone(),
            max_vis_distance: self.max_vis_distance,
            max_vis_up_one_distance: self.max_vis_up_one_distance,
            world_map_location: self.world_map_location.clone(),
            on_rest: self.on_rest.clone(),
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
