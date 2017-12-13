extern crate game;

extern crate backtrace;
use backtrace::Backtrace;

#[macro_use] extern crate log;
extern crate flexi_logger;

use std::error::Error;
use std::{thread, time};
use std::rc::Rc;
use std::panic;

use game::config;
use game::resource;
use game::state::GameState;
use game::ui;
use game::animation;

use flexi_logger::{Logger, opt_format};

fn main() {
    info!("Initializing");
    info!("Parsing configuration file 'config.json'");
    let config = config::Config::new("config.json");
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("There was a fatal error loading the configuration from 'config.json'");
            eprintln!("Exiting...");
            ::std::process::exit(1);
        }
    };

    setup_logger(&config.log_level);

    info!("Reading resources from {}", &config.resources.directory);
    let resource_set = resource::ResourceSet::new(&config.resources.directory);
    let resource_set = match resource_set {
        Ok(r) => r,
        Err(e) => {
            error!("  {}: {}", e.description(), e);
            error!("There was a fatal error loading resource set from 'data':");
            error!("Exiting...");
            ::std::process::exit(1);
        }
    };

    info!("Initializing game state.");
    let game_state = GameState::new(config.clone(), &resource_set);
    let mut game_state = match game_state {
        Ok(s) => s,
        Err(e) => {
            error!("{}",  e);
            error!("There was a fatal error creating the game state.");
            error!("Exiting...");
            ::std::process::exit(1);
        }
    };

    info!("Setting up display adapter.");
    let mut io = game::io::create(&config);

    let mut root = ui::create_ui_tree(Rc::clone(&game_state.area_state), &config,
        &resource_set);

    let fpms = (1000.0 / (config.display.frame_rate as f32)) as u64;
    let frame_time = time::Duration::from_millis(fpms);
    trace!("Computed {} frames per milli.", fpms);

    info!("Setup complete.");
    let main_loop_start_time = time::Instant::now();
    loop {
        let start_time = time::Instant::now();

        io.process_input(&mut game_state, &mut root);
        game_state.update();

        let total_elapsed =
            animation::get_elapsed_millis(main_loop_start_time.elapsed());
        io.render_output(&game_state, &root, total_elapsed);

        if game_state.should_exit {
            trace!("Exiting main loop.");
            break;
        }

        let frame_elapsed = start_time.elapsed();
        if frame_time > frame_elapsed {
            thread::sleep(frame_time - frame_elapsed);
        }
    }

    info!("Shutting down.");
}

fn setup_logger(log_level: &str) {
    Logger::with_str(log_level)
        .log_to_file()
        .directory("log")
        .suppress_timestamp()
        .duplicate_error()
        .format(opt_format)
        .start()
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            eprintln!("There was a fatal error initializing logging to 'log/'");
            eprintln!("Exiting...");
            ::std::process::exit(1);
        });

    panic::set_hook(Box::new(|p| {
        error!("Thread main panic.  Exiting.");
        debug!("with payload: {:?}", p.payload());
        if let Some(loc) = p.location() {
           debug!("at {:?}", loc);
        }

        let bt = Backtrace::new();
        debug!("{:?}", bt);
    }));
}
