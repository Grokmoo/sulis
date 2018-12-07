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

use sulis_core::io::{GraphicsRenderer};
use sulis_core::ui::{Callback, Widget, WidgetKind};
use sulis_core::util::Point;
use sulis_module::{Actor, Module};
use sulis_widgets::Button;

use crate::{AreaModel, EditorMode};

const NAME: &str = "actor_picker";

pub struct ActorPicker {
    cur_actor: Option<Rc<Actor>>,
    removal_actors: Vec<(Point, Rc<Actor>)>,
    cursor_pos: Option<Point>,
}

impl ActorPicker {
    pub fn new() -> Rc<RefCell<ActorPicker>> {
        Rc::new(RefCell::new(ActorPicker {
            cur_actor: None,
            removal_actors: Vec::new(),
            cursor_pos: None,
        }))
    }
}

impl EditorMode for ActorPicker {
    fn draw_mode(&mut self, renderer: &mut GraphicsRenderer, _model: &AreaModel, x: f32, y: f32,
            scale_x: f32, scale_y: f32, millis: u32) {

        for &(pos, ref actor) in self.removal_actors.iter() {
            let w = actor.race.size.width as f32 / 2.0;
            let h = actor.race.size.height as f32 / 2.0;
            actor.draw(renderer, scale_x, scale_y, x + pos.x as f32 - w,
                       y + pos.y as f32 - h, millis);
        }

        let actor = match self.cur_actor {
            None => return,
            Some(ref actor) => actor,
        };

        let pos = match self.cursor_pos {
            None => return,
            Some(pos) => pos,
        };

        let w = actor.race.size.width as f32 / 2.0;
        let h = actor.race.size.height as f32 / 2.0;
        actor.draw(renderer, scale_x, scale_y, x + pos.x as f32 - w,
                   y + pos.y as f32 - h, millis);
    }

    fn cursor_size(&self) -> (i32, i32) {
        match self.cur_actor {
            None => (0, 0),
            Some(ref actor) => (actor.race.size.width, actor.race.size.height),
        }
    }

    fn mouse_move(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        self.cursor_pos = Some(Point::new(x, y));

        let actor = match self.cur_actor {
            None => return,
            Some(ref actor) => actor,
        };

        self.removal_actors = model.actors_within(x, y, actor.race.size.width, actor.race.size.height);
    }

    fn left_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let actor = match self.cur_actor {
            None => return,
            Some(ref actor) => actor,
        };

        model.add_actor(Rc::clone(actor), x, y);
    }

    fn right_click(&mut self, model: &mut AreaModel, x: i32, y: i32) {
        let actor = match self.cur_actor {
            None => return,
            Some(ref actor) => actor,
        };

        self.removal_actors.clear();
        model.remove_actors_within(x, y, actor.race.size.width, actor.race.size.height);
    }
}

impl WidgetKind for ActorPicker {
    fn get_name(&self) -> &str { NAME }

    fn as_any(&self) -> &Any { self }

    fn as_any_mut(&mut self) -> &mut Any { self }

    fn on_add(&mut self, _widget: &Rc<RefCell<Widget>>) -> Vec<Rc<RefCell<Widget>>> {
        let mut all_actors = Module::all_actors();
        all_actors.sort_by(|a, b| a.id.cmp(&b.id));

        let mut widgets: Vec<Rc<RefCell<Widget>>> = Vec::new();
        for actor in all_actors {
            let button = Widget::with_theme(Button::empty(), "actor_button");
            button.borrow_mut().state.add_text_arg("name", &actor.id);
            button.borrow_mut().state.add_callback(Callback::new(Rc::new(move |widget, _| {
                let parent = Widget::get_parent(widget);
                let cur_state = widget.borrow_mut().state.is_active();
                if !cur_state {
                    trace!("Set active actor: {}", widget.borrow().state.text);
                    for child in parent.borrow().children.iter() {
                        child.borrow_mut().state.set_active(false);
                    }
                    widget.borrow_mut().state.set_active(true);
                }

                let actor_picker = Widget::downcast_kind_mut::<ActorPicker>(&parent);
                actor_picker.cur_actor = Some(Rc::clone(&actor));
            })));

            widgets.push(button);
        }

        widgets
    }
}
