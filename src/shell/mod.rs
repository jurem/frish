use std::io;

use crate::common::report_error;
use crate::common::State;
use crate::parser;

pub mod exec;
mod redirect;

pub fn eval(state: &State, cmdstr: &str) {
    if let Some(cmd) = parser::parse(&cmdstr) {
        let res = match state.find_builtin(cmd.args[0]) {
            Some(builtin) => exec::run_builtin(builtin, state, &cmd),
            None => exec::run_external(&cmd),
        };
        match res {
            Ok(status) => state.set_status(&status),
            Err(err) => {
                state.set_status_code(nix::errno::errno());
                report_error(&err);
            }
        }
    } else {
        debug!("No command given.");
    }
}

pub fn read_eval(state: &State) {
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => state.terminate(),
        Ok(_) => eval(state, &line),
        Err(err) => {
            state.set_status_code(nix::errno::errno());
            report_error(&err);
        }
    }
}

pub fn read_eval_loop(state: &State) {
    while state.running.get() {
        exec::print_prompt(&state);
        read_eval(&state);
    }
}
