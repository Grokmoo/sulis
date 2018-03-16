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

use std::slice::Iter;
use std::rc::Rc;
use std::cell::RefCell;

use rlua::{Lua, UserData, UserDataMethods};

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{animation_state, color, Cursor};
use sulis_module::{Ability, Module};

use script::{Result, ScriptEntity, ScriptEntitySet};
use {EntityState, GameState};

pub struct Targeter {
    ability: Rc<Ability>,
    parent: Rc<RefCell<EntityState>>,
    targets: Vec<Rc<RefCell<EntityState>>>,

    cur_target: Option<Rc<RefCell<EntityState>>>,

    cancel: bool,
}

impl Targeter {
    pub fn target_iter<'a>(&'a self) -> Iter<'a, Rc<RefCell<EntityState>>> {
        self.targets.iter()
    }

    pub fn parent(&self) -> &Rc<RefCell<EntityState>> {
        &self.parent
    }

    pub fn cancel(&self) -> bool {
        self.cancel
    }

    pub fn draw(&self, renderer: &mut GraphicsRenderer, x_offset: f32, y_offset: f32,
                scale_x: f32, scale_y: f32) {
        let mut draw_list = DrawList::empty_sprite();

        for target in self.targets.iter() {
            draw_list.append(&mut self.draw_target(target, x_offset, y_offset));
        }

        if !draw_list.is_empty() {
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        if let Some(ref target) = self.cur_target {
            let mut draw_list = self.draw_target(target, x_offset, y_offset);
            draw_list.set_scale(scale_x, scale_y);
            draw_list.set_color(color::RED);
            renderer.draw(draw_list);
        }
    }

    fn draw_target(&self, target: &Rc<RefCell<EntityState>>, x_offset: f32, y_offset: f32) -> DrawList {
        let target = target.borrow();
        DrawList::from_sprite_f32(&target.size.cursor_sprite,
                                  target.location.x as f32 - x_offset,
                                  target.location.y as f32 - y_offset,
                                  target.size.width as f32,
                                  target.size.height as f32)
    }

    pub fn on_mouse_move(&mut self, cursor_x: i32, cursor_y: i32) {
        for target in self.targets.iter() {
            {
                let target = target.borrow();
                let x1 = target.location.x;
                let y1 = target.location.y;
                let x2 = x1 + target.size.width;
                let y2 = y1 + target.size.height;

                if cursor_x < x1 || cursor_x > x2 || cursor_y < y1 || cursor_y > y2 { continue; }
            }

            self.cur_target = Some(Rc::clone(target));
            Cursor::set_cursor_state(animation_state::Kind::MouseSelect);
            return;
        }

        self.cur_target = None;
        Cursor::set_cursor_state(animation_state::Kind::MouseInvalid);
    }

    pub fn on_mouse_release(&mut self) {
        self.cancel = true;

        let cur_target = match self.cur_target {
            None => return,
            Some(ref target) => Rc::clone(target),
        };

        GameState::execute_ability_on_target_select(&self.parent, &self.ability, vec![cur_target]);
    }
}

impl<'a> From<&'a TargeterData> for Targeter {
    fn from(data: &TargeterData) -> Targeter {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();

        let parent = area_state.get_entity(data.parent);
        let targets = data.targets.iter().map(|t| area_state.get_entity(*t)).collect();

        Targeter {
            ability: Module::ability(&data.ability_id).unwrap(),
            parent,
            targets,
            cancel: false,
            cur_target: None,
        }
    }
}

#[derive(Clone)]
pub struct TargeterData {
    ability_id: String,
    parent: usize,
    targets: Vec<usize>,
}

impl TargeterData {
    pub fn new(parent: usize, ability_id: &str) -> TargeterData {
        TargeterData {
            parent,
            ability_id: ability_id.to_string(),
            targets: Vec::new(),
        }
    }
}

impl UserData for TargeterData {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("activate", &activate);
        methods.add_method_mut("add_all", |_, targeter, targets: ScriptEntitySet| {
            targeter.targets.append(&mut targets.indices.clone());
            Ok(())
        });
        methods.add_method_mut("add", |_, targeter, target: ScriptEntity| {
            targeter.targets.push(target.index);
            Ok(())
        });
    }
}

fn activate(_lua: &Lua, data: &TargeterData, _args: ()) -> Result<()> {
    info!("Activating targeter");

    let targeter = Targeter::from(data);

    let area_state = GameState::area_state();
    area_state.borrow_mut().set_targeter(targeter);
    Ok(())
}
