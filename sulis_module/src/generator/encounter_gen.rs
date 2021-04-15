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

use std::collections::HashMap;
use std::io::Error;
use std::rc::Rc;

use crate::generator::{
    maze::Room, overlaps_any, GenModel, Maze, Rect, RegionKind, RegionKinds, WeightedEntry,
    WeightedList,
};
use crate::{
    area::{EncounterDataBuilder, Layer},
    Encounter, Module,
};
use sulis_core::util::{Point, Size};

pub struct EncounterGen<'a, 'b> {
    model: &'b mut GenModel,
    params: &'a EncounterParams,
    maze: &'b Maze,
}

impl<'a, 'b> EncounterGen<'a, 'b> {
    pub(crate) fn new(
        model: &'b mut GenModel,
        _layers: &'b [Layer],
        params: &'a EncounterParams,
        maze: &'b Maze,
    ) -> EncounterGen<'a, 'b> {
        EncounterGen {
            model,
            params,
            maze,
        }
    }

    pub(crate) fn generate(
        &mut self,
        addn_passes: &[EncounterPass],
    ) -> Vec<EncounterDataBuilder> {
        let mut encounters = Vec::new();

        for pass in self.params.passes.iter().chain(addn_passes) {
            for room in self.maze.rooms() {
                let encounter = pass.kinds.pick(&mut self.model.rand);

                if self.model.rand.gen(1, 101) > pass.chance_per_room {
                    continue;
                }

                let data =
                    EncounterData::gen(&mut self.model, encounter, room, pass.size.x, pass.size.y);

                let p1 = Point::from(self.model.to_region_coords(data.x, data.y));
                let p2 = Point::from(
                    self.model
                        .to_region_coords(data.x + data.w, data.y + data.h),
                );

                if !pass.allowable_regions.check_coords(&self.maze, p1, p2) {
                    continue;
                }

                if overlaps_any(&data, &encounters, pass.spacing as i32) {
                    continue;
                }

                encounters.push(data);
            }
        }

        let mut out = Vec::with_capacity(encounters.len());
        for encounter in encounters {
            out.push(EncounterDataBuilder {
                id: encounter.encounter.id.to_string(),
                location: Point::new(encounter.x, encounter.y),
                size: Size::new(encounter.w, encounter.h),
            });
        }
        out
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
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn w(&self) -> i32 {
        self.w
    }
    fn h(&self) -> i32 {
        self.h
    }
}

impl EncounterData {
    fn gen(
        model: &mut GenModel,
        encounter: &Rc<Encounter>,
        room: &Room,
        w: i32,
        h: i32,
    ) -> EncounterData {
        let encounter = Rc::clone(encounter);
        let (min_x, min_y) = model.from_region_coords(room.x, room.y);
        let (max_x, max_y) = model.from_region_coords(room.x + room.width, room.y + room.height);
        let x = model.rand.gen(min_x, max_x - w);
        let y = model.rand.gen(min_y, max_y - h);

        EncounterData {
            encounter,
            x,
            y,
            w,
            h,
        }
    }
}

pub struct EncounterParams {
    pub passes: Vec<EncounterPass>,
}

impl EncounterParams {
    pub(crate) fn with_module(
        builder: EncounterParamsBuilder,
        module: &Module,
    ) -> Result<EncounterParams, Error> {
        EncounterParams::build(builder, |id| {
            module.encounters.get(id).map(|e| Rc::clone(e))
        })
    }

    pub(crate) fn new(builder: EncounterParamsBuilder) -> Result<EncounterParams, Error> {
        EncounterParams::build(builder, |id| Module::encounter(id))
    }

    fn build<F>(builder: EncounterParamsBuilder, f: F) -> Result<EncounterParams, Error>
    where
        F: Fn(&str) -> Option<Rc<Encounter>>,
    {
        let mut passes = Vec::new();

        for pass in builder.passes {
            let kinds = WeightedList::new(pass.kinds, "Encounter", &f)?;
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

pub struct EncounterPass {
    kinds: WeightedList<Rc<Encounter>>,
    spacing: u32,
    chance_per_room: u32,
    allowable_regions: RegionKinds,
    size: Point,
}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct EncounterParamsBuilder {
    passes: Vec<EncounterPassBuilder>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EncounterPassBuilder {
    kinds: HashMap<String, WeightedEntry>,
    spacing: u32,
    chance_per_room: u32,
    allowable_regions: Vec<RegionKind>,
    size: (u32, u32),
}
