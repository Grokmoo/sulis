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

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::io::event::ClickKind;
use sulis_core::ui::{Cursor, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_module::Module;
use sulis_module::area::Tile;

const NAME: &str = "area_editor";

pub struct AreaEditor {
    tile_picker: Rc<RefCell<Widget>>,
    tiles: Vec<(Point, Rc<Tile>)>,
}

impl AreaEditor {
    pub fn new(tile_picker: &Rc<RefCell<Widget>>) -> Rc<RefCell<AreaEditor>> {
        let mut tiles: Vec<(Point, Rc<Tile>)> = Vec::new();
        let tile = Module::tile("grass4").unwrap();
        tiles.push((Point::new(0, 0), Rc::clone(&tile)));

        Rc::new(RefCell::new(AreaEditor {
            tiles,
            tile_picker: Rc::clone(tile_picker),
        }))
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

        let mut draw_list = DrawList::empty_sprite();
        for &(pos, ref tile) in self.tiles.iter() {
            let sprite = &tile.image_display;
            let x = pos.x + p.x + s.x;
            let y = pos.y + p.y + s.y;
            draw_list.append(&mut DrawList::from_sprite(sprite, x, y, tile.width, tile.height));
        }

        renderer.draw(draw_list);
    }

    fn on_mouse_release(&mut self, widget: &Rc<RefCell<Widget>>, _kind: ClickKind) -> bool {
        let x = Cursor::get_x() - widget.borrow().state.position.x;
        let y = Cursor::get_y() - widget.borrow().state.position.y;

        trace!("Getting tile {:?}", self.tile_picker.borrow().state.get_text_arg("current_tile"));

        let cur_tile = match self.tile_picker.borrow().state.get_text_arg("current_tile") {
            None => return true,
            Some(tile) => match Module::tile(tile) {
                None => { trace!("Didn't find"); return true; },
                Some(tile) => tile,
            }
        };

        self.tiles.push((Point::new(x, y), cur_tile));
        true
    }
}
