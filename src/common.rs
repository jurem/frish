use nix::errno;
use std::io;

use crate::builtins::default_builtins;
use crate::builtins::Builtin;

#[derive(Debug)]
pub struct State {
    pub name: String,
    pub debug: bool,
    pub fd_terminal: i32,
    pub interactive: bool,
    pub running: bool,
    pub builtins: Vec<Builtin>,
    pub status: i32,
    pub background: bool,
    pub inredirect: String,
    pub outredirect: String,
}

impl State {
    pub fn new(name: &str) -> State {
        State {
            name: String::from(name),
            debug: false,
            fd_terminal: 0,
            interactive: true,
            running: true,
            builtins: default_builtins(),
            status: 0,
            background: false,
            inredirect: String::new(),
            outredirect: String::new(),
        }
    }

    pub fn find_builtin(&self, name: &str) -> Option<&Builtin> {
        self.builtins.iter().find(|&b| b.cmd == name)
    }
}

// ********** helper functions **********

pub fn handle_error(state: &mut State, errno: i32, msg: &str) {
    state.status = errno;
    eprintln!("{}", msg);
}

pub fn handle_ioerror(state: &mut State, err: &io::Error) {
    handle_error(state, errno::errno(), &err.to_string());
}

pub fn debug(state: &State, msg: &str) {
    if state.debug {
        eprintln!("{}", msg);
    }
}
