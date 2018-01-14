extern crate grt;
extern crate game;

extern crate backtrace;
use backtrace::Backtrace;

#[macro_use] extern crate log;
extern crate flexi_logger;

use std::{thread, time};
use std::rc::Rc;
use std::cell::RefCell;
use std::panic;
use std::io::Error;

use flexi_logger::{Logger, opt_format};

use grt::ui::{self, Widget};
use grt::io::{IO, MainLoopUpdater};
use grt::config::CONFIG;
use grt::resource::ResourceSet;

use game::state::{GameStateMainLoopUpdater, GameState};
use game::animation;
use game::view::RootView;
use game::module::Module;
use game::main_menu::{MainMenuView, MainMenuLoopUpdater};

fn main() {
    // CONFIG will be lazily initialized here; if it fails it
    // prints an error and exits
    setup_logger();
    info!("Setup Logger and read configuration from 'config.yml'");

    info!("Reading resources from {}", CONFIG.resources.directory);
    if let Err(e) = ResourceSet::init(&CONFIG.resources.directory) {
        error!("{}", e);
        error!("There was a fatal error loading resource set from 'data':");
        error!("Exiting...");
        ::std::process::exit(1);
    };

    info!("Setting up display adapter.");
    let mut io = match grt::io::create() {
        Ok(io) => io,
        Err(e) => {
            error!("{}", e);
            error!("There was a fatal error initializing the display.");
            error!("Exiting...");
            ::std::process::exit(1);
        }
    };

    let modules_list = vec!["Module1".to_string(), "Module2".to_string(), "TestCampaign".to_string()];

    {
        let main_menu_view = MainMenuView::new(modules_list);
        let loop_updater = MainMenuLoopUpdater::new(&main_menu_view);
        let main_menu_root = ui::create_ui_tree(main_menu_view);
        match ResourceSet::get_theme().children.get("main_menu") {
            None => warn!("No theme found for 'main_menu"),
            Some(ref theme) => {
                main_menu_root.borrow_mut().theme = Some(Rc::clone(theme));
                main_menu_root.borrow_mut().theme_id = ".main_menu".to_string();
                main_menu_root.borrow_mut().theme_subname = "main_menu".to_string();
            }
        }

        if let Err(e) = main_loop(&mut io, main_menu_root, Box::new(loop_updater)) {
            error!("{}", e);
            error!("Error in main menu.  Exiting...");
            ::std::process::exit(1);
        }
    }

    info!("Reading module from {}", CONFIG.resources.directory);
    if let Err(e) =  Module::init(&CONFIG.resources.directory) {
        error!("{}", e);
        error!("There was a fatal error setting up the module.");
        error!("Exiting...");
        ::std::process::exit(1);
    };

    info!("Initializing game state.");
    if let Err(e) = GameState::init() {
        error!("{}",  e);
        error!("There was a fatal error creating the game state.");
        error!("Exiting...");
        ::std::process::exit(1);
    };

    let root = ui::create_ui_tree(RootView::new());

    if let Err(e) = main_loop(&mut io, root, Box::new(GameStateMainLoopUpdater { })) {
        error!("{}", e);
        error!("Error in main loop.  Exiting...");
    }

    info!("Shutting down.");
}

fn main_loop(io: &mut Box<IO>, root: Rc<RefCell<Widget>>,
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

        let total_elapsed =
            animation::get_elapsed_millis(main_loop_start_time.elapsed());
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
