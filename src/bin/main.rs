extern crate grt;
extern crate game;

extern crate backtrace;
use backtrace::Backtrace;

#[macro_use] extern crate log;
extern crate flexi_logger;

use std::{thread, time};
use std::rc::Rc;
use std::panic;

use grt::ui::{self, Widget};
use grt::config::CONFIG;
use grt::resource;

use game::state::GameState;
use game::animation;
use game::view::RootView;
use game::module::Module;

use flexi_logger::{Logger, opt_format};

fn main() {
    // CONFIG will be lazily initialized here; if it fails it
    // prints an error and exits
    setup_logger();
    info!("Setup Logger and read configuration from 'config.yml'");

    info!("Reading resources from {}", CONFIG.resources.directory);
    let resource_set_err = resource::ResourceSet::init(&CONFIG.resources.directory);
    match resource_set_err {
        Ok(_) => (),
        Err(e) => {
            error!("{}", e);
            error!("There was a fatal error loading resource set from 'data':");
            error!("Exiting...");
            ::std::process::exit(1);
        }
    };

    info!("Reading module from {}", CONFIG.resources.directory);
    match Module::init(&CONFIG.resources.directory) {
        Ok(_) => (),
        Err(e) => {
            error!("{}", e);
            error!("There was a fatal error setting up the module.");
            error!("Exiting...");
            ::std::process::exit(1);
        }
    };

    info!("Initializing game state.");
    match GameState::init() {
        Ok(_) => {},
        Err(e) => {
            error!("{}",  e);
            error!("There was a fatal error creating the game state.");
            error!("Exiting...");
            ::std::process::exit(1);
        }
    };

    info!("Setting up display adapter.");
    let io = grt::io::create();
    let mut io = match io {
        Ok(io) => io,
        Err(e) => {
            error!("{}", e);
            error!("There was a fatal error initializing the display.");
            error!("Exiting...");
            ::std::process::exit(1);
        }
    };

    let root = ui::create_ui_tree(RootView::new());

    let fpms = (1000.0 / (CONFIG.display.frame_rate as f32)) as u64;
    let frame_time = time::Duration::from_millis(fpms);
    trace!("Computed {} frames per milli.", fpms);

    info!("Setup complete.");
    let main_loop_start_time = time::Instant::now();

    let mut frames = 0;
    let mut render_time = time::Duration::from_secs(0);
    loop {
        let start_time = time::Instant::now();

        io.process_input(Rc::clone(&root));
        GameState::update();

        match Widget::update(&root) {
            Err(e) => {
                error!("{}", e);
                error!("There was a fatal error updating the UI tree state.");
                break;
            }
            _ => (),
        }

        let total_elapsed =
            animation::get_elapsed_millis(main_loop_start_time.elapsed());
        io.render_output(root.borrow(), total_elapsed);

        if GameState::is_exit() {
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

    info!("Shutting down.");
}

fn setup_logger() {
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
        debug!("with payload: {:?}", p.payload());
        if let Some(loc) = p.location() {
           debug!("at {:?}", loc);
        }

        let bt = Backtrace::new();
        debug!("{:?}", bt);
    }));
}
