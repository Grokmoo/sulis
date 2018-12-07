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
use sulis_core::ui::{animation_state, Callback, Color, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_module::{Prop, Module};
use sulis_widgets::Button;

use crate::{AreaModel, EditorMode};

const NAME: &str = "prop_picker";

pub struct PropPicker {
    cur_prop: Option<Rc<Prop>>,
    removal_props: Vec<(Point, Rc<Prop>)>,
    cursor_pos: Option<Point>,
}

impl PropPicker {
    pub fn new() -> Rc<RefCell<PropPicker>> {
        Rc::new(RefCell::new(PropPicker {
            cur_prop: None,
            removal_props: Vec::new(),
            cursor_pos: None,
        }))
    }
}

impl EditorMode for PropPicker {
    fn draw_mode(&mut self, renderer: &mut GraphicsRenderer, _model: &AreaModel, x: f32, y: f32,
            scale_x: f32, scale_y: f32, millis: u32) {

        for &(pos, ref prop) in self.removal_props.iter() {
            let x = x + pos.x as f32;
            let y = y + pos.y as f32;
            let mut draw_list = DrawList::empty_sprite();
            prop.append_to_draw_list(&mut draw_list, &animation_state::NORMAL, x, y, millis);
            draw_list.set_color(Color::from_string("FFF8"));
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        let prop = match self.cur_prop {
            None => return,
            Some(ref prop) => prop,
        };

        let pos = match self.cursor_pos {
            None => return,
            Some(pos) => pos,
        };

        let mut draw_list = DrawList::empty_sprite();
        let x = x + pos.x as f32;
        let y = y + pos.y as f32;
        prop.append_to_draw_list(&mut draw_list, &animation_state::NORMAL, x, y, millis);
        draw_list.set_color(Color::from_string("FFF8"));
        draw_list.set_scale(scale_x, scale_y);
        renderer.draw(draw_list);
    }

    fn cursor_size(&self) -> (i32, i32) {
        match self.cur_prop {
            None => (0, 0),
            Some(ref prop) => (prop.size.width, prop.size.height),
        }
    }

    fn mouse_move(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        self.cursor_pos = Some(Point::new(x, y));

        let prop = match self.cur_prop {
            None => return,
            Some(ref prop) => prop,
        };

        self.removal_props = model.props_within(x, y, prop.size.width, prop.size.height);
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let prop = match self.cur_prop {
            None => return,
            Some(ref prop) => prop,
        };

        model.add_prop(Rc::clone(prop), x, y);
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let prop = match self.cur_prop {
            None => return,
            Some(ref prop) => prop,
        };

        self.removal_props.clear();
        model.remove_props_within(x, y, prop.size.width, prop.size.height);
    }
}

impl WidgetKind for PropPicker {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut all_props = Module::all_props();
        all_props.sort_by(|a, b| a.id.cmp(&b.id));

        let mut widgets: Vec<Rc<RefCell<Widget>>> = Vec::new();
        for prop in all_props {
            let button = Widget::with_theme(Button::empty(), "prop_button");
            button.borrow_mut().state.add_text_arg("name", &prop.id);
            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::get_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    trace!("Set active prop: {}", widget.borrow().state.text);
                    for child in parent.borrow().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);
                }

                let prop_picker = Widget::downcast_kind_mut::<PropPicker>(&parent);
                prop_picker.cur_prop = Some(Rc::clone(&prop));
            })));

            widgets.push(button);
        }

        widgets
    }
}
