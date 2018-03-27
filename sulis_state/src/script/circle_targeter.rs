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

use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::image::Image;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{animation_state, color, Cursor};
use sulis_core::util::Point;
use sulis_module::{Ability, Module};

use script::{Targeter, TargeterData};
use {EntityState, GameState};

pub struct CircleTargeter {
    ability: Rc<Ability>,
    parent: Rc<RefCell<EntityState>>,
    targets: Vec<Rc<RefCell<EntityState>>>,

    radius: f32,
    cur_target: Option<Rc<RefCell<EntityState>>>,
    cur_points: Vec<Point>,
    cur_affected: Vec<Rc<RefCell<EntityState>>>,

    cancel: bool,
}

impl CircleTargeter {
    pub fn from(data: &TargeterData, radius: f32) -> CircleTargeter {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();

        let parent = area_state.get_entity(data.parent);
        let targets = data.targets.iter().map(|t| area_state.get_entity(*t)).collect();

        CircleTargeter {
            ability: Module::ability(&data.ability_id).unwrap(),
            parent,
            targets,
            cancel: false,
            cur_target: None,
            cur_points: Vec::new(),
            cur_affected: Vec::new(),
            radius,
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

    fn calculate_points(&mut self) {
        self.cur_points.clear();
        self.cur_affected.clear();
        let target = match self.cur_target {
            None => return,
            Some(ref target) => target,
        };

        let center_x = target.borrow().center_x();
        let center_y = target.borrow().center_y();

        let r = (self.radius + 1.0).ceil() as i32;
        let shift = if target.borrow().size.width % 2 == 0 {
            0.5
        } else {
            0.0
        };

        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        for y in -r..r {
            for x in -r..r {
                if (x as f32 + shift).hypot(y as f32 + shift) > self.radius { continue; }

                let x = x + center_x;
                let y = y + center_y;

                if let Some(entity) = area_state.get_entity_at(x, y) {
                    if Rc::ptr_eq(target, &entity) { continue; }

                    let mut already_present = false;
                    for e in self.cur_affected.iter() {
                        if Rc::ptr_eq(e, &entity) {
                            already_present = true;
                            break;
                        }
                    }
                    if !already_present { self.cur_affected.push(entity); }
                }

                self.cur_points.push(Point::new(x, y));
            }
        }
    }
}

impl Targeter for CircleTargeter {
    fn cancel(&self) -> bool {
        self.cancel
    }

    fn draw(&self, renderer: &mut GraphicsRenderer, tile: &Rc<Image>, x_offset: f32, y_offset: f32,
                scale_x: f32, scale_y: f32, millis: u32) {
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
            for target in self.cur_affected.iter() {
                draw_list.append(&mut self.draw_target(target, x_offset, y_offset));
            }

            draw_list.set_scale(scale_x, scale_y);
            draw_list.set_color(color::RED);
            renderer.draw(draw_list);

            let mut draw_list = DrawList::empty_sprite();
            for p in self.cur_points.iter() {
                let x = p.x as f32 - x_offset;
                let y = p.y as f32 - y_offset;
                tile.append_to_draw_list(&mut draw_list, &animation_state::NORMAL, x, y, 1.0, 1.0, millis);
            }

            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }
    }

    fn on_mouse_move(&mut self, cursor_x: i32, cursor_y: i32) -> Option<&Rc<RefCell<EntityState>>> {
        self.cur_target = None;

        for target in self.targets.iter() {
            {
                let target = target.borrow();
                let x1 = target.location.x;
                let y1 = target.location.y;
                let x2 = x1 + target.size.width - 1;
                let y2 = y1 + target.size.height - 1;

                if cursor_x < x1 || cursor_x > x2 || cursor_y < y1 || cursor_y > y2 { continue; }
            }

            self.cur_target = Some(Rc::clone(target));
            break;
        }

        self.calculate_points();
        let kind = match self.cur_target {
            None => animation_state::Kind::MouseInvalid,
            Some(_) => animation_state::Kind::MouseSelect,
        };
        Cursor::set_cursor_state(kind);

        self.cur_target.as_ref()
    }

    fn on_mouse_release(&mut self) {
        self.cancel = true;

        match self.cur_target {
            None => return,
            Some(_) => (),
        };

        GameState::execute_ability_on_target_select(&self.parent, &self.ability,
                                                    self.cur_affected.clone());
    }
}
