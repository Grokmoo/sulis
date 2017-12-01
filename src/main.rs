extern crate game;

use std::io;
use std::error::Error;

use game::config;
use game::resource;
use game::state::GameState;

fn main() {
    let stdout = io::stdout();
    let stdin = io::stdin();
    let stdout = stdout.lock();
    let stdin = stdin.lock();

    let config = config::Config::new("config.toml");
    let config = match config {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("There was a fatal error loading the configuration from 'config.toml'");
            eprintln!("Exiting...");
            ::std::process::exit(1);
        }
    };

    let resource_set = resource::ResourceSet::new(&config.resources.directory);
    let resource_set = match resource_set {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  {}: {}", e.description(), e);
            eprintln!("There was a fatal error loading resource set from 'data':");
            eprintln!("Exiting...");
            ::std::process::exit(1);
        }
    };

    let mut io = game::io::create(config.display.adapter, stdin, stdout);
    
    let game_state = GameState::new(config, &resource_set);
    let mut game_state = match game_state {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}",  e);
            eprintln!("There was a fatal error creating the game state.");
            eprintln!("Exiting...");
            ::std::process::exit(1);
        }
    };

    loop {
        io.render_output(&game_state);
        io.process_input(&mut game_state);
    }
}
