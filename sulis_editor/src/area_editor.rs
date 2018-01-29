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

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::io::Error;
use std::fs::File;
use std::cmp;

use sulis_core::serde_yaml;

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::io::event::ClickKind;
use sulis_core::ui::{Color, Cursor, Widget, WidgetKind};
use sulis_core::util::{invalid_data_error, Point};
use sulis_module::Module;
use sulis_module::area::{AreaBuilder, Tile};

const NAME: &str = "area_editor";

pub struct AreaEditor {
    tile_picker: Rc<RefCell<Widget>>,
    tiles: Vec<(Point, Rc<Tile>)>,
    cur_tile: Option<(Point, Rc<Tile>)>,
}

impl AreaEditor {
    pub fn new(tile_picker: &Rc<RefCell<Widget>>) -> Rc<RefCell<AreaEditor>> {
        Rc::new(RefCell::new(AreaEditor {
            tiles: Vec::new(),
            tile_picker: Rc::clone(tile_picker),
            cur_tile: None,
        }))
    }

    pub fn save(&self, filename: &str) {
        debug!("Saving current area state to {}", filename);
        let id = "test1".to_string();
        let name = "Test1".to_string();
        let visibility_tile = "gui/area_invis".to_string();

        let mut width = 0;
        let mut height = 0;
        let mut layers: Vec<String> = Vec::new();
        let mut terrain: HashMap<String, Vec<Vec<usize>>> = HashMap::new();

        for &(position, ref tile) in self.tiles.iter() {
            width = cmp::max(width, position.x + tile.width);
            height = cmp::max(height, position.y + tile.height);

            if !layers.contains(&tile.layer) {
                layers.push(tile.layer.to_string());
            }

            let tiles_vec = terrain.entry(tile.id.to_string()).or_insert(Vec::new());
            tiles_vec.push(vec![position.x as usize, position.y as usize]);
        }
        let entity_layer = layers.len() - 1;

        let area_builder = AreaBuilder {
            id,
            name,
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

        match write_to_file(filename, &area_builder) {
            Err(e) => {
                error!("Unable to save area state to file {}", filename);
                error!("{}", e);
            },
            Ok(()) => {},
        }
    }

    fn get_cursor_pos(&self, widget: &Rc<RefCell<Widget>>) -> (i32, i32) {
        let x = Cursor::get_x() - widget.borrow().state.position.x;
        let y = Cursor::get_y() - widget.borrow().state.position.y;
        (x, y)
    }

    fn get_current_tile(&self) -> Option<Rc<Tile>> {
        match self.tile_picker.borrow().state.get_text_arg("current_tile") {
            None => None,
            Some(tile) => Module::tile(tile),
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

    fn draw_graphics_mode(&mut self, renderer: &mut GraphicsRenderer, _pixel_size: Point,
                          widget: &Widget, _millis: u32) {
        let p = widget.state.inner_position;
        let s = widget.state.scroll_pos;

        if self.tiles.is_empty() { return; }

        let mut draw_list = DrawList::empty_sprite();
        for &(pos, ref tile) in self.tiles.iter() {
            let sprite = &tile.image_display;
            let x = pos.x + p.x + s.x;
            let y = pos.y + p.y + s.y;
            draw_list.append(&mut DrawList::from_sprite(sprite, x, y, tile.width, tile.height));
        }

        renderer.draw(draw_list);

        if let Some((cur_tile_pos, ref cur_tile)) = self.cur_tile {
            let sprite = &cur_tile.image_display;
            let x = cur_tile_pos.x + p.x + s.x;
            let y = cur_tile_pos.y + p.x + s.y;
            let mut draw_list = DrawList::from_sprite(sprite, x, y, cur_tile.width, cur_tile.height);
            draw_list.set_color(Color::from_string("FFFFFF88"));

            renderer.draw(draw_list);
        }
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, _kind: ClickKind) -> bool {
        let (x, y) = self.get_cursor_pos(widget);

        trace!("Getting tile {:?}", self.tile_picker.borrow().state.get_text_arg("current_tile"));

        let cur_tile = match self.get_current_tile() {
            None => return true,
            Some(tile) => tile,
        };

        self.tiles.push((Point::new(x, y), cur_tile));
        true
    }

    fn on_mouse_move(&mut self, widget: &Rc<RefCell<Widget>>) -> bool {
        let (x, y) = self.get_cursor_pos(widget);

        let cur_tile = match self.get_current_tile() {
            None => return true,
            Some(tile) => tile,
        };

        self.cur_tile = Some((Point::new(x, y), cur_tile));

        true
    }
}
