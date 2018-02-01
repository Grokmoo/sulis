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
use std::io::Error;
use std::fs::File;
use std::cmp;

use sulis_core::serde_yaml;

use sulis_core::config::CONFIG;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::io::event::ClickKind;
use sulis_module::Module;
use sulis_core::resource::read_single_resource;
use sulis_core::ui::{Color, Cursor, Widget, WidgetKind};
use sulis_core::util::{invalid_data_error, Point};
use sulis_module::area::{AreaBuilder, Tile};

use TilePicker;

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
    cur_tile: Option<(Point, Rc<Tile>)>,
    removal_tiles: Vec<(Point, Rc<Tile>)>,
    scroll_x_f32: f32,
    scroll_y_f32: f32,

    tiles: Vec<(String, Vec<(Point, Rc<Tile>)>)>,
    pub id: String,
    pub name: String,
    pub filename: String,
    pub visibility_tile: String,
}

impl AreaEditor {
    pub fn new(tile_picker: &Rc<RefCell<TilePicker>>) -> Rc<RefCell<AreaEditor>> {
        let mut tiles: Vec<(String, Vec<(Point, Rc<Tile>)>)> = Vec::new();
        for ref layer_id in CONFIG.editor.area.layers.iter() {
            tiles.push((layer_id.to_string(), Vec::new()));
        }

        Rc::new(RefCell::new(AreaEditor {
            tiles,
            tile_picker: Rc::clone(tile_picker),
            cur_tile: None,
            removal_tiles: Vec::new(),
            scroll_x_f32: 0.0,
            scroll_y_f32: 0.0,
            id: CONFIG.editor.area.id.clone(),
            name: CONFIG.editor.area.name.clone(),
            filename: CONFIG.editor.area.filename.clone(),
            visibility_tile: CONFIG.editor.area.visibility_tile.clone(),
        }))
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
    }

    pub fn save(&self, filename_prefix: &str) {
        let filename = format!("{}/{}.yml", filename_prefix, self.filename);
        debug!("Saving current area state to {}", filename);
        let visibility_tile = CONFIG.editor.area.visibility_tile.clone();

        let mut width = 0;
        let mut height = 0;
        let mut layers: Vec<String> = Vec::new();
        let mut terrain: HashMap<String, Vec<Vec<usize>>> = HashMap::new();

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
            actors: Vec::new(),
            transitions: Vec::new(),
        };

        match write_to_file(&filename, &area_builder) {
            Err(e) => {
                error!("Unable to save area state to file {}", filename);
                error!("{}", e);
            },
            Ok(()) => {},
        }
    }

    fn get_cursor_pos(&self, widget: &Rc<RefCell<Widget>>, tile: &Rc<Tile>) -> (i32, i32) {
        let x = widget.borrow().state.position.x - widget.borrow().state.scroll_pos.x;
        let y = widget.borrow().state.position.y - widget.borrow().state.scroll_pos.y;

        let x = Cursor::get_x_f32() - x as f32 - tile.width as f32 / 2.0;
        let y = Cursor::get_y_f32() - y as f32 - tile.height as f32 / 2.0;

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

        let (x, y) = self.get_cursor_pos(widget, &cur_tile);
        if x < 0 || y < 0 { return; }

        let index = self.create_layer_if_missing(&cur_tile.layer);
        self.tiles[index].1.push((Point::new(x, y), cur_tile));
    }

    fn remove_cur_tiles(&mut self, widget: &Rc<RefCell<Widget>>) {
        let cur_tile = match self.get_current_tile() {
            None => return,
            Some(tile) => tile,
        };

        let (x, y) = self.get_cursor_pos(widget, &cur_tile);
        if x < 0 || y < 0 { return; }

        self.removal_tiles.clear();
        for &mut (ref layer_id, ref mut tiles) in self.tiles.iter_mut() {
            if !is_current_layer(&self.tile_picker, layer_id) {
                continue;
            }

            tiles.retain(|&(pos, ref tile)| {
                !is_removal(pos, tile, x, y, cur_tile.width, cur_tile.height)
            });
        }
    }
}

fn write_to_file(filename: &str, builder: &AreaBuilder) -> Result<(), Error> {
    let file = File::create(filename)?;

    match serde_yaml::to_writer(file, builder) {
        Err(e) => invalid_data_error(&format!("{}", e)),
        Ok(()) => Ok(()),
    }
}

impl WidgetKind for AreaEditor {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn on_add(&mut self, widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        widget.borrow_mut().state.set_max_scroll_pos(256, 256);

        Vec::new()
    }

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
                          widget: &Widget, _millis: u32) {
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

        if let Some((cur_tile_pos, ref cur_tile)) = self.cur_tile {
            let sprite = &cur_tile.image_display;
            let x = cur_tile_pos.x + p.x - s.x;
            let y = cur_tile_pos.y + p.y - s.y;
            let mut draw_list = DrawList::from_sprite(sprite, x, y, cur_tile.width, cur_tile.height);
            draw_list.set_color(Color::from_string("FFFFFF88"));

            renderer.draw(draw_list);
        }
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind) -> bool {
        match kind {
            ClickKind::Left => self.add_cur_tile(widget),
            ClickKind::Right => self.remove_cur_tiles(widget),
            _ => (),
        };

        true
    }

    fn on_mouse_drag(&mut self, widget: &Rc<RefCell<Widget>>, kind: ClickKind,
                     delta_x: f32, delta_y: f32) -> bool {
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
        let cur_tile = match self.get_current_tile() {
            None => return true,
            Some(tile) => tile,
        };

        let (x, y) = self.get_cursor_pos(widget, &cur_tile);
        if x < 0 || y < 0 { return true; }

        let w = cur_tile.width;
        let h = cur_tile.height;
        self.cur_tile = Some((Point::new(x, y), cur_tile));

        self.removal_tiles.clear();
        for &(ref layer_id, ref tiles) in self.tiles.iter() {
            if !is_current_layer(&self.tile_picker, layer_id) {
                continue;
            }

            for &(pos, ref tile) in tiles {
                if is_removal(pos, tile, x, y, w, h) {
                    self.removal_tiles.push((pos, Rc::clone(tile)));
                }
            }
        }

        true
    }
}

fn is_removal(pos: Point, tile: &Rc<Tile>, x: i32, y: i32, w: i32, h: i32) -> bool {
    // if one rectangle is on left side of the other
    if pos.x >= x + w || x >= pos.x + tile.width {
        return false;
    }

    // if one rectangle is above the other
    if pos.y >= y + h || y >= pos.y + tile.height {
        return false
    }

    true
}
