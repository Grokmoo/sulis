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

use sulis_core::config::Config;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::ui::{Color, Widget, WidgetKind};

use crate::{AreaModel, EditorMode};

const NAME: &str = "elevation_picker";

pub struct VisPicker {
    cursor_sprite: Rc<Sprite>,
}

impl VisPicker {
    pub fn new() -> Rc<RefCell<VisPicker>> {
        let cursor_sprite = match ResourceSet::sprite(&Config::editor_config().cursor) {
            Err(_) => panic!(
                "Unable to find cursor sprite '{}'",
                Config::editor_config().cursor
            ),
            Ok(sprite) => sprite,
        };

        Rc::new(RefCell::new(VisPicker { cursor_sprite }))
    }
}

impl EditorMode for VisPicker {
    fn draw_mode(
        &mut self,
        renderer: &mut GraphicsRenderer,
        model: &AreaModel,
        x_offset: f32,
        y_offset: f32,
        scale_x: f32,
        scale_y: f32,
        _millis: u32,
    ) {
        let mut draw_list = DrawList::empty_sprite();
        for (p, tile) in model.all_tiles() {
            let x_base = x_offset + p.x as f32;
            let y_base = y_offset + p.y as f32;
            for p in tile.invis.iter() {
                let x = p.x as f32 + x_base;
                let y = p.y as f32 + y_base;
                draw_list.append(&mut DrawList::from_sprite_f32(
                    &self.cursor_sprite,
                    x,
                    y,
                    1.0,
                    1.0,
                ));
            }
        }
        draw_list.set_scale(scale_x, scale_y);
        draw_list.set_color(Color::from_string("F008"));

        renderer.draw(draw_list);
    }

    fn cursor_size(&self) -> (i32, i32) {
        (1, 1)
    }

    fn mouse_move(&mut self, _model: &mut AreaModel, _x: i32, _y: i32) {}

    fn left_click(&mut self, _model: &mut AreaModel, _x: i32, _y: i32) {}

    fn right_click(&mut self, _model: &mut AreaModel, _x: i32, _y: i32) {}
}

impl WidgetKind for VisPicker {
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
        Vec::new()
    }
}
