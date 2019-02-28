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

use std::rc::Rc;
use std::io::Error;
use std::collections::HashMap;

use sulis_core::util::{gen_rand, Point, Size};
use crate::{Module, Encounter, area::{Layer, EncounterDataBuilder}};
use crate::generator::{GenModel, WeightedEntry, WeightedList, Maze, RegionKind, RegionKinds,
    maze::Room, Rect, overlaps_any};

pub struct EncounterGen<'a, 'b> {
    model: &'b mut GenModel<'a>,
    params: &'a EncounterParams,
    maze: &'b Maze,
}

impl<'a, 'b> EncounterGen<'a, 'b> {
    pub(crate) fn new(model: &'b mut GenModel<'a>,
                      _layers: &'b [Layer],
                      params: &'a EncounterParams,
                      maze: &'b Maze) -> EncounterGen<'a, 'b> {
        EncounterGen {
            model,
            params,
            maze,
        }
    }

    pub(crate) fn generate(&mut self) -> Result<Vec<EncounterDataBuilder>, Error> {
        let mut encounters = Vec::new();

        for pass in self.params.passes.iter() {
            for room in self.maze.rooms() {
                let encounter = pass.kinds.pick();

                if gen_rand(1, 101) > pass.chance_per_room { continue; }


                let data = EncounterData::gen(&self.model, encounter, room,
                                              pass.size.x, pass.size.y);

                let p1 = Point::from(self.model.to_region_coords(data.x, data.y));
                let p2 = Point::from(self.model.to_region_coords(data.x + data.w, data.y + data.h));

                if !pass.allowable_regions.check_coords(&self.maze, p1, p2) { continue; }

                if overlaps_any(&data, &encounters, pass.spacing as i32) { continue; }

                encounters.push(data);
            }
        }

        let mut out = Vec::new();
        for encounter in encounters {
            out.push(EncounterDataBuilder {
                id: encounter.encounter.id.to_string(),
                location: Point::new(encounter.x, encounter.y),
                size: Size::new(encounter.w, encounter.h),
            });
        }
        Ok(out)
    }
}

struct EncounterData {
    encounter: Rc<Encounter>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Rect for EncounterData {
    fn x(&self) -> i32 { self.x }
    fn y(&self) -> i32 { self.y }
    fn w(&self) -> i32 { self.w }
    fn h(&self) -> i32 { self.h }
}

impl EncounterData {
    fn gen(model: &GenModel, encounter: &Rc<Encounter>,
           room: &Room, w: i32, h: i32) -> EncounterData {
        let encounter = Rc::clone(encounter);
        let (min_x, min_y) = model.from_region_coords(room.x, room.y);
        let (max_x, max_y) = model.from_region_coords(room.x + room.width, room.y + room.height);
        let x = gen_rand(min_x, max_x - w);
        let y = gen_rand(min_y, max_y - h);

        EncounterData { encounter, x, y, w, h }
    }
}

pub(crate) struct EncounterParams {
    passes: Vec<EncounterPass>,
}

impl EncounterParams {
    pub(crate) fn new(builder: EncounterParamsBuilder,
                      module: &Module) -> Result<EncounterParams, Error> {
        let mut passes = Vec::new();

        for pass in builder.passes {
            let kinds = WeightedList::new(pass.kinds, "Encounter",
                                          |id| module.encounters.get(id).map(|p| Rc::clone(p)))?;
            let regions = RegionKinds::new(pass.allowable_regions);

            passes.push(EncounterPass {
                kinds,
                spacing: pass.spacing,
                chance_per_room: pass.chance_per_room,
                allowable_regions: regions,
                size: Point::new(pass.size.0 as i32, pass.size.1 as i32),
            });
        }
        Ok(EncounterParams { passes })
    }
}

pub(crate) struct EncounterPass {
    kinds: WeightedList<Rc<Encounter>>,
    spacing: u32,
    chance_per_room: u32,
    allowable_regions: RegionKinds,
    size: Point,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EncounterParamsBuilder {
    passes: Vec<EncounterPassBuilder>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EncounterPassBuilder {
    kinds: HashMap<String, WeightedEntry>,
    spacing: u32,
    chance_per_room: u32,
    allowable_regions: Vec<RegionKind>,
    size: (u32, u32),
}
