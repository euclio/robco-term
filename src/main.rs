#[macro_use]
extern crate log;
extern crate docopt;
extern crate itertools;
extern crate log4rs;
extern crate ncurses;
extern crate rand;
extern crate rustc_serialize;
extern crate time;

mod game;
mod window;
mod consts;

use docopt::Docopt;
use time::PreciseTime;

use game::{GameState, InputEvent};

static USAGE: &'static str = "
Usage:
    robco-term [options]
    robco-term (-h | --help)

Options:
    -h --help                       Show this screen.
    -d LEVEL --difficulty=LEVEL     Set difficulty of the game (default 5). Currently this only
                                    affects the length of potential passwords.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_difficulty: Option<i32>,
}

fn main() {
    log4rs::init_file("config/log.yaml", Default::default()).unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    info!("Starting game.");

    let mut game_state = GameState::new(args.flag_difficulty.unwrap_or(5));
    let window = window::create();

    let mut last_time = PreciseTime::now();
    while game_state.is_playing {
        let elapsed = last_time.to(PreciseTime::now());
        let event = window.handle_input(&mut game_state);
        match event {
            Some(InputEvent::Quit) => break,
            _ => game_state.update(event, elapsed),
        }
        window.render(&game_state);
        last_time = PreciseTime::now();
    }
}
