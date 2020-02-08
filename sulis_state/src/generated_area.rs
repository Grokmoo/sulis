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

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::rc::Rc;

use sulis_core::resource::ResourceSet;
use sulis_core::util::{self, unable_to_create_error, ReproducibleRandom};
use sulis_module::area::{
    create_prop, Area, EncounterData, LayerSet, PathFinderGrid, PropData, Tile, Transition,
    TransitionBuilder,
};
use sulis_module::generator::AreaGenerator;
use sulis_module::Module;

pub struct GeneratedArea {
    pub area: Rc<Area>,
    pub width: i32,
    pub height: i32,
    pub layer_set: LayerSet,
    path_grids: HashMap<String, PathFinderGrid>,
    pub props: Vec<PropData>,
    pub transitions: Vec<Transition>,
    pub encounters: Vec<EncounterData>,
}

impl GeneratedArea {
    pub fn new(area: Rc<Area>, pregen_out: Option<PregenOutput>) -> Result<GeneratedArea, Error> {
        let mut generated_encounters = Vec::new();
        let mut generated_props = Vec::new();
        let mut layers = Vec::new();
        let mut transition_builders = Vec::new();

        for transition in area.builder.transitions.iter() {
            transition_builders.push((*transition).clone());
        }

        if let Some(pregen) = pregen_out {
            let start_time = std::time::Instant::now();

            for transition in pregen.transitions {
                transition_builders.push(transition);
            }

            let params = area.generator.as_ref().unwrap();

            let output = pregen.generator.generate(
                area.width,
                area.height,
                pregen.rand,
                params,
                &transition_builders,
                pregen.tiles_to_add,
            )?;
            layers = output.layers;
            generated_props = output.props;
            generated_encounters = output.encounters;

            info!(
                "Area generation complete in {} secs",
                util::format_elapsed_secs(start_time.elapsed())
            );
        }

        let mut props: Vec<_> = area.props.to_vec();
        for builder in generated_props {
            props.push(create_prop(&builder)?);
        }

        let mut encounters: Vec<_> = area.encounters.to_vec();
        for builder in generated_encounters {
            let encounter = match Module::encounter(&builder.id) {
                None => {
                    warn!("No encounter '{}' found", builder.id);
                    return unable_to_create_error("area", &area.id);
                }
                Some(enc) => enc,
            };
            encounters.push(EncounterData {
                encounter,
                location: builder.location,
                size: builder.size,
                triggers: Vec::new(),
            });
        }

        let layer_set = LayerSet::new(&area.builder, &props, layers)?;

        let mut path_grids = HashMap::new();
        for size in Module::all_sizes() {
            let path_grid = PathFinderGrid::new(
                Rc::clone(&size),
                layer_set.width,
                layer_set.height,
                &layer_set.passable,
            );
            path_grids.insert(size.id.to_string(), path_grid);
        }

        let mut transitions = Vec::new();
        for (index, t_builder) in transition_builders.into_iter().enumerate() {
            let img_id = &t_builder.image_display;
            let image = ResourceSet::image(img_id).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("No image '{}' found", img_id),
                )
            })?;

            let size = Module::size(&t_builder.size).ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("No size '{}' found", t_builder.size),
                )
            })?;

            let p = t_builder.from;
            if !p.in_bounds(area.width, area.height) {
                warn!("Transition {} falls outside area bounds", index);
                continue;
            }

            p.add(size.width, size.height);
            if !p.in_bounds(area.width, area.height) {
                warn!("Transition {} falls outside area bounds", index);
                continue;
            }

            let transition = Transition {
                from: t_builder.from,
                to: t_builder.to.clone(),
                hover_text: t_builder.hover_text.clone(),
                size,
                image_display: image,
            };
            transitions.push(transition);
        }

        info!("{} total transitions created", transitions.len());

        let (width, height) = (area.width, area.height);

        Ok(GeneratedArea {
            area,
            width,
            height,
            props,
            encounters,
            layer_set,
            path_grids,
            transitions,
        })
    }

    pub fn path_grid(&self, size_id: &str) -> &PathFinderGrid {
        &self.path_grids[size_id]
    }
}

pub struct PregenOutput {
    generator: Rc<AreaGenerator>,
    pub tiles_to_add: Vec<(Rc<Tile>, i32, i32)>,
    pub transitions: Vec<TransitionBuilder>,
    rand: ReproducibleRandom,
}

impl PregenOutput {
    pub fn new(area: &Area, seed: Option<u128>) -> Result<Option<PregenOutput>, Error> {
        let mut rand = ReproducibleRandom::new(seed);
        let start_time = std::time::Instant::now();

        let params = match &area.generator {
            None => return Ok(None),
            Some(params) => params,
        };

        let generator = Module::generator(&params.id).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidInput,
                format!("Generator '{}' not found", params.id),
            )
        })?;

        let (w, h) = (area.width, area.height);
        let transition_out = generator.generate_transitions(w, h, &mut rand, &params)?;

        info!(
            "Area pregen complete in {} secs",
            util::format_elapsed_secs(start_time.elapsed())
        );

        let mut transitions = Vec::new();
        let mut tiles_out = Vec::new();
        for mut data in transition_out {
            transitions.push(data.transition);
            tiles_out.append(&mut data.tiles);
        }

        Ok(Some(PregenOutput {
            generator,
            tiles_to_add: tiles_out,
            transitions,
            rand,
        }))
    }

    pub fn seed(&self) -> u128 {
        self.rand.seed()
    }
}
