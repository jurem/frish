use std::process::exit;

mod builtins;
mod common;
mod parser;
mod shell;
mod state;

use crate::shell::eval::read_eval_loop;
use crate::state::State;

#[macro_use]
extern crate log;

fn main() {
    env_logger::init();
    info!("Initializing shell");
    let interactive = unsafe { libc::isatty(libc::STDIN_FILENO) > 0 };
    let state = State::new("frish", interactive);
    // run
    read_eval_loop(&state);
    // done
    info!("Finalizing shell");
    exit(state.status.get().code());
}
