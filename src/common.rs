use nix::unistd;
use std::cell::{Cell, RefCell};
use std::io;

use crate::builtins::Builtin;

#[derive(Debug, Clone)]
pub struct State {
    pub builtins: Vec<Builtin>,
    pub name: RefCell<String>,
    pub depth: u32,
    pub debug: Cell<bool>,
    pub interactive: bool,
    pub running: Cell<bool>,
    pub status: Cell<i32>,
    pub lastpid: Cell<unistd::Pid>,
}

impl State {
    pub fn new(builtins: Vec<Builtin>, name: &str, interactive: bool) -> State {
        State {
            builtins,
            name: RefCell::new(String::from(name)),
            depth: 0,
            debug: Cell::new(false),
            interactive,
            running: Cell::new(true),
            status: Cell::new(0),
            lastpid: Cell::new(unistd::Pid::from_raw(0)),
        }
    }

    pub fn sub(&self) -> State {
        State {
            builtins: self.builtins.clone(),
            name: self.name.clone(), // RefCell::new(String::from(self.name.borrow())),
            depth: self.depth + 1,
            debug: Cell::new(self.debug.get()),
            interactive: self.interactive,
            running: Cell::new(true),
            status: Cell::new(0),
            lastpid: Cell::new(unistd::Pid::from_raw(0)),
        }
    }

    pub fn terminate(&self) {
        self.running.set(false);
    }

    pub fn set_status(&self, status: i32) {
        self.status.set(status);
    }

    pub fn set_name(&self, name: &str) {
        *self.name.borrow_mut() = String::from(name);
    }

    pub fn find_builtin(&self, name: &str) -> Option<&Builtin> {
        self.builtins.iter().find(|&b| b.command == name)
    }
}

pub struct Command<'a> {
    pub args: Vec<&'a str>,
    pub background: bool,
    pub inredirect: Option<&'a str>,
    pub outredirect: Option<&'a str>,
}

// ********** helper functions **********

pub fn report_error(err: &io::Error) {
    eprintln!("Error: {}", err);
}

pub fn report_nixerror(err: &nix::errno::Errno) {
    eprintln!("Error: {}", err);
}
