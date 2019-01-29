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
use sulis_core::resource::{ResourceSet, Sprite};
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{Callback, Color, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_widgets::{Label, Spinner};
use sulis_module::area::MAX_AREA_SIZE;

use crate::{AreaModel, EditorMode};

const NAME: &str = "elevation_picker";

pub struct ElevPicker {
    cursor_sprite: Rc<Sprite>,
    cursor_pos: Option<Point>,
    elev_tiles: Vec<Rc<Sprite>>,

    brush_size: i32,
    set_elev_to: u8,
}

impl ElevPicker {
    pub fn new() -> Rc<RefCell<ElevPicker>> {
        let cursor_sprite = match ResourceSet::sprite(&Config::editor_config().cursor) {
            Err(_) => panic!("Unable to find cursor sprite '{}'", Config::editor_config().cursor),
            Ok(sprite) => sprite,
        };

        let mut elev_tiles = Vec::new();
        for ref sprite_id in Config::editor_config().area.elev_tiles.iter() {
            let sprite = match ResourceSet::sprite(sprite_id) {
                Err(_) => {
                    warn!("Editor elevation tile '{}' not found", sprite_id);
                    continue;
                }, Ok(sprite) => sprite,
            };
            elev_tiles.push(sprite);
        }

        Rc::new(RefCell::new(ElevPicker {
            cursor_sprite,
            cursor_pos: None,
            elev_tiles,
            brush_size: 4,
            set_elev_to: 1,
        }))
    }
}

impl EditorMode for ElevPicker {
    fn draw_mode(&mut self, renderer: &mut GraphicsRenderer, model: &AreaModel, x_offset: f32, y_offset: f32,
            scale_x: f32, scale_y: f32, _millis: u32) {
        let mut draw_list = DrawList::empty_sprite();
        for y in 0..MAX_AREA_SIZE {
            for x in 0..MAX_AREA_SIZE {
                let sprite = &self.elev_tiles[model.elevation(x, y) as usize];
                let x = x as f32 + x_offset;
                let y = y as f32 + y_offset;
                draw_list.append(&mut DrawList::from_sprite_f32(&sprite, x, y, 1.0, 1.0));
            }
        }
        draw_list.set_scale(scale_x, scale_y);

        renderer.draw(draw_list);

        let mut draw_list = DrawList::empty_sprite();
        if let Some(pos) = self.cursor_pos {
            for y in 0..self.brush_size {
                for x in 0..self.brush_size {
                    let x = x as f32 + pos.x as f32 + x_offset;
                    let y = y as f32 + pos.y as f32 + y_offset;
                    draw_list.append(&mut DrawList::from_sprite_f32(&self.cursor_sprite, x, y, 1.0, 1.0));
                }
            }
            draw_list.set_scale(scale_x, scale_y);
            draw_list.set_color(Color::from_string("0F08"));
            renderer.draw(draw_list);
        }
    }

    fn cursor_size(&self) -> (i32, i32) {
        (self.brush_size, self.brush_size)
    }

    fn mouse_move(&mut self, _model: &mut AreaModel, x: i32, y: i32) {
        self.cursor_pos = Some(Point::new(x, y));
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        for y in y..(y + self.brush_size) {
            for x in x..(x + self.brush_size) {
                model.set_elevation(self.set_elev_to, x, y);
            }
        }
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        for y in y..(y + self.brush_size) {
            for x in x..(x + self.brush_size) {
                model.set_elevation(0, x, y);
            }
        }
    }
}

impl WidgetKind for ElevPicker {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let brush_size = Widget::with_theme(Spinner::new(self.brush_size, 1, 10), "brush_size");
        brush_size.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<ElevPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast to spinner"),
                Some(widget) => widget,
            };

            picker.brush_size = spinner.value();
        })));

        let brush_size_label = Widget::with_theme(Label::empty(), "brush_size_label");

        let elev = Widget::with_theme(
            Spinner::new(self.set_elev_to as i32, 0,
                         Config::editor_config().area.elev_tiles.len() as i32 - 1), "elev");
        elev.borrow_mut().state.add_callback(Callback::new(Rc::new(|widget, kind| {
            let parent = Widget::get_parent(&widget);
            let picker = Widget::downcast_kind_mut::<ElevPicker>(&parent);

            let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                None => panic!("Unable to downcast to spinner"),
                Some(widget) => widget,
            };

            picker.set_elev_to = spinner.value() as u8;
        })));

        let elev_label = Widget::with_theme(Label::empty(), "elev_label");

        vec![brush_size, brush_size_label, elev, elev_label]
    }
}
