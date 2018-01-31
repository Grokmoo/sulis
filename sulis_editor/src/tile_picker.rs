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

use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_module::Module;
use sulis_module::area::Tile;
use sulis_widgets::Button;

const NAME: &str = "tile_picker";

pub struct TilePicker {
    cur_tile: Option<Rc<Tile>>,
    cur_layer: Option<String>,
}

impl TilePicker {
    pub fn new() -> Rc<RefCell<TilePicker>> {
        Rc::new(RefCell::new(TilePicker {
            cur_tile: None,
            cur_layer: None,
        }))
    }

    pub fn get_cur_tile(&self) -> Option<Rc<Tile>> {
        match self.cur_tile {
            None => None,
            Some(ref tile) => Some(Rc::clone(tile)),
        }
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
            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget| {
                let parent = Widget::go_up_tree(widget, 2);

                {
                    let kind = &parent.borrow().kind;
                    let mut kind = kind.borrow_mut();
                    let tile_picker = match kind.as_any_mut().downcast_mut::<TilePicker>() {
                        Some(mut tile_picker) => tile_picker,
                        None => unreachable!("Failed to downcast to tilepicker"),
                    };
                    tile_picker.cur_layer = Some(layer_ref.clone());
                }
                parent.borrow_mut().invalidate_children();
            })));

            Widget::add_child_to(&layers_content, button);
        }

        let cur_layer = match self.cur_layer {
            None => return vec![layers_content],
            Some(ref layer) => layer,
        };

        let tiles_content = Widget::empty("tiles_content");
        for tile in all_tiles {
            if &tile.layer != cur_layer { continue; }

            let button = Widget::with_theme(Button::with_text(&tile.name), "tile_button");
            let sprite_id = format!("{}/{}", tile.image_display.id, tile.id);
            button.borrow_mut().state.add_text_arg("icon", &sprite_id);

            let cb: Callback = Callback::new(Rc::new(move |widget| {
                let parent = Widget::get_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    trace!("Set active: {}", widget.borrow().state.text);
                    for child in parent.borrow_mut().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);

                    let parent = Widget::get_parent(&parent);
                    let kind = &parent.borrow().kind;
                    let mut kind = kind.borrow_mut();
                    let tile_picker = match kind.as_any_mut().downcast_mut::<TilePicker>() {
                        Some(mut tile_picker) => tile_picker,
                        None => unreachable!("Failed to downcast to tilepicker"),
                    };
                    tile_picker.cur_tile = Some(Rc::clone(&tile));
                } else {
                    widget.borrow_mut().state.set_active(false);
                }
            }));

            button.borrow_mut().state.add_callback(cb);
            Widget::add_child_to(&tiles_content, button);
        }

        vec![layers_content, tiles_content]
    }
}
