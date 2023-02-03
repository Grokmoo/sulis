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

use std::cmp;
use std::collections::HashMap;
use std::rc::Rc;
use std::slice::Iter;

use sulis_core::config::{Config, EditorConfig};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::resource::{read_single_resource, write_to_file, ResourceSet, Sprite};
use sulis_core::ui::{animation_state, LineRenderer};
use sulis_core::util::{Offset, Point, Rect, Scale, Size};
use sulis_module::area::*;
use sulis_module::generator::{is_removal, TilesModel};
use sulis_module::{Actor, Encounter, Module, Prop};

pub struct AreaModel {
    pub config: EditorConfig,

    tiles: TilesModel,
    actors: Vec<(Point, Rc<Actor>, Option<String>)>,
    props: Vec<PropData>,
    encounters: Vec<EncounterData>,
    transitions: Vec<Transition>,
    triggers: Vec<TriggerBuilder>,

    encounter_sprite: Option<Rc<Sprite>>,
    font_renderer: Option<LineRenderer>,

    id: String,
    name: String,
    filename: String,
    pub max_vis_distance: i32,
    pub max_vis_up_one_distance: i32,
    pub world_map_location: Option<String>,
    pub location_kind: LocationKind,
    pub on_rest: OnRest,

    ambient_sound: Option<String>,
    default_music: Option<String>,
    default_combat_music: Option<String>,
}

impl Default for AreaModel {
    fn default() -> AreaModel {
        let config = Config::editor_config();

        let encounter_sprite = match ResourceSet::sprite(&config.area.encounter_tile) {
            Ok(sprite) => Some(sprite),
            Err(_) => {
                warn!("Encounter tile '{}' not found", config.area.encounter_tile);
                None
            }
        };

        let font_renderer = match ResourceSet::font(&Config::default_font()) {
            None => {
                warn!("Font '{}' not found", Config::default_font());
                None
            }
            Some(font) => Some(LineRenderer::new(&font)),
        };

        let id = config.area.id.clone();
        let name = config.area.name.clone();
        let filename = config.area.filename.clone();

        let tiles = TilesModel::new();

        AreaModel {
            config,
            tiles,
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
            ambient_sound: None,
            default_music: None,
            default_combat_music: None,
            location_kind: LocationKind::Outdoors,
            on_rest: OnRest::Disabled {
                message: "<PLACEHOLDER>".to_string(),
            },
        }
    }
}

impl AreaModel {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn filename(&self) -> &str {
        &self.filename
    }
    pub fn location_kind(&self) -> LocationKind {
        self.location_kind
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_string();
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    pub fn set_filename(&mut self, filename: &str) {
        self.filename = filename.to_string();
    }

    pub fn set_location_kind(&mut self, location_kind: LocationKind) {
        self.location_kind = location_kind;
    }

    pub fn add_trigger(&mut self, x: i32, y: i32, w: i32, h: i32) {
        if x < 0 || y < 0 {
            return;
        }

        let location = Point::new(x, y);
        let size = Size::new(w, h);

        self.triggers.push(TriggerBuilder {
            kind: TriggerKind::OnPlayerEnter { location, size },
            on_activate: Vec::new(),
            initially_enabled: true,
            fire_more_than_once: false,
        });
    }

    pub fn add_encounter(&mut self, encounter: Rc<Encounter>, x: i32, y: i32, w: i32, h: i32) {
        if x < 0 || y < 0 {
            return;
        }

        self.encounters.push(EncounterData {
            encounter,
            location: Point::new(x, y),
            size: Size::new(w, h),
            triggers: Vec::new(),
        });
    }

    pub fn add_actor(&mut self, actor: Rc<Actor>, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }

        self.actors.push((Point::new(x, y), actor, None));
    }

    pub fn remove_actors_within(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.actors.retain(|&(pos, ref actor, _)| {
            !is_removal(
                pos,
                actor.race.size.width,
                actor.race.size.height,
                x,
                y,
                width,
                height,
            )
        });
    }

    pub fn actors_within(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Vec<(Point, Rc<Actor>)> {
        let mut actors = Vec::new();
        for &(pos, ref actor, _) in self.actors.iter() {
            if !is_removal(
                pos,
                actor.race.size.width,
                actor.race.size.height,
                x,
                y,
                width,
                height,
            ) {
                continue;
            }

            actors.push((pos, Rc::clone(actor)));
        }

        actors
    }

    pub fn add_prop(&mut self, prop: Rc<Prop>, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }

        let prop_data = PropData {
            prop,
            enabled: true,
            location: Point::new(x, y),
            items: Vec::new(),
            hover_text: None,
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

    pub fn props_within(&self, x: i32, y: i32, width: i32, height: i32) -> Vec<(Point, Rc<Prop>)> {
        let mut within = Vec::new();
        for prop_data in self.props.iter() {
            let prop = &prop_data.prop;
            let pos = prop_data.location;
            if !is_removal(pos, prop.size.width, prop.size.height, x, y, width, height) {
                continue;
            }

            within.push((pos, Rc::clone(prop)));
        }

        within
    }

    pub fn remove_triggers_within(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.triggers.retain(|trig| match trig.kind {
            TriggerKind::OnPlayerEnter { location, size } => {
                !is_removal(location, size.width, size.height, x, y, width, height)
            }
            _ => true,
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
        let sprite = match ResourceSet::image(&self.config.transition_image) {
            None => {
                warn!("No image with ID {} found.", self.config.transition_image);
                return None;
            }
            Some(image) => image,
        };

        let size = match Module::object_size(&self.config.transition_sizes[0]) {
            None => {
                warn!(
                    "No size with ID '{}' found.",
                    self.config.transition_sizes[0]
                );
                return None;
            }
            Some(ref size) => Rc::clone(size),
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

    pub fn add_tile(&mut self, tile: &Option<Rc<Tile>>, x: i32, y: i32) {
        if let Some(tile) = tile {
            self.tiles.add(Rc::clone(tile), x, y);
        }
    }

    pub fn shift_tiles(&mut self, delta_x: i32, delta_y: i32) {
        self.tiles.shift(delta_x, delta_y);
    }

    pub fn remove_all_tiles(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.tiles.remove_all(x, y, width, height);
    }

    pub fn remove_tiles_within(&mut self, layer_id: &str, x: i32, y: i32, width: i32, height: i32) {
        self.tiles.remove_within(layer_id, x, y, width, height);
    }

    pub fn tiles(&self) -> &TilesModel {
        &self.tiles
    }

    pub fn set_elevation(&mut self, elev: u8, x: i32, y: i32) {
        self.tiles.set_elevation(elev, x, y);
    }

    pub fn set_wall(&mut self, x: i32, y: i32, elev: u8, index: Option<usize>) {
        self.tiles.set_wall(x, y, elev, index);
    }

    pub fn set_terrain_index(&mut self, x: i32, y: i32, index: Option<usize>) {
        self.tiles.set_terrain_index(x, y, index);
    }

    pub fn check_add_terrain_border(&mut self, x: i32, y: i32) {
        self.tiles.check_add_terrain_border(x, y);
    }

    pub fn check_add_wall_border(&mut self, x: i32, y: i32) {
        self.tiles.check_add_wall_border(x, y);
    }

    pub fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        offset: Offset,
        scale: Scale,
        millis: u32,
    ) {
        for (_, tiles) in self.tiles.iter() {
            let mut draw_list = DrawList::empty_sprite();
            for &(pos, ref tile) in tiles {
                let sprite = &tile.image_display;
                let rect = Rect {
                    x: offset.x + pos.x as f32,
                    y: offset.y + pos.y as f32,
                    w: tile.width as f32,
                    h: tile.height as f32,
                };
                draw_list.append(&mut DrawList::from_sprite_f32(sprite, rect));
            }
            if !draw_list.is_empty() {
                draw_list.set_scale(scale);
                renderer.draw(draw_list);
            }
        }

        for prop_data in self.props.iter() {
            let x = prop_data.location.x as f32 + offset.x;
            let y = prop_data.location.y as f32 + offset.y;
            let mut draw_list = DrawList::empty_sprite();
            prop_data.prop.append_to_draw_list(
                &mut draw_list,
                &animation_state::NORMAL,
                Offset { x, y },
                millis,
            );
            draw_list.set_scale(scale);
            renderer.draw(draw_list);
        }

        for &(pos, ref actor, _) in self.actors.iter() {
            let w = actor.race.size.width as f32 / 2.0;
            let h = actor.race.size.height as f32 / 2.0;
            actor.draw(
                renderer,
                Offset {
                    x: pos.x as f32 + offset.x - w,
                    y: pos.y as f32 + offset.y - h,
                },
                scale,
                millis,
            );
        }

        let encounter_sprite = match self.encounter_sprite {
            None => return,
            Some(ref sprite) => sprite,
        };

        let font_renderer = match self.font_renderer {
            None => return,
            Some(ref font) => font,
        };

        for transition in self.transitions.iter() {
            let offset = Offset {
                x: transition.from.x as f32 + offset.x,
                y: transition.from.y as f32 + offset.y,
            };
            let rect = Rect {
                x: offset.x,
                y: offset.y,
                w: transition.size.width as f32,
                h: transition.size.height as f32,
            };
            let mut draw_list = DrawList::from_sprite_f32(encounter_sprite, rect);
            draw_list.set_scale(scale);
            renderer.draw(draw_list);

            let text = match transition.to {
                ToKind::CurArea { .. } => "to Current Area".to_string(),
                ToKind::Area { ref id, .. } => format!("to {id}"),
                ToKind::WorldMap => "to World Map".to_string(),
                ToKind::FindLink { ref id, .. } => format!("to {id}"),
            };

            let (mut draw_list, _) = font_renderer.get_draw_list(&text, offset, 1.0);
            draw_list.set_scale(scale);
            renderer.draw(draw_list);
        }

        for encounter_data in self.encounters.iter() {
            let offset = Offset {
                x: encounter_data.location.x as f32 + offset.x,
                y: encounter_data.location.y as f32 + offset.y,
            };
            let rect = Rect {
                x: offset.x,
                y: offset.y,
                w: encounter_data.size.width as f32,
                h: encounter_data.size.height as f32,
            };
            let mut draw_list = DrawList::from_sprite_f32(encounter_sprite, rect);
            draw_list.set_scale(scale);
            renderer.draw(draw_list);

            let text = &encounter_data.encounter.id;
            let (mut draw_list, _) = font_renderer.get_draw_list(text, offset, 1.0);
            draw_list.set_scale(scale);
            renderer.draw(draw_list);
        }

        for trigger_data in self.triggers.iter() {
            let (loc, size) = match trigger_data.kind {
                TriggerKind::OnPlayerEnter { location, size } => (location, size),
                _ => continue,
            };

            let offset = Offset {
                x: loc.x as f32 + offset.x,
                y: loc.y as f32 + offset.y,
            };
            let rect = Rect {
                x: offset.x,
                y: offset.y,
                w: size.width as f32,
                h: size.height as f32,
            };
            let mut draw_list = DrawList::from_sprite_f32(encounter_sprite, rect);
            draw_list.set_scale(scale);
            renderer.draw(draw_list);

            let (mut draw_list, _) = font_renderer.get_draw_list("Trigger", offset, 1.0);
            draw_list.set_scale(scale);
            renderer.draw(draw_list);
        }
    }

    pub fn load(&mut self, filename_prefix: &str, filename: &str) {
        let path = format!("{filename_prefix}/{filename}");
        debug!("Loading area state from {}", filename);

        let mut area_builder: AreaBuilder = match read_single_resource(&path) {
            Err(e) => {
                warn!("Unable to load area from {}", path);
                warn!("{}", e);
                return;
            }
            Ok(builder) => builder,
        };

        self.id = area_builder.id;
        self.name = area_builder.name;
        self.filename = filename.to_string();
        self.max_vis_distance = area_builder.max_vis_distance;
        self.max_vis_up_one_distance = area_builder.max_vis_up_one_distance;
        self.world_map_location = area_builder.world_map_location.clone();
        self.on_rest = area_builder.on_rest.clone();
        self.location_kind = area_builder.location_kind;
        self.ambient_sound = area_builder.ambient_sound;
        self.default_music = area_builder.default_music;
        self.default_combat_music = area_builder.default_combat_music;

        let width = area_builder.width as i32;

        self.load_terrain(area_builder.terrain, width);

        self.load_walls(area_builder.walls, width);

        self.load_layer_set(area_builder.layer_set);

        self.load_actors(area_builder.actors);

        self.load_encounters(area_builder.encounters);

        self.load_props(area_builder.props);

        self.load_transitions(area_builder.transitions);

        trace!("Loading area triggers.");
        self.triggers.clear();
        self.triggers.append(&mut area_builder.triggers);

        trace!("Loading area elevation.");
        let elev = &area_builder.elevation;
        let dest_elev = self.tiles.raw_elevation();
        if elev.len() != area_builder.height * area_builder.width {
            warn!("Invalid elevation array in {}", path);
            for elev in dest_elev.iter_mut() {
                *elev = 0;
            }
        } else {
            for y in 0..area_builder.height {
                for x in 0..area_builder.width {
                    let val = elev[x + y * area_builder.width];
                    dest_elev[x + y * MAX_AREA_SIZE as usize] = val;
                }
            }
        }
    }

    pub fn load_props(&mut self, props: Vec<PropDataBuilder>) {
        trace!("Loading area props.");
        self.props.clear();
        for prop_builder in props {
            let prop = match Module::prop(&prop_builder.id) {
                None => {
                    warn!("No prop with ID {} found", prop_builder.id);
                    continue;
                }
                Some(prop) => prop,
            };

            let prop_data = PropData {
                prop,
                enabled: prop_builder.enabled.unwrap_or(true),
                location: prop_builder.location,
                items: prop_builder.items,
                hover_text: prop_builder.hover_text,
            };

            self.props.push(prop_data);
        }
    }

    pub fn load_transitions(&mut self, transitions: Vec<TransitionBuilder>) {
        trace!("Loading area transitions.");
        self.transitions.clear();
        for transition_builder in transitions {
            let image = match ResourceSet::image(&transition_builder.image_display) {
                None => {
                    warn!(
                        "No image with ID {} found.",
                        transition_builder.image_display
                    );
                    continue;
                }
                Some(image) => image,
            };

            let size = match Module::object_size(&transition_builder.size) {
                None => {
                    warn!("No size with ID '{}' found.", transition_builder.size);
                    continue;
                }
                Some(ref size) => Rc::clone(size),
            };

            self.transitions.push(Transition {
                from: transition_builder.from,
                to: transition_builder.to,
                size,
                hover_text: transition_builder.hover_text,
                image_display: image,
            });
        }
    }

    pub fn load_layer_set(&mut self, layer_set: HashMap<String, Vec<Vec<u16>>>) {
        trace!("Loading area layer_set.");
        self.tiles.clear();

        for (tile_id, positions) in layer_set {
            let tile = match Module::tile(&tile_id) {
                None => {
                    warn!("No tile with ID {} found", tile_id);
                    continue;
                }
                Some(tile) => tile,
            };

            for position in positions {
                if position.len() != 2 {
                    warn!("tile position vector is not length 2.");
                    continue;
                }

                self.tiles
                    .add(Rc::clone(&tile), position[0] as i32, position[1] as i32);
            }
        }
    }

    pub fn load_encounters(&mut self, encounters: Vec<EncounterDataBuilder>) {
        trace!("Loading area encounters.");
        self.encounters.clear();
        for enc_builder in encounters {
            let enc = match Module::encounter(&enc_builder.id) {
                None => {
                    warn!("No encounter '{}' found", enc_builder.id);
                    continue;
                }
                Some(encounter) => encounter,
            };

            let enc_data = EncounterData {
                encounter: enc,
                location: enc_builder.location,
                size: enc_builder.size,
                triggers: Vec::new(),
            };
            self.encounters.push(enc_data);
        }
    }

    pub fn load_actors(&mut self, actors: Vec<ActorData>) {
        trace!("Loading area actors.");
        self.actors.clear();
        for actor_data in actors {
            let actor = match Module::actor(&actor_data.id) {
                None => {
                    warn!("No actor with ID {} found", actor_data.id);
                    continue;
                }
                Some(actor) => actor,
            };

            self.actors
                .push((actor_data.location, actor, actor_data.unique_id));
        }
    }

    pub fn load_walls(&mut self, walls: Vec<(u8, Option<String>)>, width: i32) {
        trace!("Loading walls");
        let (mut x, mut y) = (0, 0);
        for (elev, id) in walls {
            if x + y * MAX_AREA_SIZE >= MAX_AREA_SIZE * MAX_AREA_SIZE {
                break;
            }

            match id {
                None => self.tiles.set_wall(x, y, elev, None),
                Some(id) => {
                    for (index, kind) in self.tiles().wall_kinds().iter().enumerate() {
                        if kind.id == id {
                            self.tiles.set_wall(x, y, elev, Some(index));
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
    }

    pub fn load_terrain(&mut self, terrain: Vec<Option<String>>, width: i32) {
        trace!("Loading terrain");
        let (mut x, mut y) = (0, 0);
        for id in terrain {
            if x + y * MAX_AREA_SIZE >= MAX_AREA_SIZE * MAX_AREA_SIZE {
                break;
            }

            match id {
                None => self.tiles.set_terrain_index(x, y, None),
                Some(id) => {
                    for (index, kind) in self.tiles.terrain_kinds().iter().enumerate() {
                        if kind.id == id {
                            self.tiles.set_terrain_index(x, y, Some(index));
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
    }

    pub fn save(&self, filename_prefix: &str) {
        let filename = format!("{}/{}.yml", filename_prefix, self.filename);
        debug!("Saving current area state to {}", filename);
        let visibility_tile = self.config.area.visibility_tile.clone();
        let explored_tile = self.config.area.explored_tile.clone();

        let mut width = 0;
        let mut height = 0;
        let mut layers: Vec<String> = Vec::new();
        let mut layer_set: HashMap<String, Vec<Vec<u16>>> = HashMap::new();

        trace!("Saving layer_set.");
        for (layer_id, tiles) in self.tiles.iter() {
            layers.push(layer_id.to_string());
            for &(position, ref tile) in tiles.iter() {
                width = cmp::max(width, position.x + tile.width);
                height = cmp::max(height, position.y + tile.height);

                if position.x >= MAX_AREA_SIZE || position.y >= MAX_AREA_SIZE {
                    continue;
                }
                let tiles_vec = layer_set
                    .entry(tile.id.to_string())
                    .or_insert_with(Vec::new);
                tiles_vec.push(vec![position.x as u16, position.y as u16]);
            }
        }
        width = cmp::min(width, MAX_AREA_SIZE);
        height = cmp::min(height, MAX_AREA_SIZE);
        let entity_layer = self.config.area.entity_layer;

        trace!("Saving actors.");
        let mut actors: Vec<ActorData> = Vec::new();
        for &(pos, ref actor, ref unique_id) in self.actors.iter() {
            actors.push(ActorData {
                id: actor.id.to_string(),
                unique_id: unique_id.clone(),
                location: pos,
            });
        }

        trace!("Saving props.");
        let mut props: Vec<PropDataBuilder> = Vec::new();
        for prop_data in self.props.iter() {
            let builder = PropDataBuilder {
                id: prop_data.prop.id.to_string(),
                enabled: Some(prop_data.enabled),
                location: prop_data.location,
                items: prop_data.items.clone(),
                hover_text: prop_data.hover_text.clone(),
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
        for transition in self.transitions.iter() {
            transitions.push(TransitionBuilder {
                from: transition.from,
                size: transition.size.id.to_string(),
                to: transition.to.clone(),
                hover_text: transition.hover_text.to_string(),
                image_display: self.config.transition_image.clone(),
            });
        }

        let (elevation, terrain) = self.save_terrain(width, height);

        trace!("Saving walls");
        let mut walls = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let (elev, index) = match self.tiles.wall_at(x, y) {
                    (elev, None) => {
                        walls.push((elev, None));
                        continue;
                    }
                    (elev, Some(index)) => (elev, index),
                };

                let cur_wall = self.tiles.wall_kind(index);
                walls.push((elev, Some(cur_wall.id.to_string())));
            }
        }

        let area_builder = AreaBuilder {
            id: self.id.clone(),
            name: self.name.clone(),
            location_kind: self.location_kind,
            elevation,
            terrain,
            walls,
            layer_set,
            layers,
            visibility_tile,
            explored_tile,
            width: width as usize,
            height: height as usize,
            generator: None,
            entity_layer,
            actors,
            props,
            encounters,
            transitions,
            triggers: self.triggers.clone(),
            max_vis_distance: self.max_vis_distance,
            max_vis_up_one_distance: self.max_vis_up_one_distance,
            world_map_location: self.world_map_location.clone(),
            ambient_sound: self.ambient_sound.clone(),
            default_music: self.default_music.clone(),
            default_combat_music: self.default_combat_music.clone(),
            on_rest: self.on_rest.clone(),
        };

        trace!("Writing to file {}", filename);
        if let Err(e) = write_to_file(&filename, &area_builder) {
            error!("Unable to save area state to file {}", filename);
            error!("{}", e);
        }
    }

    fn save_terrain(&self, width: i32, height: i32) -> (Vec<u8>, Vec<Option<String>>) {
        trace!("Saving elevation");
        let mut elevation = Vec::new();
        for y in 0..height {
            for x in 0..width {
                elevation.push(self.tiles.elevation(x, y));
            }
        }

        trace!("Saving terrain");
        let mut terrain = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let index = match self.tiles.terrain_index_at(x, y) {
                    None => {
                        terrain.push(None);
                        continue;
                    }
                    Some(index) => index,
                };

                let tiles = self.tiles.terrain_kind(index);
                terrain.push(Some(tiles.id.to_string()));
            }
        }

        (elevation, terrain)
    }
}
