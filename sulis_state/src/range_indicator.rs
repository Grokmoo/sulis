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

use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use sulis_core::image::Image;
use sulis_core::io::DrawList;
use sulis_core::resource::ResourceSet;
use sulis_core::ui::animation_state;
use sulis_core::util::Point;

use crate::{is_within, EntityState, GameState};
use sulis_module::{ability::Range, Ability};

const NW: u8 = 1;
const N: u8 = 2;
const NE: u8 = 4;
const E: u8 = 8;
const SE: u8 = 16;
const S: u8 = 32;
const SW: u8 = 64;
const W: u8 = 128;

pub struct RangeIndicatorHandler {
    indicators: Vec<RangeIndicator>,
}

impl Default for RangeIndicatorHandler {
    fn default() -> Self {
        Self {
            indicators: Vec::default(),
        }
    }
}

impl RangeIndicatorHandler {
    pub fn current(&self) -> Option<&RangeIndicator> {
        self.indicators.first()
    }

    pub fn add_attack(&mut self, parent: &Rc<RefCell<EntityState>>) {
        let indicator = RangeIndicator::attack(parent);
        self.add(Some(indicator));
    }

    pub fn add(&mut self, indicator: Option<RangeIndicator>) {
        let indicator = match indicator {
            None => return,
            Some(ind) => ind,
        };

        self.indicators.push(indicator);
        self.indicators.sort_by(|a, b| a.kind.cmp(&b.kind));
    }

    pub fn clear(&mut self) {
        self.indicators.clear();
    }

    pub fn remove_ability(&mut self, ability: &Rc<Ability>) {
        self.indicators.retain(|ind| match &ind.kind {
            Kind::Ability(other) => !Rc::ptr_eq(ability, other),
            _ => true,
        });
    }

    pub fn remove_targeter(&mut self) {
        self.indicators.retain(|ind| match &ind.kind {
            Kind::Targeter => false,
            _ => true,
        });
    }

    pub fn remove_attack(&mut self) {
        self.indicators.retain(|ind| match &ind.kind {
            Kind::Attack => false,
            _ => true,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Kind {
    Ability(Rc<Ability>),
    Targeter,
    Attack,
}

impl PartialOrd for Kind {
    fn partial_cmp(&self, other: &Kind) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Kind {
    fn cmp(&self, other: &Kind) -> Ordering {
        use Kind::*;
        match self {
            Ability(self_ability) => match other {
                Ability(other_ability) => self_ability.id.cmp(&other_ability.id),
                _ => Ordering::Less,
            },
            Targeter => match other {
                Ability(..) => Ordering::Greater,
                Targeter => Ordering::Equal,
                Attack => Ordering::Less,
            },
            Attack => match other {
                Attack => Ordering::Equal,
                _ => Ordering::Greater,
            },
        }
    }
}

#[derive(Clone)]
pub struct RangeIndicator {
    kind: Kind,
    parent: Rc<RefCell<EntityState>>,
    neighbors: Vec<u8>,
    half_width: i32,
}

impl RangeIndicator {
    /// Creates an ability range indicator.  will panic if ability is not active.
    pub fn ability(parent: &Rc<RefCell<EntityState>>, ability: &Rc<Ability>) -> RangeIndicator {
        let active = ability.active.as_ref().unwrap();

        let mut radius = match active.range {
            Range::None => 0.0,
            Range::Personal => 0.0,
            Range::Radius(r) => r,
            Range::Touch => parent.borrow().actor.stats.touch_distance(),
            Range::Attack => parent.borrow().actor.stats.attack_distance(),
            Range::Visible => {
                let area = GameState::area_state();
                let area = &area.borrow().area.area;
                area.vis_dist as f32 - 1.0
            }
        };

        let level = parent
            .borrow()
            .actor
            .actor
            .ability_level(&ability.id)
            .unwrap_or(0);
        for (index, upgrade) in ability.upgrades.iter().enumerate() {
            if index as u32 >= level {
                break;
            }

            radius += upgrade.range_increase;
        }

        if let Some(increase) = &active.range_increases_with {
            if let Some(level) = parent.borrow().actor.actor.ability_level(&increase.ability) {
                radius += (level + 1) as f32 * increase.amount;
            }
        }

        let ability = Rc::clone(ability);
        RangeIndicator::new(Kind::Ability(ability), radius, parent)
    }

    pub fn targeter(radius: f32, parent: &Rc<RefCell<EntityState>>) -> RangeIndicator {
        RangeIndicator::new(Kind::Targeter, radius, parent)
    }

    pub fn attack(parent: &Rc<RefCell<EntityState>>) -> RangeIndicator {
        let radius = parent.borrow().actor.stats.attack_distance();
        RangeIndicator::new(Kind::Attack, radius, parent)
    }

    fn new(kind: Kind, radius: f32, parent: &Rc<RefCell<EntityState>>) -> RangeIndicator {
        let parent = Rc::clone(parent);

        let half_width = radius.ceil() as i32 + 5;
        let width = (half_width * 2) as usize;

        let points = if radius == 0.0 {
            personal_points(&parent.borrow(), half_width, width)
        } else {
            compute_points(&parent.borrow(), radius, half_width, width)
        };

        let mut neighbors = vec![0; width * width];
        for y in 0..width {
            for x in 0..width {
                neighbors[x + y * width] = find_neighbors(width, &points, x, y);
            }
        }

        // warn!("RangeIndicator Points:");
        // for y in 0..width {
        //     let mut line = String::new();
        //     for x in 0..width {
        //         if points[x + y * width] {
        //             line.push('*');
        //         } else {
        //             line.push(' ');
        //         }
        //     }
        //     warn!("  {}", line);
        // }
        //
        // warn!("Neighbors:");
        // for y in 0..width {
        //     let mut line = String::new();
        //     for x in 0..width {
        //       line.push_str(&format!("{:<3x?}", neighbors[x + y * width]));
        //     }
        //     warn!("  {}", line);
        // }

        RangeIndicator {
            neighbors,
            half_width,
            parent,
            kind,
        }
    }

    pub fn get_draw_list(
        &self,
        image_set: &RangeIndicatorImageSet,
        x_offset: f32,
        y_offset: f32,
        millis: u32,
    ) -> DrawList {
        let x_offset = x_offset - self.parent.borrow().location.x as f32;
        let y_offset = y_offset - self.parent.borrow().location.y as f32;
        let mut draw_list = DrawList::empty_sprite();

        let half_width_f32 = self.half_width as f32;
        let width = (self.half_width * 2) as usize;
        for y in 0..width {
            for x in 0..width {
                let n = self.neighbors[x + y * width];

                if let Some(ref image) = image_set.images[n as usize] {
                    image.append_to_draw_list(
                        &mut draw_list,
                        &animation_state::NORMAL,
                        x as f32 - x_offset - half_width_f32,
                        y as f32 - y_offset - half_width_f32,
                        1.0,
                        1.0,
                        millis,
                    );
                }
            }
        }

        draw_list
    }
}

fn personal_points(parent: &EntityState, half_width: i32, width: usize) -> Vec<bool> {
    let mut points = vec![false; width * width];

    for p in parent.location_points() {
        let x = (p.x - parent.location.x + half_width) as usize;
        let y = (p.y - parent.location.y + half_width) as usize;

        let idx = x + y * width;
        points[idx] = true;
    }

    points
}

fn compute_points(parent: &EntityState, radius: f32, half_width: i32, width: usize) -> Vec<bool> {
    let mut points = vec![true; width * width];

    for y in 0..width {
        for x in 0..width {
            let (x1, y1) = (
                x as i32 + parent.location.x - half_width,
                y as i32 + parent.location.y - half_width,
            );
            let p = Point::new(x1, y1);

            let idx = x + y * width;
            points[idx] = !is_within(parent, &p, radius);
        }
    }

    points
}

fn find_neighbors(width: usize, points: &[bool], x: usize, y: usize) -> u8 {
    let mut total = 0;

    let (x, y, w) = (x as i32, y as i32, width as i32);

    if check_idx(w, points, x, y) {
        return 255;
    }

    if check_idx(w, points, x - 1, y) {
        total += W;
    }
    if check_idx(w, points, x - 1, y - 1) {
        total += NW;
    }
    if check_idx(w, points, x, y - 1) {
        total += N;
    }
    if check_idx(w, points, x + 1, y - 1) {
        total += NE;
    }
    if check_idx(w, points, x + 1, y) {
        total += E;
    }
    if check_idx(w, points, x + 1, y + 1) {
        total += SE;
    }
    if check_idx(w, points, x, y + 1) {
        total += S;
    }
    if check_idx(w, points, x - 1, y + 1) {
        total += SW;
    }

    total
}

fn check_idx(width: i32, points: &[bool], x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= width || y >= width {
        return false;
    }

    points[x as usize + y as usize * width as usize]
}

pub struct RangeIndicatorImageSet {
    images: Vec<Option<Rc<dyn Image>>>,
}

impl RangeIndicatorImageSet {
    pub fn new(prefix: String) -> RangeIndicatorImageSet {
        let rules = [
            ("outer_nw", W + NW + N),
            ("outer_n", NW + N + NE),
            ("outer_ne", N + NE + E),
            ("outer_e", NE + E + SE),
            ("outer_se", E + SE + S),
            ("outer_s", SE + S + SW),
            ("outer_sw", S + SW + W),
            ("outer_w", SW + W + NW),
            ("outer_se", SW + W + NW + N + NE),
            ("outer_sw", NW + N + NE + E + SE),
            ("outer_nw", NE + E + SE + S + SW),
            ("outer_ne", SE + S + SW + W + NW),
            ("outer_nw", SW + W + NW + N),
            ("outer_ne", SE + N + NE + E),
            ("outer_se", NE + E + SE + S),
            ("outer_sw", NW + S + SW + W),
            ("outer_n", N + NE),
            ("outer_n", N + NW),
            ("outer_s", S + SE),
            ("outer_s", S + SW),
            ("outer_e", E + NE),
            ("outer_e", E + SE),
            ("outer_w", W + NW),
            ("outer_w", W + SW),
            ("outer_ne", NW + N + NE + E),
            ("outer_nw", NW + N + NE + W),
            ("outer_se", E + SE + S + SW),
            ("outer_sw", S + SW + W + SE),
            ("inner_nw", SE),
            ("inner_ne", SW),
            ("inner_sw", NE),
            ("inner_se", NW),
            ("outer_n", NE + E + SE + S + SW + W + NW),
            ("outer_s", SW + W + NW + N + NE + E + SE),
            ("outer_e", SE + S + SW + W + NW + N + NE),
            ("outer_w", NW + N + NE + E + SE + S + SW),
            ("outer_nw", NW + N + NE + SW + W),
            ("outer_ne", NW + N + NE + E + SE),
            ("outer_sw", NW + SE + S + SW + W),
            ("outer_se", NE + E + SE + S + SW),
            ("center", 0),
        ];

        let mut images = vec![None; 256];
        for rule in rules.iter() {
            let rule = match ImageRule::new(&prefix, rule.0, rule.1) {
                None => continue,
                Some(rule) => rule,
            };

            images[rule.neighbors as usize] = Some(rule.image);
        }

        RangeIndicatorImageSet { images }
    }
}

struct ImageRule {
    image: Rc<dyn Image>,
    neighbors: u8,
}

impl ImageRule {
    fn new(prefix: &str, postfix: &str, neighbors: u8) -> Option<ImageRule> {
        let id = format!("{}{}", prefix, postfix);
        let image = match ResourceSet::image(&id) {
            None => {
                warn!("No image found for selection area {}", id);
                return None;
            }
            Some(image) => image,
        };

        Some(ImageRule { image, neighbors })
    }
}
