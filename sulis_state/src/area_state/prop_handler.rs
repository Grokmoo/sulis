//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2019 Jared Stephen
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

use std::io::Error;
use std::rc::Rc;

use crate::{prop_state, save_state::PropSaveState, Location, PropState};
use sulis_core::util::{invalid_data_error, Point};
use sulis_module::{area::PropData, prop::Interactive, Area, Module, Prop};

pub struct PropHandler {
    area: Rc<Area>,
    props: Vec<Option<PropState>>,
    prop_grid: Vec<Vec<usize>>,

    prop_vis_grid: Vec<bool>,
    prop_pass_grid: Vec<bool>,
}

impl PropHandler {
    pub fn new(dim: usize, area: &Rc<Area>) -> PropHandler {
        PropHandler {
            props: Vec::new(),
            prop_grid: vec![Vec::new(); dim],
            prop_vis_grid: vec![true; dim],
            prop_pass_grid: vec![true; dim],
            area: Rc::clone(area),
        }
    }

    pub fn load(&mut self, props: Vec<PropSaveState>) -> Result<(), Error> {
        for prop in props {
            self.load_prop(prop)?;
        }

        Ok(())
    }

    pub fn populate(&mut self, props: &[PropData]) {
        for data in props {
            let location = Location::from_point(data.location, &self.area);
            if let Err(e) = self.add(data, location, false) {
                warn!("Unable to add prop at {:?}", &data.location);
                warn!("{}", e);
            }
        }
    }

    fn load_prop(&mut self, data: PropSaveState) -> Result<(), Error> {
        let prop = match Module::prop(&data.id) {
            None => invalid_data_error(&format!("No prop with ID '{}'", data.id)),
            Some(prop) => Ok(prop),
        }?;

        let location = Location::from_point(data.location, &self.area);

        let prop_data = PropData {
            prop,
            location: data.location,
            items: Vec::new(),
            enabled: data.enabled,
            hover_text: None,
        };

        let index = self.add(&prop_data, location, false)?;
        self.props[index]
            .as_mut()
            .unwrap()
            .load_interactive(data.interactive)?;

        self.update_vis_pass_grid(index);
        Ok(())
    }

    pub fn index_valid(&self, index: usize) -> bool {
        if index >= self.props.len() {
            return false;
        }

        self.props[index].is_some()
    }

    fn container_index_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.area.coords_valid(x, y) {
            return None;
        }

        let index = (x + y * self.area.width) as usize;
        for prop_index in &self.prop_grid[index] {
            use prop_state::Interactive::*;
            match self.props[*prop_index].as_ref().unwrap().interactive {
                Not | Door { .. } | Hover { .. } => (),
                Container { .. } => return Some(*prop_index),
            }
        }
        None
    }

    pub fn remove_matching(&mut self, x: i32, y: i32, name: &str) {
        let mut match_idx = None;
        for (index, prop) in self.props.iter().enumerate() {
            if let Some(prop) = prop {
                if let prop_state::Interactive::Hover { ref text } = prop.interactive {
                    if text == name && prop.location.x == x && prop.location.y == y {
                        match_idx = Some(index);
                        break;
                    }
                }
            }
        }

        if let Some(index) = match_idx {
            self.remove(index);
        }
    }

    pub fn remove(&mut self, index: usize) {
        let prop = match self.props[index] {
            None => return,
            Some(ref prop) => prop,
        };

        trace!("Removing prop '{}'", prop.prop.id);

        let start_x = prop.location.x as usize;
        let start_y = prop.location.y as usize;
        let end_x = start_x + prop.prop.size.width as usize;
        let end_y = start_y + prop.prop.size.height as usize;

        for y in start_y..end_y {
            for x in start_x..end_x {
                self.prop_grid[x + y * self.area.width as usize].retain(|i| *i != index);
            }
        }

        self.props[index] = None;
    }

    #[must_use]
    pub fn check_or_create_container(&mut self, x: i32, y: i32) -> Option<usize> {
        if let Some(idx) = self.container_index_at(x, y) {
            return Some(idx);
        }

        let prop = match Module::prop(&Module::rules().loot_drop_prop) {
            None => {
                warn!("Unable to generate loot drop prop.");
                return None;
            }
            Some(prop) => prop,
        };

        let location = Location::new(x, y, &self.area);
        let data = PropData {
            prop,
            enabled: true,
            location: location.to_point(),
            items: Vec::new(),
            hover_text: None,
        };

        match self.add(&data, location, true) {
            Err(e) => {
                warn!("Unable to add temp container at {},{}", x, y);
                warn!("{}", e);
                None
            }
            Ok(idx) => Some(idx),
        }
    }

    pub fn add_at(
        &mut self,
        prop: &Rc<Prop>,
        x: i32,
        y: i32,
        enabled: bool,
        hover_text: Option<String>,
    ) {
        let location = Location::new(x, y, &self.area);
        let data = PropData {
            prop: Rc::clone(prop),
            enabled,
            location: Point::new(x, y),
            items: Vec::new(),
            hover_text,
        };

        if let Err(e) = self.add(&data, location, true) {
            warn!("Unable to add prop at {},{}", x, y);
            warn!("{}", e);
        }
    }

    pub fn add(
        &mut self,
        prop_data: &PropData,
        location: Location,
        temporary: bool,
    ) -> Result<usize, Error> {
        let prop = &prop_data.prop;

        if !self.area.coords_valid(location.x, location.y) {
            return invalid_data_error(&format!(
                "Prop location: {},{} in {} invalid",
                location.x, location.y, location.area_id
            ));
        }

        if !self
            .area
            .coords_valid(location.x + prop.size.width, location.y + prop.size.height)
        {
            return invalid_data_error(&format!(
                "Prop location: {},{} in {} invalid due to prop size",
                location.x, location.y, location.area_id
            ));
        }

        let state = PropState::new(prop_data, location, temporary);

        let start_x = state.location.x as usize;
        let start_y = state.location.y as usize;
        let end_x = start_x + state.prop.size.width as usize;
        let end_y = start_y + state.prop.size.height as usize;

        let index = self.find_index_to_add();

        for y in start_y..end_y {
            for x in start_x..end_x {
                self.prop_grid[x + y * self.area.width as usize].push(index);
            }
        }

        self.props[index] = Some(state);
        self.update_vis_pass_grid(index);

        Ok(index)
    }

    pub fn update(&mut self) {
        let len = self.len();
        for index in 0..len {
            let prop = match self.props[index] {
                None => continue,
                Some(ref prop) => prop,
            };

            if !prop.is_marked_for_removal() {
                continue;
            }

            self.remove(index);
        }
    }

    pub fn grid(&self) -> &[Vec<usize>] {
        &self.prop_grid
    }

    pub fn entire_vis_grid(&self) -> &[bool] {
        &self.prop_vis_grid
    }

    pub fn entire_pass_grid(&self) -> &[bool] {
        &self.prop_pass_grid
    }

    pub fn vis_grid(&self, index: usize) -> bool {
        self.prop_vis_grid[index]
    }

    pub fn pass_grid(&self, index: usize) -> bool {
        self.prop_pass_grid[index]
    }

    pub fn iter(&self) -> PropIterator {
        PropIterator {
            handler: self,
            index: 0,
        }
    }

    pub fn index_at(&self, x: i32, y: i32) -> Option<usize> {
        if !self.area.coords_valid(x, y) {
            return None;
        }

        let index = (x + y * self.area.width) as usize;
        self.prop_grid[index].get(0).copied()
    }

    pub fn get(&self, index: usize) -> &PropState {
        self.props[index].as_ref().unwrap()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut PropState {
        let prop = self.props[index].as_mut();
        prop.unwrap()
    }

    pub fn len(&self) -> usize {
        self.props.len()
    }

    pub fn get_mut_at(&mut self, x: i32, y: i32) -> Option<&mut PropState> {
        let index = match self.index_at(x, y) {
            None => return None,
            Some(index) => index,
        };

        Some(self.get_mut(index))
    }

    pub fn get_at(&self, x: i32, y: i32) -> Option<&PropState> {
        let index = match self.index_at(x, y) {
            None => return None,
            Some(index) => index,
        };

        Some(self.get(index))
    }

    pub fn set_enabled_at(&mut self, x: i32, y: i32, enabled: bool) -> bool {
        if !self.area.coords_valid(x, y) {
            return false;
        }

        let mut result = false;
        let index = (x + y * self.area.width) as usize;
        for prop_index in &self.prop_grid[index] {
            let prop = self.props[*prop_index].as_mut().unwrap();
            prop.set_enabled(enabled);
            result = true;
        }

        result
    }

    // This method must be called by the owning AreaState in order
    // to compute visibility correctly
    pub(in crate::area_state) fn toggle_active(&mut self, index: usize) -> bool {
        let state = self.get_mut(index);
        state.toggle_active();
        if !state.is_door() {
            return false;
        }

        self.update_vis_pass_grid(index);

        true
    }

    fn find_index_to_add(&mut self) -> usize {
        for (index, item) in self.props.iter().enumerate() {
            if item.is_none() {
                return index;
            }
        }

        self.props.push(None);
        self.props.len() - 1
    }

    fn update_vis_pass_grid(&mut self, index: usize) {
        let prop = self.props[index].as_mut();
        let state = prop.unwrap();

        if !state.is_door() {
            return;
        }

        let width = self.area.width;
        let start_x = state.location.x;
        let start_y = state.location.y;
        let end_x = start_x + state.prop.size.width;
        let end_y = start_y + state.prop.size.height;

        if state.is_active() {
            for y in start_y..end_y {
                for x in start_x..end_x {
                    let idx = (x + y * width) as usize;
                    self.prop_vis_grid[idx] = true;
                    self.prop_pass_grid[idx] = true;
                }
            }
        } else if let Interactive::Door {
            ref closed_invis,
            ref closed_impass,
            ..
        } = state.prop.interactive
        {
            for p in closed_invis {
                self.prop_vis_grid[(p.x + start_x + (p.y + start_y) * width) as usize] = false;
            }

            for p in closed_impass {
                self.prop_pass_grid[(p.x + start_x + (p.y + start_y) * width) as usize] = false;
            }
        }
    }
}

pub struct PropIterator<'a> {
    handler: &'a PropHandler,
    index: usize,
}

impl<'a> Iterator for PropIterator<'a> {
    type Item = &'a PropState;

    fn next(&mut self) -> Option<&'a PropState> {
        loop {
            let next = self.handler.props.get(self.index);
            self.index += 1;

            match next {
                None => return None,
                Some(prop) => match prop {
                    None => continue,
                    Some(ref prop) => return Some(prop),
                },
            }
        }
    }
}
