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

mod point;
pub use self::point::{Offset, Point, Rect, Scale};

pub mod size;
pub use self::size::Size;

use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::f32;
use std::fmt;
use std::fs;
use std::io::{Error, ErrorKind};
use std::ops::*;
use std::panic;
use std::path::PathBuf;
use std::time::Duration;

use flexi_logger::{opt_format, Duplicate, FileSpec, LogSpecBuilder, Logger, LoggerHandle};
use log::LevelFilter;
use rand::{self, distributions::uniform::SampleUniform, seq::SliceRandom, Rng};
use rand_pcg::Pcg64Mcg;

use crate::config::{self, Config};
use crate::resource::write_to_file;

const MAX_ULPS: i32 = 100;
const MAX_DIFF: f32 = 2.0 * f32::EPSILON;

pub fn approx_eq_slice(a: &[f32], b: &[f32]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for (a, b) in a.iter().zip(b.iter()) {
        if !approx_eq(*a, *b) {
            return false;
        }
    }

    true
}

pub fn approx_eq(a: f32, b: f32) -> bool {
    if (a - b).abs() <= MAX_DIFF {
        return true;
    }

    if a.signum() != b.signum() {
        return false;
    }

    let a_int = a.to_bits() as i32;
    let b_int = b.to_bits() as i32;

    i32::abs(a_int - b_int) <= MAX_ULPS
}

#[derive(Clone)]
pub struct ReproducibleRandom {
    seed: u128,
    gen: Pcg64Mcg,
}

impl ReproducibleRandom {
    pub fn new(seed: Option<u128>) -> ReproducibleRandom {
        // TODO only seed with u64 for now because serde_yaml doesn't serialize u128 correctly
        let seed = match seed {
            Some(s) => s,
            None => rand::thread_rng().gen::<u64>() as u128,
        };

        ReproducibleRandom {
            seed,
            gen: Pcg64Mcg::new(seed),
        }
    }

    pub fn gen<T: SampleUniform + PartialOrd>(&mut self, min: T, max: T) -> T {
        self.gen.gen_range(min..max)
    }

    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        values.shuffle(&mut self.gen);
    }

    pub fn seed(&self) -> u128 {
        self.seed
    }
}

impl std::fmt::Debug for ReproducibleRandom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let state = serde_json::to_string(&self.gen).map_err(|_| std::fmt::Error)?;
        write!(f, "Random: {state}")
    }
}

pub fn shuffle<T>(values: &mut [T]) {
    values.shuffle(&mut rand::thread_rng());
}

pub fn gen_rand<T: SampleUniform + PartialOrd>(min: T, max: T) -> T {
    rand::thread_rng().gen_range(min..max)
}

fn active_resources_file_path() -> PathBuf {
    let mut path = config::USER_DIR.clone();
    path.push("active_resources.yml");
    path
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ActiveResources {
    pub campaign: Option<String>,
    pub mods: Vec<String>,
}

impl ActiveResources {
    pub fn read() -> ActiveResources {
        let path = active_resources_file_path();

        let data = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(_) => {
                info!("active_resources file not found");
                return ActiveResources::default();
            }
        };

        let active_resources: ActiveResources = match serde_yaml::from_str(&data) {
            Ok(val) => val,
            Err(e) => {
                warn!("Error reading active resources file");
                warn!("{}", e);
                return ActiveResources::default();
            }
        };

        active_resources
    }

    pub fn write(&self) {
        let file = active_resources_file_path();
        match write_to_file(file, self) {
            Ok(()) => (),
            Err(e) => {
                warn!("Error writing active resources file");
                warn!("{}", e);
            }
        }
    }

    pub fn directories(&self) -> Vec<String> {
        let mut dirs = vec![Config::resources_config().directory];

        if let Some(ref dir) = self.campaign {
            dirs.push(dir.to_string());
        }

        for mod_dir in self.mods.iter() {
            dirs.push(mod_dir.to_string());
        }

        dirs
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(deny_unknown_fields, untagged)]
pub enum ExtInt {
    Int(u32),
    Infinity,
}

impl Ord for ExtInt {
    fn cmp(&self, other: &ExtInt) -> Ordering {
        match self {
            ExtInt::Int(val) => match other {
                ExtInt::Int(other) => val.cmp(other),
                ExtInt::Infinity => Ordering::Less,
            },
            ExtInt::Infinity => match other {
                ExtInt::Int(_) => Ordering::Greater,
                ExtInt::Infinity => Ordering::Equal,
            },
        }
    }
}

impl PartialOrd for ExtInt {
    fn partial_cmp(&self, other: &ExtInt) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ExtInt {
    pub fn max(a: ExtInt, b: ExtInt) -> ExtInt {
        if a > b {
            a
        } else {
            b
        }
    }

    pub fn min(a: ExtInt, b: ExtInt) -> ExtInt {
        if a > b {
            b
        } else {
            a
        }
    }

    pub fn divide(self, other: ExtInt) -> f32 {
        match self {
            ExtInt::Int(amount) => match other {
                ExtInt::Int(other_amount) => amount as f32 / other_amount as f32,
                ExtInt::Infinity => 0.0,
            },
            ExtInt::Infinity => match other {
                ExtInt::Int(_) => 0.0,
                ExtInt::Infinity => 1.0,
            },
        }
    }

    pub fn is_zero(self) -> bool {
        match self {
            ExtInt::Int(amount) => amount == 0,
            ExtInt::Infinity => false,
        }
    }

    pub fn is_infinite(self) -> bool {
        match self {
            ExtInt::Int(_) => false,
            ExtInt::Infinity => true,
        }
    }

    pub fn to_f32(self) -> f32 {
        match self {
            ExtInt::Int(amount) => amount as f32,
            ExtInt::Infinity => 1e12, // use a value that serde json can serialize properly
        }
    }

    pub fn less_than(self, other: u32) -> bool {
        match self {
            ExtInt::Int(amount) => amount < other,
            ExtInt::Infinity => false,
        }
    }

    pub fn greater_than(self, other: u32) -> bool {
        match self {
            ExtInt::Int(amount) => amount > other,
            ExtInt::Infinity => true,
        }
    }
}

impl fmt::Display for ExtInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExtInt::Int(amount) => write!(f, "{amount}"),
            ExtInt::Infinity => write!(f, "infinity"),
        }
    }
}

impl Mul<u32> for ExtInt {
    type Output = ExtInt;
    fn mul(self, other: u32) -> ExtInt {
        match self {
            ExtInt::Int(amount) => ExtInt::Int(amount * other),
            ExtInt::Infinity => ExtInt::Infinity,
        }
    }
}

impl Add<ExtInt> for ExtInt {
    type Output = ExtInt;
    fn add(self, other: ExtInt) -> ExtInt {
        match self {
            ExtInt::Int(amount) => match other {
                ExtInt::Int(other_amount) => ExtInt::Int(amount + other_amount),
                ExtInt::Infinity => ExtInt::Infinity,
            },
            ExtInt::Infinity => ExtInt::Infinity,
        }
    }
}

impl Add<u32> for ExtInt {
    type Output = ExtInt;
    fn add(self, other: u32) -> ExtInt {
        match self {
            ExtInt::Int(amount) => ExtInt::Int(amount + other),
            ExtInt::Infinity => ExtInt::Infinity,
        }
    }
}

impl Sub<u32> for ExtInt {
    type Output = ExtInt;
    fn sub(self, other: u32) -> ExtInt {
        match self {
            ExtInt::Int(amount) => {
                if other > amount {
                    ExtInt::Int(0)
                } else {
                    ExtInt::Int(amount - other)
                }
            }
            ExtInt::Infinity => ExtInt::Infinity,
        }
    }
}

pub fn invalid_data_error<T>(str: &str) -> Result<T, Error> {
    Err(Error::new(ErrorKind::InvalidData, str))
}

pub fn unable_to_create_error<T>(kind: &str, id: &str) -> Result<T, Error> {
    Err(Error::new(
        ErrorKind::InvalidData,
        format!("Unable to create {kind} '{id}'"),
    ))
}

/// Helper function to return the number of milliseconds elapsed in
/// the given duration.
pub fn get_elapsed_millis(elapsed: Duration) -> u32 {
    (elapsed.as_secs() as u32) * 1_000 + elapsed.subsec_millis()
}

/// Helper function to return a string representation of the elapsed time
/// in seconds
pub fn format_elapsed_secs(elapsed: Duration) -> String {
    let secs = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;
    format!("{secs:.6}")
}

pub fn error_and_exit(error: &str) {
    error!("{}", error);
    error!("Exiting...");
    ::std::process::exit(1)
}

#[must_use]
pub fn setup_logger() -> LoggerHandle {
    let mut path = config::USER_DIR.clone();
    path.push("log");
    let log_dir = path;

    let log_config = Config::logging_config();

    let mut log_builder = LogSpecBuilder::new();
    log_builder.default(log_config.log_level);

    let dup = match log_config.stderr_log_level {
        LevelFilter::Error => Duplicate::Error,
        LevelFilter::Warn => Duplicate::Warn,
        LevelFilter::Info => Duplicate::Info,
        LevelFilter::Debug => Duplicate::Debug,
        LevelFilter::Trace => Duplicate::Trace,
        LevelFilter::Off => Duplicate::None,
    };

    let logger = Logger::with(log_builder.finalize())
        .log_to_file(
            FileSpec::default()
                .directory(log_dir)
                .use_timestamp(log_config.use_timestamps),
        )
        .print_message()
        .duplicate_to_stderr(dup)
        .o_append(log_config.append)
        .format(opt_format);

    let handle = logger.start().unwrap_or_else(|e| {
        eprintln!("{e}");
        eprintln!("There was a fatal error initializing logging to 'log/'");
        eprintln!("Exiting...");
        ::std::process::exit(1);
    });

    panic::set_hook(Box::new(|p| {
        if let Some(s) = p.payload().downcast_ref::<String>() {
            error!("Thread main panic with: '{}'", s);
        } else if let Some(s) = p.payload().downcast_ref::<&str>() {
            error!("Thread main panic with: '{}'", s);
        } else {
            error!("Thread main panic");
        }
        warn!("at {:?}", p.location());

        let bt = std::backtrace::Backtrace::force_capture();
        warn!("{:?}", bt);
    }));

    create_user_dirs();

    handle
}

fn create_user_dirs() {
    let res = Config::resources_config();

    let mut campaign_dir = config::USER_DIR.clone();
    campaign_dir.push(&res.campaigns_directory);
    config::create_dir_and_warn(&campaign_dir);

    let mut mods_dir = config::USER_DIR.clone();
    mods_dir.push(&res.mods_directory);
    config::create_dir_and_warn(&mods_dir);
}
