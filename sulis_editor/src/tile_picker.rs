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

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_core::widgets::{Button, ScrollPane};
use sulis_module::area::Tile;
use sulis_module::Module;

use crate::{AreaModel, EditorMode};

const NAME: &str = "tile_picker";

pub struct TilePicker {
    cur_tile: Option<Rc<Tile>>,
    cur_layer: Option<String>,
    removal_tiles: Vec<(Point, Rc<Tile>)>,
    cursor_pos: Option<Point>,
}

impl TilePicker {
    pub fn new() -> Rc<RefCell<TilePicker>> {
        Rc::new(RefCell::new(TilePicker {
            cur_tile: None,
            cur_layer: None,
            cursor_pos: None,
            removal_tiles: Vec::new(),
        }))
    }
}

impl EditorMode for TilePicker {
    fn draw_mode(
        &mut self,
        renderer: &mut GraphicsRenderer,
        _model: &AreaModel,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
        _millis: u32,
    ) {
        if !self.removal_tiles.is_empty() {
            let mut draw_list = DrawList::empty_sprite();
            for &(pos, ref tile) in self.removal_tiles.iter() {
                let x = x + pos.x as f32;
                let y = y + pos.y as f32;
                draw_list.append(&mut DrawList::from_sprite_f32(
                    &tile.image_display,
                    x,
                    y,
                    tile.width as f32,
                    tile.height as f32,
                ));
            }

            draw_list.set_color(Color::from_string("F008"));
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        let tile = match self.cur_tile {
            None => return,
            Some(ref tile) => tile,
        };

        let pos = match self.cursor_pos {
            None => return,
            Some(pos) => pos,
        };

        let x = x + pos.x as f32;
        let y = y + pos.y as f32;
        let mut draw_list = DrawList::from_sprite_f32(
            &tile.image_display,
            x,
            y,
            tile.width as f32,
            tile.height as f32,
        );
        draw_list.set_color(Color::from_string("FFF8"));
        draw_list.set_scale(scale_x, scale_y);
        renderer.draw(draw_list);
    }

    fn cursor_size(&self) -> (i32, i32) {
        match self.cur_tile {
            None => (0, 0),
            Some(ref tile) => (tile.width as i32, tile.height as i32),
        }
    }

    fn mouse_move(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        self.cursor_pos = Some(Point::new(x, y));

        let tile = match self.cur_tile {
            None => return,
            Some(ref tile) => tile,
        };

        let layer_id = match self.cur_layer {
            None => return,
            Some(ref layer) => layer,
        };
        self.removal_tiles = model.tiles_within(layer_id, x, y, tile.width, tile.height);
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        model.add_tile(&self.cur_tile, x, y);
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let tile = match self.cur_tile {
            None => return,
            Some(ref tile) => tile,
        };

        let layer_id = match self.cur_layer {
            None => return,
            Some(ref layer) => layer,
        };

        self.removal_tiles.clear();
        model.remove_tiles_within(layer_id, x, y, tile.width, tile.height);
    }
}

impl WidgetKind for TilePicker {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut all_tiles = Module::all_tiles();
        all_tiles.sort_by(|a, b| a.id.cmp(&b.id));

        let mut layers: Vec<String> = Vec::new();
        for tile in all_tiles.iter() {
            if !layers.contains(&tile.layer) {
                layers.push(tile.layer.clone());
            }
        }

        let layers_content = Widget::empty("layers_content");
        for layer in layers {
            let button = Widget::with_theme(Button::with_text(&layer), "layer_button");
            let layer_ref = layer.clone();
            button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let (parent, tile_picker) = Widget::parent_mut::<TilePicker>(widget);
                    tile_picker.cur_layer = Some(layer_ref.clone());
                    parent.borrow_mut().invalidate_children();
                })));

            Widget::add_child_to(&layers_content, button);
        }

        let cur_layer = match self.cur_layer {
            None => return vec![layers_content],
            Some(ref layer) => layer,
        };

        let scrollpane = ScrollPane::new();
        for tile in all_tiles {
            if &tile.layer != cur_layer {
                continue;
            }

            let button = Widget::with_theme(Button::empty(), "tile_button");
            button
                .borrow_mut()
                .state
                .add_text_arg("icon", &tile.image_display.full_id());

            let cb: Callback = Callback::new(Rc::new(move |widget, _kind| {
                let parent = Widget::direct_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    trace!("Set active: {}", widget.borrow().state.text);
                    for child in parent.borrow_mut().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);

                    let (_, tile_picker) = Widget::parent_mut::<TilePicker>(&parent);
                    tile_picker.cur_tile = Some(Rc::clone(&tile));
                }
            }));

            button.borrow_mut().state.add_callback(cb);
            scrollpane.borrow().add_to_content(button);
        }

        vec![Widget::with_theme(scrollpane, "tiles"), layers_content]
    }
}
