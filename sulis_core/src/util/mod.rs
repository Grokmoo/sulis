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
pub use self::point::Point;

pub mod size;
pub use self::size::Size;

use std::rc::Rc;
use std::cell::RefCell;
use std::io::{Error, ErrorKind};
use std::{thread, time};
use std::time::Duration;
use std::panic;

use backtrace::Backtrace;
use flexi_logger::{Logger, opt_format};

use config::CONFIG;
use ui::Widget;
use io::{IO, MainLoopUpdater};

pub fn invalid_data_error<T>(str: &str) -> Result<T, Error> {
    Err(Error::new(ErrorKind::InvalidData, str))
}

/// Helper function to return the number of milliseconds elapsed in
/// the given duration.
pub fn get_elapsed_millis(elapsed: Duration) -> u32 {
    (elapsed.as_secs() as u32) * 1_000 +
        elapsed.subsec_nanos() / 1_000_000
}

/// Helper function to return a string representation of the elapsed time
/// in seconds
pub fn format_elapsed_secs(elapsed: Duration) -> String {
    let secs = elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9;
    format!("{:.6}", secs)
}

pub fn ok_and_exit(message: &str) {
    info!("{}", message);
    info!("Exiting...");
    ::std::process::exit(0)
}

pub fn error_and_exit(error: &str) {
    error!("{}", error);
    error!("Exiting...");
    ::std::process::exit(1)
}

pub fn main_loop(io: &mut Box<IO>, root: Rc<RefCell<Widget>>,
             updater: Box<MainLoopUpdater>) -> Result<(), Error> {
    let fpms = (1000.0 / (CONFIG.display.frame_rate as f32)) as u64;
    let frame_time = time::Duration::from_millis(fpms);
    trace!("Computed {} frames per milli.", fpms);

    info!("Starting main loop.");
    let main_loop_start_time = time::Instant::now();

    let mut frames = 0;
    let mut render_time = time::Duration::from_secs(0);
    loop {
        let start_time = time::Instant::now();

        io.process_input(Rc::clone(&root));
        updater.update();

        if let Err(e) = Widget::update(&root) {
            error!("There was a fatal error updating the UI tree state.");
            return Err(e);
        }

        let total_elapsed = get_elapsed_millis(main_loop_start_time.elapsed());
        io.render_output(root.borrow(), total_elapsed);

        if updater.is_exit() {
            trace!("Exiting main loop.");
            break;
        }

        let frame_elapsed = start_time.elapsed();
        if frame_time > frame_elapsed {
            thread::sleep(frame_time - frame_elapsed);
        }

        render_time += frame_elapsed;
        frames += 1;
    }

    let secs = render_time.as_secs() as f64 + render_time.subsec_nanos() as f64 * 1e-9;
    info!("Rendered {} frames with total render time {:.4} seconds", frames, secs);
    info!("Average frame render time: {:.2} milliseconds", 1000.0 * secs / frames as f64);

    Ok(())
}

pub fn setup_logger() {
    let mut logger = Logger::with_str(&CONFIG.logging.log_level)
        .log_to_file()
        .directory("log")
        .duplicate_error()
        .format(opt_format);

    if !CONFIG.logging.use_timestamps {
        logger = logger.suppress_timestamp();
    }

    logger.start()
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            eprintln!("There was a fatal error initializing logging to 'log/'");
            eprintln!("Exiting...");
            ::std::process::exit(1);
        });

    panic::set_hook(Box::new(|p| {
        error!("Thread main panic.  Exiting.");
        warn!("with payload: {:?}", p.payload());
        if let Some(loc) = p.location() {
           warn!("at {:?}", loc);
        }

        let bt = Backtrace::new();
        warn!("{:?}", bt);
    }));
}
