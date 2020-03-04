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

use std::cell::RefCell;
use std::time::Instant;

use rlua::{self, UserData, UserDataMethods};

use crate::config::Config;

thread_local! {
    static BENCH: RefCell<Vec<Bench>> = RefCell::new(Vec::new());
}

pub fn start_bench(tag: Option<String>) -> Handle {
    BENCH.with(|benches| {
        let mut benches = benches.borrow_mut();

        let handle = Handle::new(benches.len());
        benches.push(Bench::new(handle, tag));
        handle
    })
}

pub fn end_bench(handle: Handle) {
    let end = Instant::now();
    let index = handle.index;
    BENCH.with(|benches| {
        let mut benches = benches.borrow_mut();

        let bench = &mut benches[index];
        bench.end = Some(end);
        bench.report();
    });
}

#[derive(Copy, Clone, Debug)]
pub struct Handle {
    index: usize,
}

impl Handle {
    fn new(index: usize) -> Handle {
        Handle { index }
    }
}

impl UserData for Handle {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(_methods: &mut M) {}
}

struct Bench {
    handle: Handle,
    tag: Option<String>,
    start: Instant,
    end: Option<Instant>,
}

impl Bench {
    fn new(handle: Handle, tag: Option<String>) -> Bench {
        Bench {
            handle,
            tag,
            start: Instant::now(),
            end: None,
        }
    }

    fn report(&self) {
        let end = match self.end {
            None => return,
            Some(end) => end,
        };

        let id;
        if let Some(tag) = &self.tag {
            id = tag.to_string();
        } else {
            id = self.handle.index.to_string();
        }

        let micros = end.duration_since(self.start).as_micros();
        let millis = micros as f64 / 1000.0;

        log!(Config::bench_log_level(), "BENCHMARK '{}': {:.3} millis", id, millis);
    }
}
