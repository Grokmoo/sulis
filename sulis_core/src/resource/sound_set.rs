//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2020 Jared Stephen
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

use std::io::{Error, ErrorKind};
use std::fs::File;
use std::rc::Rc;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::io::SoundSource;

pub struct SoundSet {
    id: String,
    sounds: HashMap<String, SoundSource>,
}

impl SoundSet {
    pub fn new(builder: SoundSetBuilder) -> Result<Rc<SoundSet>, Error> {
        let mut sounds = HashMap::new();

        for (id, entry_builder) in &builder.sounds {
            let mut source = None;
            for dir in builder.source_dirs.iter().rev() {
                let mut filepath = PathBuf::from(dir);
                filepath.push(entry_builder.file.to_string());

                let file = match File::open(filepath) {
                    Ok(file) => file,
                    Err(_) => continue,
                };

                let s_id = format!("{}/{}", builder.id, id);
                if let Ok(sound_source) = SoundSource::new(s_id, file, entry_builder) {

                    source = Some(sound_source);
                    break;
                }
            }

            let source = source.ok_or_else(|| {
                warn!("Unable to read sound '{}' from any of '{:?}'",
                    id, builder.source_dirs);
                Error::new(ErrorKind::InvalidData,
                    format!("Unable to create sound_set '{}'", builder.id))
            })?;

            sounds.insert(id.to_string(), source);
        }

        Ok(Rc::new(SoundSet { sounds, id: builder.id }))
    }

    pub fn id(&self) -> &str { &self.id }

    pub fn get(&self, id: &str) -> Option<&SoundSource> {
        self.sounds.get(id)
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct SoundSetBuilder {
    pub id: String,
    pub source_dirs: Vec<String>,
    pub sounds: HashMap<String, EntryBuilder>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EntryBuilder {
    pub file: String,

    #[serde(default)]
    pub loops: bool,

    #[serde(default = "float_1")]
    pub volume: f32,

    #[serde(default)]
    pub delay: f32,
}

fn float_1() -> f32 { 1.0 }
