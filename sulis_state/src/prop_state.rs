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

use std::fmt;
use std::rc::Rc;

use sulis_core::io::{DrawList, GraphicsRenderer};
use sulis_core::ui::{animation_state, AnimationState};
use sulis_module::Prop;
use sulis_module::area::PropData;
use {ChangeListenerList, ItemState, Location};
use entity_state::AreaDrawable;

pub struct PropState {
    pub prop: Rc<Prop>,
    pub location: Location,
    pub animation_state: AnimationState,
    pub items: Vec<ItemState>,
    pub listeners: ChangeListenerList<PropState>,
    can_generate_loot: bool,

    temporary: bool,
    marked_for_removal: bool,
}

impl fmt::Debug for PropState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Prop: {:?}", self.prop)?;
        write!(f, "Location: {:?}", self.location)
    }
}

impl PropState {
    pub(crate) fn new(prop_data: &PropData, location: Location, temporary: bool) -> PropState {
        let mut items = Vec::new();
        for item in prop_data.items.iter() {
            items.push(ItemState::new(Rc::clone(item)));
        }

        let can_generate_loot = prop_data.prop.loot.is_some();

        PropState {
            prop: Rc::clone(&prop_data.prop),
            location,
            animation_state: AnimationState::default(),
            items,
            listeners: ChangeListenerList::default(),
            can_generate_loot,
            temporary,
            marked_for_removal: false,
        }
    }

    pub (crate) fn is_marked_for_removal(&self) -> bool {
        self.marked_for_removal
    }

    pub fn toggle_active(&mut self) {
        if !self.is_active() && self.can_generate_loot {
            if let Some(ref loot) = self.prop.loot {
                let items = loot.generate();
                for item in items {
                    self.items.push(ItemState::new(item));
                }
            }

            self.can_generate_loot = false;
        }

        self.animation_state.toggle(animation_state::Kind::Active);
    }

    pub fn is_active(&self) -> bool {
        self.animation_state.contains(animation_state::Kind::Active)
    }

    /// Removes an item from this Prop at the specified index and returns it
    pub fn remove_item(&mut self, index: usize) -> ItemState {
        let item = self.items.remove(index);
        self.listeners.notify(&self);

        if self.temporary && self.items.is_empty() {
            self.marked_for_removal = true;
        }

        item
    }

    pub fn num_items(&self) -> usize {
        self.items.len()
    }

    pub fn append_to_draw_list(&self, draw_list: &mut DrawList, x: f32, y: f32, millis: u32) {
        self.prop.append_to_draw_list(draw_list, &self.animation_state, x, y, millis);
    }
}

impl AreaDrawable for PropState {
    fn draw(&self, renderer: &mut GraphicsRenderer, scale_x: f32, scale_y: f32, x: f32, y: f32,
                millis: u32) {
        let x = x + self.location.x as f32;
        let y = y + self.location.y as f32;

        let mut draw_list = DrawList::empty_sprite();
        draw_list.set_scale(scale_x, scale_y);
        self.append_to_draw_list(&mut draw_list, x, y, millis);
        renderer.draw(draw_list);
    }

    fn location(&self) -> &Location {
        &self.location
    }
}
