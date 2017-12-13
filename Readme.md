# rust-game
This is a game written in Rust.  It is currently in very early development.

## Getting Started

### Prerequisites
You'll need recent versions of Rust and Cargo installed.  The game currently works on Stable, but may eventually require nightly.  [Rustup](https://www.rustup.rs/)

### Installation

1. Clone the git repository.
1. `cargo build`
1. `cp config.sample.yml config.yml`
1. Edit `config.yml` with your preferences.  On Windows, you'll need to specify the "Pancurses" display adapter instead of "Termion".
1. Run the game with `cargo run`

## Built With
* [Pancurses](https://github.com/ihalila/pancurses)
* [Termion](https://github.com/ticki/termion)
* [Serde](https://serde.rs/)

## Authors
* **Jared Stephen** - *Initial Development* - [Grokmoo](https://github.com/Grokmoo)

## License

This project is licensed under the GPLv3 - see the [LICENSE](LICENSE) file for details

