use std::process::exit;

mod builtins;
pub mod common;
pub mod exec;
pub mod parser;
mod shell;

use crate::common::State;
use crate::exec::read_eval_loop;

#[macro_use]
extern crate log;

fn main() {
    // init
    env_logger::init();
    info!("Initializing shell");
    let builtins = builtins::default_hm();
    let interactive = unsafe { libc::isatty(libc::STDIN_FILENO) > 0 };
    let state = State::new(builtins, "frish", interactive);
    // run
    read_eval_loop(&state);
    // done
    info!("Finalizing shell");
    exit(state.status.get().code());
}
