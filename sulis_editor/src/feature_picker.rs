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
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_core::widgets::{Button, ScrollPane, ScrollDirection};
use sulis_module::{area::tile::Feature, Module};

use crate::{AreaModel, EditorMode};

const NAME: &str = "feature_picker";

fn draw(
    feature: &Rc<Feature>,
    renderer: &mut dyn GraphicsRenderer,
    x: f32,
    y: f32,
    s_x: f32,
    s_y: f32,
) {
    for (tile, p) in &feature.preview {
        let mut draw_list = DrawList::from_sprite_f32(
            &tile.image_display,
            x + p.x as f32,
            y + p.y as f32,
            tile.width as f32,
            tile.height as f32,
        );
        draw_list.set_scale(s_x, s_y);
        renderer.draw(draw_list);
    }
}

pub struct FeaturePicker {
    cur_feature: Option<Rc<Feature>>,
    cursor_pos: Option<Point>,
}

impl FeaturePicker {
    pub fn new() -> Rc<RefCell<FeaturePicker>> {
        Rc::new(RefCell::new(FeaturePicker {
            cur_feature: None,
            cursor_pos: None,
        }))
    }
}

impl EditorMode for FeaturePicker {
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
        let feature = match self.cur_feature {
            None => return,
            Some(ref feature) => feature,
        };

        let pos = match self.cursor_pos {
            None => return,
            Some(pos) => pos,
        };

        draw(
            feature,
            renderer,
            x + pos.x as f32,
            y + pos.y as f32,
            scale_x,
            scale_y,
        );
    }

    fn cursor_size(&self) -> (i32, i32) {
        match self.cur_feature {
            None => (0, 0),
            Some(ref feature) => (feature.size.width, feature.size.height),
        }
    }

    fn mouse_move(&mut self, _model: &mut AreaModel, x: i32, y: i32) {
        self.cursor_pos = Some(Point::new(x, y));
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let feature = match self.cur_feature {
            None => return,
            Some(ref feature) => feature,
        };

        for (ref tile, point) in feature.rand_entry() {
            let tile = Some(Rc::clone(tile));
            model.add_tile(&tile, x + point.x, y + point.y);
        }
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let feature = match self.cur_feature {
            None => return,
            Some(ref feature) => feature,
        };

        for (tile, _) in &feature.preview {
            let layer = &tile.layer;
            model.remove_tiles_within(layer, x, y, feature.size.width, feature.size.height);
        }
    }
}

impl WidgetKind for FeaturePicker {
    fn get_name(&self) -> &str {
        NAME
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut all_features = Module::all_features();
        all_features.sort_by(|a, b| a.id.cmp(&b.id));

        let scrollpane = ScrollPane::new(ScrollDirection::Vertical);
        for feature in all_features {
            let button = Widget::with_theme(Button::empty(), "feature_button");
            button.borrow_mut().state.add_text_arg("name", &feature.id);
            button
                .borrow_mut()
                .state
                .add_callback(Callback::new(Rc::new(move |widget, _| {
                    let parent = Widget::direct_parent(widget);
                    let cur_state = widget.borrow_mut().state.is_active();
                    if !cur_state {
                        trace!("Set active feature: {}", widget.borrow().state.text);
                        for child in parent.borrow().children.iter() {
                            child.borrow_mut().state.set_active(false);
                        }
                        widget.borrow_mut().state.set_active(true);
                    }

                    let (_, feature_picker) = Widget::parent_mut::<FeaturePicker>(&parent);
                    feature_picker.cur_feature = Some(Rc::clone(&feature));
                })));

            scrollpane.borrow().add_to_content(button);
        }

        vec![Widget::with_theme(scrollpane, "features")]
    }
}
