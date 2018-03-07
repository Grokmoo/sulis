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
}

impl fmt::Debug for PropState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Prop: {:?}", self.prop)?;
        write!(f, "Location: {:?}", self.location)
    }
}

impl PropState {
    pub(crate) fn new(prop_data: &PropData, location: Location) -> PropState {
        let mut items = Vec::new();
        for item in prop_data.items.iter() {
            items.push(ItemState::new(Rc::clone(item)));
        }

        PropState {
            prop: Rc::clone(&prop_data.prop),
            location,
            animation_state: AnimationState::default(),
            items,
            listeners: ChangeListenerList::default(),
        }
    }

    pub fn toggle_active(&mut self) {
        let kind = animation_state::Kind::Active;

        self.animation_state.toggle(kind);
    }

    pub fn is_active(&self) -> bool {
        self.animation_state.contains(animation_state::Kind::Active)
    }

    /// Removes an item from this Prop at the specified index and returns it
    pub fn remove_item(&mut self, index: usize) -> ItemState {
        let item = self.items.remove(index);
        self.listeners.notify(&self);
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
