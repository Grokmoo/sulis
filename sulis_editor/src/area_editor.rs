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

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::slice::Iter;
use std::cmp;

use sulis_core::config::CONFIG;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::io::event::ClickKind;
use sulis_module::{Actor, Module};
use sulis_core::resource::{ResourceSet, read_single_resource, write_to_file};
use sulis_core::ui::{Color, Cursor, Widget, WidgetKind};
use sulis_core::util::{Point};
use sulis_module::area::{ActorData, AreaBuilder, Tile, Transition, TransitionBuilder};

use {ActorPicker, TilePicker};

pub enum Mode {
    Tiles,
    Actors,
}

const NAME: &str = "area_editor";

fn is_current_layer(tile_picker: &Rc<RefCell<TilePicker>>, layer_id: &str) -> bool {
    let tile_picker = tile_picker.borrow();

    match tile_picker.get_cur_layer() {
        None => false,
        Some(id) => id == layer_id,
    }
}

pub struct AreaEditor {
    tile_picker: Rc<RefCell<TilePicker>>,
    actor_picker: Rc<RefCell<ActorPicker>>,
    mode: Mode,

    cur_actor: Option<(Point, Rc<Actor>)>,
    removal_actors: Vec<(Point, Rc<Actor>)>,
    cur_tile: Option<(Point, Rc<Tile>)>,
    removal_tiles: Vec<(Point, Rc<Tile>)>,
    scroll_x_f32: f32,
    scroll_y_f32: f32,

    tiles: Vec<(String, Vec<(Point, Rc<Tile>)>)>,
    actors: Vec<(Point, Rc<Actor>)>,
    transitions: Vec<Transition>,

    pub id: String,
    pub name: String,
    pub filename: String,
    pub visibility_tile: String,
}

impl AreaEditor {
    pub fn new(actor_picker: &Rc<RefCell<ActorPicker>>,
               tile_picker: &Rc<RefCell<TilePicker>>) -> Rc<RefCell<AreaEditor>> {
        let mut tiles: Vec<(String, Vec<(Point, Rc<Tile>)>)> = Vec::new();
        for ref layer_id in CONFIG.editor.area.layers.iter() {
            tiles.push((layer_id.to_string(), Vec::new()));
        }

        Rc::new(RefCell::new(AreaEditor {
            mode: Mode::Tiles,
            tiles,
            actors: Vec::new(),
            transitions: Vec::new(),
            tile_picker: Rc::clone(tile_picker),
            actor_picker: Rc::clone(actor_picker),
            cur_actor: None,
            cur_tile: None,
            removal_tiles: Vec::new(),
            removal_actors: Vec::new(),
            scroll_x_f32: 0.0,
            scroll_y_f32: 0.0,
            id: CONFIG.editor.area.id.clone(),
            name: CONFIG.editor.area.name.clone(),
            filename: CONFIG.editor.area.filename.clone(),
            visibility_tile: CONFIG.editor.area.visibility_tile.clone(),
        }))
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.cur_tile = None;
        self.cur_actor = None;
        self.removal_tiles.clear();
        self.removal_actors.clear();
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

    fn create_layer_if_missing(&mut self, layer_id: &str) -> usize {
        for (index, &(ref id, _)) in self.tiles.iter().enumerate() {
            if id == layer_id {
                return index;
            }
        }

        self.tiles.push((layer_id.to_string(), Vec::new()));
        self.tiles.len() - 1
    }

    pub fn load(&mut self, filename_prefix: &str, filename: &str) {
        self.removal_tiles.clear();
        self.cur_tile = None;

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
        self.visibility_tile = area_builder.visibility_tile;

        trace!("Loading area terrain.");
        self.tiles.clear();
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
    }

    pub fn save(&self, filename_prefix: &str) {
        let filename = format!("{}/{}.yml", filename_prefix, self.filename);
        debug!("Saving current area state to {}", filename);
        let visibility_tile = CONFIG.editor.area.visibility_tile.clone();

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
        let entity_layer = 0;

        trace!("Saving actors.");
        let mut actors: Vec<ActorData> = Vec::new();
        for &(pos, ref actor) in self.actors.iter() {
            actors.push(ActorData { id: actor.id.to_string(), location: pos });
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

        let area_builder = AreaBuilder {
            id: self.id.clone(),
            name: self.name.clone(),
            terrain,
            layers,
            visibility_tile,
            width: width as usize,
            height: height as usize,
            generate: false,
            entity_layer,
            actors,
            transitions,
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

    fn get_cursor_pos(&self, widget: &Rc<RefCell<Widget>>, width: i32, height: i32) -> (i32, i32) {
        let x = widget.borrow().state.position.x - widget.borrow().state.scroll_pos.x;
        let y = widget.borrow().state.position.y - widget.borrow().state.scroll_pos.y;

        let x = Cursor::get_x_f32() - x as f32 - width as f32 / 2.0;
        let y = Cursor::get_y_f32() - y as f32 - height as f32 / 2.0;

        (x.round() as i32, y.round() as i32)
    }

    fn get_current_tile(&self) -> Option<Rc<Tile>> {
        self.tile_picker.borrow().get_cur_tile()
    }

    fn add_cur_tile(&mut self, widget: &Rc<RefCell<Widget>>) {
        let cur_tile = match self.get_current_tile() {
            None => return,
            Some(tile) => tile,
        };

        let (x, y) = self.get_cursor_pos(widget, cur_tile.width, cur_tile.height);
        if x < 0 || y < 0 { return; }

        let index = self.create_layer_if_missing(&cur_tile.layer);
        self.tiles[index].1.push((Point::new(x, y), cur_tile));
    }

    fn remove_cur_tiles(&mut self, widget: &Rc<RefCell<Widget>>) {
        let cur_tile = match self.get_current_tile() {
            None => return,
            Some(tile) => tile,
        };

        let (x, y) = self.get_cursor_pos(widget, cur_tile.width, cur_tile.height);
        if x < 0 || y < 0 { return; }

        self.removal_tiles.clear();
        for &mut (ref layer_id, ref mut tiles) in self.tiles.iter_mut() {
            if !is_current_layer(&self.tile_picker, layer_id) {
                continue;
            }

            tiles.retain(|&(pos, ref tile)| {
                !is_removal(pos, tile.width, tile.height, x, y, cur_tile.width, cur_tile.height)
            });
        }
    }

    fn mouse_move_tiles_mode(&mut self, widget: &Rc<RefCell<Widget>>) {
        let cur_tile = match self.get_current_tile() {
            None => return,
            Some(tile) => tile,
        };

        let (x, y) = self.get_cursor_pos(widget, cur_tile.width, cur_tile.height);
        if x < 0 || y < 0 { return; }

        let w = cur_tile.width;
        let h = cur_tile.height;
        self.cur_tile = Some((Point::new(x, y), cur_tile));

        self.removal_tiles.clear();
        for &(ref layer_id, ref tiles) in self.tiles.iter() {
            if !is_current_layer(&self.tile_picker, layer_id) {
                continue;
            }

            for &(pos, ref tile) in tiles {
                if is_removal(pos, tile.width, tile.height, x, y, w, h) {
                    self.removal_tiles.push((pos, Rc::clone(tile)));
                }
            }
        }
    }

    fn add_cur_actor(&mut self, widget: &Rc<RefCell<Widget>>) {
        let cur_actor = match self.actor_picker.borrow().get_cur_actor() {
            None => return,
            Some(actor) => actor,
        };

        let s = cur_actor.race.size.size;
        let (x, y) = self.get_cursor_pos(widget, s, s);
        if x < 0 || y < 0 { return; }

        self.actors.push((Point::new(x, y), cur_actor));
    }

    fn remove_cur_actors(&mut self, widget: &Rc<RefCell<Widget>>) {
        let cur_actor = match self.actor_picker.borrow().get_cur_actor() {
            None => return,
            Some(actor) => actor,
        };

        let s = cur_actor.race.size.size;
        let (x, y) = self.get_cursor_pos(widget, s, s);
        if x < 0 || y < 0 { return; }

        self.removal_actors.clear();
        self.actors.retain(|&(pos, ref actor)| {
            let pos_s = actor.race.size.size;
            !is_removal(pos, pos_s, pos_s, x, y, s, s)
        });
    }

    fn mouse_move_actors_mode(&mut self, widget: &Rc<RefCell<Widget>>) {
        let cur_actor = match self.actor_picker.borrow().get_cur_actor() {
            None => return,
            Some(actor) => actor,
        };

        let w = cur_actor.race.size.size;
        let h = cur_actor.race.size.size;
        let (x, y) = self.get_cursor_pos(widget, w, h);
        self.cur_actor = Some((Point::new(x, y), cur_actor));

        self.removal_actors.clear();
        for &(pos, ref actor) in self.actors.iter() {
            let s = actor.race.size.size;
            if is_removal(pos, s, s, x, y, w, h) {
                self.removal_actors.push((pos, Rc::clone(actor)));
            }
        }
    }
}

impl WidgetKind for AreaEditor {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_max_scroll_pos(256, 256);
        Vec::new()
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
                          widget: &Widget, millis: u32) {
        let p = widget.state.inner_position;
        let s = widget.state.scroll_pos;

        let mut draw_list = DrawList::empty_sprite();
        for &(_, ref tiles) in self.tiles.iter() {
            for &(pos, ref tile) in tiles {
                let sprite = &tile.image_display;
                let x = pos.x + p.x - s.x;
                let y = pos.y + p.y - s.y;
                draw_list.append(&mut DrawList::from_sprite(sprite, x, y, tile.width, tile.height));
            }
        }
        if !draw_list.is_empty() {
            renderer.draw(draw_list);
        }

        let mut draw_list = DrawList::empty_sprite();
        for &(pos, ref actor) in self.actors.iter() {
            actor.append_to_draw_list_i32(&mut draw_list, pos.x + p.x - s.x, pos.y + p.y - s.y, millis);
        }

        if !draw_list.is_empty() {
            renderer.draw(draw_list);
        }

        for ref transition in self.transitions.iter() {
            let image = &transition.image_display;
            let x = transition.from.x + p.x - s.x;
            let y = transition.from.y + p.y - s.y;
            let w = transition.size.width;
            let h = transition.size.height;
            image.draw_graphics_mode(renderer, &widget.state.animation_state,
                                     x as f32, y as f32, w as f32, h as f32, millis);
        }

        if !self.removal_tiles.is_empty() {
            let mut draw_list = DrawList::empty_sprite();
            for &(pos, ref tile) in self.removal_tiles.iter() {
                let sprite = &tile.image_display;
                let x = pos.x + p.x - s.x;
                let y = pos.y + p.y - s.y;
                draw_list.append(&mut DrawList::from_sprite(sprite, x, y, tile.width, tile.height));
            }

            draw_list.set_color(Color::from_string("FF000088"));
            renderer.draw(draw_list);
        }

        if !self.removal_actors.is_empty() {
            let mut draw_list = DrawList::empty_sprite();
            for &(pos, ref actor) in self.removal_actors.iter() {
                actor.append_to_draw_list_i32(&mut draw_list, pos.x + p.x - s.x, pos.y + p.y - s.y, millis);
            }

            draw_list.set_color(Color::from_string("FF000088"));
            renderer.draw(draw_list);
        }

        if let Some((cur_tile_pos, ref cur_tile)) = self.cur_tile {
            let sprite = &cur_tile.image_display;
            let x = cur_tile_pos.x + p.x - s.x;
            let y = cur_tile_pos.y + p.y - s.y;
            let mut draw_list = DrawList::from_sprite(sprite, x, y, cur_tile.width, cur_tile.height);
            draw_list.set_color(Color::from_string("FFFFFF88"));

            renderer.draw(draw_list);
        }

        if let Some((cur_actor_pos, ref cur_actor)) = self.cur_actor {
            let mut draw_list = DrawList::empty_sprite();
            cur_actor.append_to_draw_list_i32(&mut draw_list, cur_actor_pos.x + p.x - s.x,
                                          cur_actor_pos.y + p.y - s.y, millis);
            draw_list.set_color(Color::from_string("FFFFFF88"));
            renderer.draw(draw_list);
        }
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        match self.mode {
            Mode::Tiles => {
                match kind {
                    ClickKind::Left => self.add_cur_tile(widget),
                    ClickKind::Right => self.remove_cur_tiles(widget),
                    _ => (),
                }
            }, Mode::Actors => {
                match kind {
                    ClickKind::Left => self.add_cur_actor(widget),
                    ClickKind::Right => self.remove_cur_actors(widget),
                    _ => (),
                }
            },
        }

        true
    }

    fn on_mouse_drag(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind,
                     delta_x: f32, delta_y: f32) -> bool {
        match self.mode {
            Mode::Tiles => (),
            _ => return true,
        }

        match kind {
            ClickKind::Left => {
                if self.removal_tiles.is_empty() {
                    self.add_cur_tile(widget);
                }
            }, ClickKind::Right => {
                self.remove_cur_tiles(widget);
            }, ClickKind::Middle => {
                self.scroll_x_f32 -= delta_x;
                self.scroll_y_f32 -= delta_y;
                if self.scroll_x_f32 < 0.0 { self.scroll_x_f32 = 0.0; }
                if self.scroll_y_f32 < 0.0 { self.scroll_y_f32 = 0.0; }
                widget.borrow_mut().state.set_scroll(self.scroll_x_f32 as i32,
                                                     self.scroll_y_f32 as i32);
            }
        }

        true
    }

    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>,
                     _delta_x: f32, _delta_y: f32) -> bool {
        match self.mode {
            Mode::Tiles => self.mouse_move_tiles_mode(widget),
            Mode::Actors => self.mouse_move_actors_mode(widget),
        }

        true
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
