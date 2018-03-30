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

use std::f32::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;

use sulis_core::image::Image;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{animation_state, color, Cursor};
use sulis_core::util::{Point};
use sulis_module::{Ability, Module, ObjectSize};

use script::{Targeter, TargeterData};
use {AreaState, EntityState, GameState};

#[derive(Clone)]
pub enum Shape {
    Single,
    Circle { radius: f32 },
    Line { size: String, origin_x: i32, origin_y: i32 },
    ObjectSize { size: String },
    Cone { origin_x: i32, origin_y: i32, radius: f32, angle: f32 },
}

fn contains(target: &Rc<RefCell<EntityState>>, list: &Vec<Rc<RefCell<EntityState>>>) -> bool {
    for entity in list.iter() {
        if Rc::ptr_eq(target, entity) { return true; }
    }

    false
}

fn cast_high(area: &AreaState, size: &Rc<ObjectSize>, start: Point, end: Point) -> Vec<Point> {
    let mut points = Vec::new();

    let mut delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let xi = if delta_x < 0 { delta_x = -delta_x; -1 } else { 1 };

    let mut d = 2 * delta_x - delta_y;
    let mut x = start.x;
    for y in start.y..end.y {
        if !area.is_terrain_passable(&size.id, x, y) { return Vec::new(); }
        points.append(&mut size.points(x, y).collect());

        if d > 0 {
            x += xi;
            d -= 2 * delta_y;
        }
        d += 2 * delta_x;
    }

    if !area.is_terrain_passable(&size.id, end.x, end.y) { return Vec::new(); }
    points.append(&mut size.points(end.x, end.y).collect());
    points
}

fn cast_low(area: &AreaState, size: &Rc<ObjectSize>, start: Point, end: Point) -> Vec<Point> {
    let mut points = Vec::new();

    let mut delta_y = end.y - start.y;
    let delta_x = end.x - start.x;
    let yi = if delta_y < 0 { delta_y = -delta_y; -1 } else { 1 };

    let mut d = 2 * delta_y - delta_x;
    let mut y = start.y;
    for x in start.x..end.x {
        if !area.is_terrain_passable(&size.id, x, y) { return Vec::new(); }
        points.append(&mut size.points(x, y).collect());

        if d > 0 {
            y += yi;
            d -= 2 * delta_x;
        }
        d += 2 * delta_y;
    }

    if !area.is_terrain_passable(&size.id, end.x, end.y) { return Vec::new(); }
    points.append(&mut size.points(end.x, end.y).collect());
    points
}

fn get_cursor_offset_from_size(size: &str) -> Point {
    let size = match Module::object_size(size) {
        None => {
            warn!("Invalid object size in Targeter: '{}'", size);
            return Point::as_zero();
        }, Some(size) => size,
    };

    Point::new(size.width / 2, size.height / 2)
}

impl Shape {
    pub fn get_cursor_offset(&self) -> Point {
        match self {
            &Shape::Single | &Shape::Circle { .. } | &Shape::Cone { .. } => Point::as_zero(),
            &Shape::Line { ref size, .. } => get_cursor_offset_from_size(size),
            &Shape::ObjectSize { ref size } => get_cursor_offset_from_size(size),
        }
    }

    pub fn get_points(&self, pos: Point, shift: f32)-> Vec<Point> {
        match self {
            &Shape::Single => Vec::new(),
            &Shape::Circle { radius } => self.get_points_circle(radius, pos, shift),
            &Shape::Line { ref size, origin_x, origin_y } =>
                self.get_points_line(Point::new(origin_x, origin_y), pos, size),
            &Shape::ObjectSize { ref size } => self.get_points_object_size(pos, size),
            &Shape::Cone { origin_x, origin_y, radius, angle } =>
                self.get_points_cone(Point::new(origin_x, origin_y), pos, radius, angle),
        }
    }

    pub fn get_effected_entities(&self, points: &Vec<Point>, target: Option<&Rc<RefCell<EntityState>>>,
                                 effectable: &Vec<Rc<RefCell<EntityState>>>)
        -> Vec<Rc<RefCell<EntityState>>> {
        match self {
            &Shape::Single => {
                match target {
                    None => Vec::new(),
                    Some(ref target) => {
                        if contains(target, effectable) {
                            vec![Rc::clone(target)]
                        } else {
                            Vec::new()
                        }
                    }
                }
            },
            _ => self.get_effected(points, effectable),
        }
    }

    fn get_effected(&self, points: &Vec<Point>, effectable: &Vec<Rc<RefCell<EntityState>>>)
        -> Vec<Rc<RefCell<EntityState>>> {
        let mut effected = Vec::new();

        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        for p in points.iter() {
            let entity = match area_state.get_entity_at(p.x, p.y) {
                None => continue,
                Some(entity) => entity,
            };

            if !contains(&entity, &effectable) { continue; }

            if contains(&entity, &effected) { continue; }

            effected.push(entity);
        }

       effected
   }

    fn get_points_line(&self, start: Point, end: Point, size: &str) -> Vec<Point> {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();
        let size = match Module::object_size(size) {
            None => {
                warn!("Invalid object size in Targeter: '{}'", size);
                return Vec::new();
            }, Some(size) => size,
        };

        let mut points = if (end.y - start.y).abs() < (end.x - start.x).abs() {
            if start.x > end.x {
                cast_low(&area_state, &size, end, start)
            } else {
                cast_low(&area_state, &size, start, end)
            }
        } else {
            if start.y > end.y {
                cast_high(&area_state, &size, end, start)
            } else {
                cast_high(&area_state, &size, start, end)
            }
        };

        points.sort();
        points.dedup();
        points
    }

    fn get_points_cone(&self, origin: Point, to: Point, radius: f32,
                       angular_size: f32) -> Vec<Point> {
        let mut points = Vec::new();

        let angle = ((to.y - origin.y) as f32).atan2((to.x - origin.x) as f32);

        let r = (radius + 1.0).ceil() as i32;
        for y in -r..r {
            for x in -r..r {
                if (x as f32).hypot(y as f32) > radius { continue; }

                let cur_angle = (y as f32).atan2(x as f32);

                let angle_diff = (angle - cur_angle + 3.0 * PI) % (2.0 * PI) - PI;
                if angle_diff.abs() > angular_size / 2.0 { continue; }

                points.push(Point::new(x + origin.x, y + origin.y));
            }
        }

        points
    }

    fn get_points_circle(&self, radius: f32, pos: Point, shift: f32) -> Vec<Point> {
        let mut points = Vec::new();

        let r = (radius + 1.0).ceil() as i32;

        for y in -r..r {
            for x in -r..r {
                if (x as f32 + shift).hypot(y as f32 + shift) > radius { continue; }
                points.push(Point::new(x + pos.x, y + pos.y));
            }
        }
        points
    }

    fn get_points_object_size(&self, pos: Point, size: &str) -> Vec<Point> {
        let size = match Module::object_size(size) {
            None => {
                warn!("Invalid object size in Targeter: '{}'", size);
                return Vec::new();
            }, Some(size) => size,
        };
        size.points(pos.x, pos.y).collect()
    }
}

pub struct AreaTargeter {
    ability: Rc<Ability>,
    parent: Rc<RefCell<EntityState>>,
    selectable: Vec<Rc<RefCell<EntityState>>>,
    effectable: Vec<Rc<RefCell<EntityState>>>,
    shape: Shape,
    show_mouseover: bool,
    free_select: Option<f32>,
    free_select_must_be_passable: Option<Rc<ObjectSize>>,

    free_select_valid: bool,
    cur_target: Option<Rc<RefCell<EntityState>>>,
    cursor_pos: Point,
    cursor_offset: Point,
    cur_points: Vec<Point>,
    cur_effected: Vec<Rc<RefCell<EntityState>>>,

    cancel: bool,
}

fn create_entity_state_vec(area_state: &AreaState,
                           input: &Vec<Option<usize>>) -> Vec<Rc<RefCell<EntityState>>> {
    let mut out = Vec::new();
    for index in input.iter() {
        let index = match index {
            &None => continue,
            &Some(ref index) => *index,
        };

        match area_state.check_get_entity(index) {
            None => (),
            Some(entity) => out.push(entity),
        }
    }
    out
}

impl AreaTargeter {
    pub fn from(data: &TargeterData) -> AreaTargeter {
        let area_state = GameState::area_state();
        let area_state = area_state.borrow();

        let free_select_must_be_passable = match data.free_select_must_be_passable {
            None => None,
            Some(ref size) => match Module::object_size(size) {
                None => {
                    warn!("Invalid object size in Targeter: '{}'", size);
                    None
                }, Some(size) => Some(size),
            },
        };

        AreaTargeter {
            ability: Module::ability(&data.ability_id).unwrap(),
            parent: area_state.get_entity(data.parent),
            selectable: create_entity_state_vec(&area_state, &data.selectable),
            effectable: create_entity_state_vec(&area_state, &data.effectable),
            cancel: false,
            free_select: data.free_select,
            free_select_must_be_passable,
            free_select_valid: false,
            show_mouseover: data.show_mouseover,
            cur_target: None,
            cursor_pos: Point::as_zero(),
            cursor_offset: Point::as_zero(),
            cur_points: Vec::new(),
            cur_effected: Vec::new(),
            shape: data.shape.clone(),
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
        self.cur_effected.clear();

        if self.free_select.is_none() {
            let target = match self.cur_target {
                None => return,
                Some(ref target) => target,
            };

            let center_x = target.borrow().center_x() - self.cursor_offset.x;
            let center_y = target.borrow().center_y() - self.cursor_offset.y;
            let shift = if target.borrow().size.width % 2 == 0 { 0.5 } else { 0.0 };

            self.cur_points = self.shape.get_points(Point::new(center_x, center_y), shift);
            self.cur_effected = self.shape.get_effected_entities(&self.cur_points,
                                                                 Some(&target), &self.effectable);
        } else {
            if !self.free_select_valid { return; }

            let pos = self.cursor_pos - self.cursor_offset;
            self.cur_points = self.shape.get_points(pos, 0.0);
            self.cur_effected = self.shape.get_effected_entities(&self.cur_points, None,
                                                                 &self.effectable);

            if self.cur_points.is_empty() {
                self.free_select_valid = false;
            }
        }
    }

    fn compute_free_select_valid(&mut self) -> bool {
        let dist = match self.free_select {
            None => { return false; }
            Some(dist) => dist,
        };

        if self.parent.borrow().dist_to_point(self.cursor_pos) > dist {
            return false;
        }

        let area_state = GameState::area_state();
        if !area_state.borrow().is_pc_visible(self.cursor_pos.x, self.cursor_pos.y) {
            // TODO use the parent's visibility since it doesn't have to be a PC
            return false;
        }

        if let Some(ref size) = self.free_select_must_be_passable {
            if !area_state.borrow().is_passable_size(size, self.cursor_pos.x - size.width / 2,
                                                     self.cursor_pos.y - size.height / 2) {
                return false;
            }
        }

        true
    }
}

impl Targeter for AreaTargeter {
    fn name(&self) -> &str {
        &self.ability.name
    }

    fn cancel(&self) -> bool {
        self.cancel
    }

    fn draw(&self, renderer: &mut GraphicsRenderer, tile: &Rc<Image>, x_offset: f32, y_offset: f32,
                scale_x: f32, scale_y: f32, millis: u32) {
        let mut draw_list = DrawList::empty_sprite();

        for target in self.selectable.iter() {
            draw_list.append(&mut self.draw_target(target, x_offset, y_offset));
        }

        if !draw_list.is_empty() {
            draw_list.set_scale(scale_x, scale_y);
            renderer.draw(draw_list);
        }

        let mut draw_list = DrawList::empty_sprite();
        for target in self.cur_effected.iter() {
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

    fn on_mouse_move(&mut self, cursor_x: i32, cursor_y: i32) -> Option<&Rc<RefCell<EntityState>>> {
        self.cursor_pos = Point::new(cursor_x, cursor_y);
        self.cursor_offset = self.shape.get_cursor_offset();
        self.cur_target = None;

        for target in self.selectable.iter() {
            {
                let target = target.borrow();
                let x1 = target.location.x;
                let y1 = target.location.y;
                let x2 = x1 + target.size.width - 1;
                let y2 = y1 + target.size.height - 1;

                if cursor_x < x1 || cursor_x > x2 || cursor_y < y1 || cursor_y > y2 {
                    continue;
                }
            }

            self.cur_target = Some(Rc::clone(target));
            break;
        }

        self.free_select_valid = self.compute_free_select_valid();
        self.calculate_points();

        let kind = if self.free_select.is_none() {
            match self.cur_target {
                None => animation_state::Kind::MouseInvalid,
                Some(_) => animation_state::Kind::MouseSelect,
            }
        } else {
            match self.free_select_valid {
                false => animation_state::Kind::MouseInvalid,
                true => animation_state::Kind::MouseSelect,
            }
        };
        Cursor::set_cursor_state(kind);

        if self.show_mouseover {
            self.cur_target.as_ref()
        } else {
            None
        }
    }

    fn on_cancel(&mut self) {
        self.cancel = true;
    }

    fn on_activate(&mut self) {
        self.cancel = true;

        if self.free_select.is_none() {
            match self.cur_target {
                None => return,
                Some(_) => (),
            };
        } else {
            if !self.free_select_valid { return; }
        }

        let affected = self.cur_effected.iter().map(|e| Some(Rc::clone(e))).collect();

        let mut pos = self.cursor_pos;
        if let Some(ref size) = self.free_select_must_be_passable {
            pos.x -= size.width / 2;
            pos.y -= size.height / 2;
        }

        GameState::execute_ability_on_target_select(&self.parent, &self.ability, affected, pos);
    }
}
