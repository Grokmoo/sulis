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
use sulis_core::ui::{Callback, Widget, WidgetKind, RcRfc};
use sulis_core::util::Point;
use sulis_core::widgets::{Label, Spinner};

use crate::{AreaModel, EditorMode};

const NAME: &str = "trigger_picker";

pub struct TriggerPicker {
    cur_width: i32,
    cur_height: i32,
    cursor_pos: Option<Point>,

    trigger_sprite: Option<Rc<Sprite>>,
}

impl TriggerPicker {
    pub fn new() -> RcRfc<TriggerPicker> {
        let enc_tile = Config::editor_config().area.encounter_tile;

        let sprite = match ResourceSet::sprite(&enc_tile) {
            Ok(sprite) => Some(sprite),
            Err(_) => {
                warn!("Encounter tile '{}' not found", enc_tile);
                None
            }
        };

        Rc::new(RefCell::new(TriggerPicker {
            cursor_pos: None,
            trigger_sprite: sprite,
            cur_width: 10,
            cur_height: 10,
        }))
    }
}

impl EditorMode for TriggerPicker {
    fn draw_mode(
        &mut self,
        renderer: &mut dyn GraphicsRenderer,
        _model: &AreaModel,
        x: f32,
        y: f32,
        scale_x: f32,
        scale_y: f32,
        _millis: u32,
    ) {
        let pos = match self.cursor_pos {
            None => return,
            Some(pos) => pos,
        };

        if let Some(ref sprite) = self.trigger_sprite {
            let x = x + pos.x as f32;
            let y = y + pos.y as f32;
            let w = self.cur_width as f32;
            let h = self.cur_height as f32;
            let mut draw_list = DrawList::from_sprite_f32(sprite, x, y, w, h);
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }
    }

    fn cursor_size(&self) -> (i32, i32) {
        (self.cur_width, self.cur_height)
    }

    fn mouse_move(&mut self, _model: &mut AreaModel, x: i32, y: i32) {
        self.cursor_pos = Some(Point::new(x, y));
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        model.add_trigger(x, y, self.cur_width, self.cur_height);
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        model.remove_triggers_within(x, y, self.cur_width, self.cur_height);
    }
}

impl WidgetKind for TriggerPicker {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, _widget: &RcRfc<Widget>) -> Vec<RcRfc<Widget>> {
        let width = Widget::with_theme(Spinner::new(self.cur_width, 1, 50), "width");
        width
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, kind| {
                let (_, picker) = Widget::parent_mut::<TriggerPicker>(widget);

                let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                    None => panic!("Unable to downcast to spinner"),
                    Some(widget) => widget,
                };

                picker.cur_width = spinner.value();
            })));
        let height = Widget::with_theme(Spinner::new(self.cur_height, 1, 50), "height");
        height
            .borrow_mut()
            .state
            .add_callback(Callback::new(Rc::new(|widget, kind| {
                let (_, picker) = Widget::parent_mut::<TriggerPicker>(widget);

                let spinner = match kind.as_any().downcast_ref::<Spinner>() {
                    None => panic!("Unable to downcast to spinner"),
                    Some(widget) => widget,
                };

                picker.cur_height = spinner.value();
            })));

        let size_label = Widget::with_theme(Label::empty(), "size_label");

        vec![width, height, size_label]
    }
}
