use std::io::Write;
use std::process::exit;

pub mod builtins;
pub mod common;
pub mod exec;
pub mod parser;

use crate::common::State;
use crate::exec::subshell;

fn print_prompt(state: &State) {
    if state.interactive {
        print!("{}> ", state.name.borrow());
        std::io::stdout().flush().expect("Cannot flush stdout");
    }
}

fn main() {
    // init
    let builtins = builtins::default_builtins();
    let interactive = unsafe { libc::isatty(libc::STDIN_FILENO) > 0 };
    let state = State::new(builtins, "frish", interactive);
    // run
    while state.running.get() {
        print_prompt(&state);
        subshell(&state);
    }
    // done
    exit(state.status.get());
}
