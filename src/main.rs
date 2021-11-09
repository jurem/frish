use std::io;
use std::io::Write;
use std::process::exit;

pub mod builtins;
pub mod common;
pub mod exec;
pub mod parser;

use crate::common::State;
use crate::exec::run_command;

fn print_prompt(state: &State) {
    if state.interactive {
        print!("{}> ", state.name);
        std::io::stdout().flush().expect("Cannot flush stdout");
    }
}

fn init(state: &mut State) {
    extern crate libc;
    state.fd_terminal = libc::STDIN_FILENO;
    state.interactive = unsafe { libc::isatty(state.fd_terminal) > 0 };
    println!("{:?}", unsafe { libc::isatty(state.fd_terminal) });
}

fn done() {}

fn main() {
    let mut state = State::new("frish");
    init(&mut state);
    // run
    while state.running {
        print_prompt(&state);
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break,
            Ok(_len) => run_command(&mut state, &line),
            Err(err) => println!("error: {}", err),
        }
    }
    // done
    done();
    exit(state.status);
}
