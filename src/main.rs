use std::io;
use std::io::Write;
use std::process::exit;

pub mod builtins;
pub mod common;
pub mod exec;
pub mod parser;

use crate::builtins::{find_builtin, Builtin};
use crate::common::{debug, report_error, State};
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
        Ok(len) => {
            // println!("Len: {}", len);

            if let Some(cmd) = parser::parse(&line) {
                let res = match find_builtin(&builtins, cmd.args[0]) {
                    Some(builtin) => run_builtin(builtin, state, cmd),
                    None => run_external(state, cmd),
                };
                match res {
                    Ok(status) => state.status = status,
                    Err(err) => {
                        state.status = nix::errno::errno();
                        report_error(&err);
                    }
                }
            } else {
                debug(state, "No command given.");
            }
        }
        Err(err) => {
            state.status = nix::errno::errno();
            report_error(&err);
        }
    }
}

// fn parse_readline() -> Result<i32, io::Error> {
//     let mut line = String::new();
//     let len = io::stdin().read_line(&mut line)?;
//     println!("Len: {}", len);
//     match len {

//     }
//     Ok(0)
// }

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
