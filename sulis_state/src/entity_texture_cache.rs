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

use std::ptr;

use crate::{EntityState, GameState};
use sulis_core::config::Config;
use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::Color;
use sulis_core::util::{Offset, Rect, Scale};

const BORDER_SIZE: i32 = 2;
const BORDER_SIZE_F: f32 = BORDER_SIZE as f32;

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct EntityTextureSlot {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    texture_id: &'static str,
    tex_coords: [f32; 8],
    slots_dim: usize,
    slot_size: u32,
}

impl EntityTextureSlot {
    pub fn redraw_entity(&self, entity: &EntityState, renderer: &mut dyn GraphicsRenderer) {
        let scale = self.slot_size as i32;
        renderer.clear_texture_region(
            self.texture_id,
            self.x * scale,
            self.y * scale,
            (self.x + self.w) * scale,
            (self.y + self.h) * scale,
        );

        let (ui_x, ui_y) = Config::ui_size();

        let offset = Offset {
            x: self.x as f32 + BORDER_SIZE_F,
            y: self.y as f32 + 2.0 * BORDER_SIZE_F,
        };
        let scale = Scale {
            x: ui_x as f32 / self.slots_dim as f32,
            y: ui_y as f32 / self.slots_dim as f32,
        };

        entity
            .actor
            .draw_to_texture(renderer, self.texture_id, offset, scale);
    }

    pub fn draw(
        &self,
        renderer: &mut dyn GraphicsRenderer,
        slot: Slot,
        offset: Offset,
        scale: Scale,
        color: Color,
        color_sec: Color,
    ) {
        let rect = Rect {
            x: slot.x - BORDER_SIZE_F - offset.x,
            y: slot.y - 2.0 * BORDER_SIZE_F - offset.y,
            w: self.w as f32 + offset.x * 2.0,
            h: self.h as f32 + offset.y * 2.0,
        };

        let mut list = DrawList::from_texture_id(self.texture_id, &self.tex_coords, rect);

        list.set_scale(scale);
        list.set_color(color);
        list.set_color_sec(color_sec);
        renderer.draw(list);
    }
}

pub struct EntityTextureCache {
    // size: u32,
    slot_size: u32,
    slots_dim: usize,
    texture_id: &'static str,

    slots: Vec<bool>,

    entity_slots: Vec<EntityTextureSlot>,
}

impl EntityTextureCache {
    pub fn new(texture_id: &'static str, size: u32, slot_size: u32) -> EntityTextureCache {
        let slots_dim = (size / slot_size) as usize;
        info!(
            "Creating entity texture cache with {} slots dimension",
            slots_dim
        );

        EntityTextureCache {
            // size,
            slot_size,
            slots_dim,
            texture_id,
            slots: vec![false; slots_dim * slots_dim],
            entity_slots: Vec::new(),
        }
    }

    pub fn invalidate(&mut self) {
        self.entity_slots.clear();

        unsafe {
            ptr::write_bytes(self.slots.as_mut_ptr(), 0, self.slots.len());
        }

        let mgr = GameState::turn_manager();
        for entity in mgr.borrow().entity_iter() {
            entity.borrow_mut().clear_texture_cache();
        }
    }

    /// Adds the specified entity to the cache, finding a slot for the entity and drawing
    /// the entity in that slot.  the returned index is a handle to the slot used by the entity
    pub fn add_entity(
        &mut self,
        entity: &EntityState,
        renderer: &mut dyn GraphicsRenderer,
    ) -> EntityTextureSlot {
        let width = entity.size.width + BORDER_SIZE * 2;
        let height = entity.size.height + BORDER_SIZE * 2;

        let slot_index = self.find_slot(width, height);
        let slot = &self.entity_slots[slot_index];
        let offset = Offset {
            x: slot.x as f32 + BORDER_SIZE_F,
            y: slot.y as f32 + 2.0 * BORDER_SIZE_F,
        };

        info!(
            "Drawing entity '{}' to slot {} at {},{}",
            entity.actor.actor.id, slot_index, offset.x, offset.y,
        );
        let (ui_x, ui_y) = Config::ui_size();
        let scale = Scale {
            x: ui_x as f32 / self.slots_dim as f32,
            y: ui_y as f32 / self.slots_dim as f32,
        };
        entity
            .actor
            .draw_to_texture(renderer, self.texture_id, offset, scale);

        slot.clone()
    }

    fn find_slot(&mut self, width: i32, height: i32) -> usize {
        let size = self.slots_dim * self.slots_dim;
        for i in 0..size {
            if self.check_and_mark_index(i, width, height) {
                let x = (i % self.slots_dim) as i32;
                let y = (i / self.slots_dim) as i32;

                let dim = self.slots_dim as f32;
                let x_min = x as f32 / dim;
                let x_max = (x as f32 + width as f32) / dim;
                let y_min = (dim - height as f32 - y as f32) / dim;
                let y_max = (dim - y as f32) / dim;
                let tex_coords = [x_min, y_max, x_min, y_min, x_max, y_max, x_max, y_min];
                trace!(
                    "Pushing slot with tex coords: {:?}, dims: {},{}",
                    tex_coords,
                    width,
                    height
                );
                self.entity_slots.push(EntityTextureSlot {
                    x,
                    y,
                    w: width,
                    h: height,
                    tex_coords,
                    texture_id: self.texture_id,
                    slots_dim: self.slots_dim,
                    slot_size: self.slot_size,
                });
                return self.entity_slots.len() - 1;
            }
        }

        warn!("Unable to find available slot for entity image");
        0
    }

    fn mark_used(&mut self, x_min: usize, y_min: usize, x_max: usize, y_max: usize) {
        for y in y_min..y_max {
            for x in x_min..x_max {
                let index = x + y * self.slots_dim;
                self.slots[index] = true;
            }
        }
    }

    fn check_and_mark_index(&mut self, index: usize, width: i32, height: i32) -> bool {
        let x_min = index % self.slots_dim;
        let y_min = index / self.slots_dim;
        let x_max = x_min + width as usize;
        let y_max = y_min + height as usize;

        if x_max >= self.slots_dim || y_max >= self.slots_dim {
            return false;
        }

        for y in y_min..y_max {
            for x in x_min..x_max {
                let cur_index = x + y * self.slots_dim;
                if self.slots[cur_index] {
                    return false;
                }
            }
        }

        self.mark_used(x_min, y_min, x_max, y_max);
        true
    }
}
