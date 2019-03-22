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

use std::io::{ErrorKind, Error};
use std::collections::HashMap;
use std::rc::Rc;

use sulis_core::util::{self, ReproducibleRandom, unable_to_create_error};
use sulis_module::Module;
use sulis_module::area::{Area, AreaBuilder, LayerSet, Tile, PathFinderGrid, PropData,
    EncounterData, Transition, TransitionBuilder, GeneratorParams, create_prop};
use sulis_module::generator::AreaGenerator;

pub struct GeneratedArea {
    area: Rc<Area>,
    layer_set: LayerSet,
    path_grid: HashMap<String, PathFinderGrid>,
    props: Vec<PropData>,
    transitions: Vec<Transition>,
    encounters: Vec<EncounterData>,
}

impl GeneratedArea {
    pub fn new(area: Rc<Area>, pregen_out: Option<PregenOutput>) -> Result<GeneratedArea, Error> {
        let mut generated_encounters = Vec::new();
        let mut generated_props = Vec::new();
        let mut layers = Vec::new();

        if let Some(pregen) = pregen_out {
            let start_time = std::time::Instant::now();

            let output = pregen.generator.generate(area.width,
                                                   area.height,
                                                   pregen.rand,
                                                   &pregen.params, pregen.tiles_to_add)?;
            layers = output.layers;
            generated_props = output.props;
            generated_encounters = output.encounters;

            info!("Area generation complete in {} secs",
                  util::format_elapsed_secs(start_time.elapsed()));
        }

        let mut props: Vec<_> = area.props.iter().map(|p| p.clone()).collect();
        for builder in generated_props {
            props.push(create_prop(&builder)?);
        }

        let mut encounters: Vec<_> = area.encounters.iter().map(|e| e.clone()).collect();
        for builder in generated_encounters {
            let encounter = match Module::encounter(&builder.id) {
                None => {
                    warn!("No encounter '{}' found", builder.id);
                    return unable_to_create_error("area", &area.id);
                },
                Some(enc) => enc,
            };
            encounters.push(EncounterData {
                encounter,
                location: builder.location,
                size: builder.size,
                triggers: Vec::new(),
            });
        }

        Ok(GeneratedArea {
            area,
            props,
            encounters,
        })
    }
}

pub struct PregenOutput {
    generator: Rc<AreaGenerator>,
    params: GeneratorParams,
    tiles_to_add: Vec<(Rc<Tile>, i32, i32)>,
    transitions: Vec<TransitionBuilder>,
    rand: ReproducibleRandom,
}

impl PregenOutput {
    pub fn new(builder: &AreaBuilder) -> Result<Option<PregenOutput>, Error> {
        let mut rand = ReproducibleRandom::new(None);
        let start_time = std::time::Instant::now();

        let params = match &builder.generator {
            None => return Ok(None),
            Some(builder) => GeneratorParams::new(builder.clone())?,
        };

        let generator = Module::generator(&params.id).ok_or(
            Error::new(ErrorKind::InvalidInput, format!("Generator '{}' not found", params.id))
        )?;

        let transition_out = generator.generate_transitions(builder, &mut rand, &params)?;

        info!("Area pregen complete in {} secs",
              util::format_elapsed_secs(start_time.elapsed()));

        let mut transitions = Vec::new();
        let mut tiles_out = Vec::new();
        for mut data in transition_out {
            transitions.push(data.transition);
            tiles_out.append(&mut data.tiles);
        }

        Ok(Some(PregenOutput {
            generator,
            params,
            tiles_to_add: tiles_out,
            transitions,
            rand,
        }))
    }
}
