extern crate game;

use std::io;
use std::error::Error;

use game::resource;
use game::state::GameState;

fn main() {
    let stdout = io::stdout();
    let stdin = io::stdin();
    let stdout = stdout.lock();
    let stdin = stdin.lock();

    let resource_set = resource::ResourceSet::new("data");
    let resource_set = match resource_set {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  {}: {}", e.description(), e);
            eprintln!("There was a fatal error loading resource set from 'data':");
            eprintln!("Exiting...");
            ::std::process::exit(1);
        }
    };

    let game_state = GameState::new(&resource_set);
    let mut game_state = match game_state {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}",  e);
            eprintln!("There was a fatal error creating the game state.");
            eprintln!("Exiting...");
            ::std::process::exit(1);
        }
    };
    
    let mut io = game::io::create(game::io::Type::Termion, stdin, stdout);

    loop {
        io.process_input(&mut game_state);
        io.render_output(&game_state);
    }
}
