use std::io;
use std::io::Write;
use std::process::exit;

pub mod builtins;
pub mod common;
pub mod exec;
pub mod parser;

use crate::builtins::{find_builtin, Builtin};
use crate::common::{debug, handle_ioerror, State};
use crate::exec::{run_builtin, run_external};

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

fn read_evaluate(builtins: &[Builtin], state: &mut State) {
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => state.running = false,
        Ok(_len) => {
            if let Some(p) = parser::parse(&line) {
                match find_builtin(&builtins, p.args[0]) {
                    Some(builtin) => run_builtin(builtin, state, &p.args),
                    None => run_external(state, &p.args),
                }
            } else {
                debug(state, "No command given.");
            }
        }
        Err(err) => handle_ioerror(state, &err),
    }
}

fn main() {
    let builtins = builtins::default_builtins();
    let mut state = State::new("frish");
    init(&mut state);
    // run
    while state.running {
        print_prompt(&state);
        read_evaluate(&builtins, &mut state);
    }
    // done
    done();
    exit(state.status);
}
