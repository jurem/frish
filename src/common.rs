use nix::unistd;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt;
use std::io;

use crate::builtins::Builtin;

// I guess I could use std::process::ExitStatus, but let's play
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Status(i32);

impl Status {
    pub fn from_code(code: i32) -> Status {
        Status(code)
    }

    pub fn from(status: &Status) -> Status {
        Status::from_code(status.0)
    }

    pub fn success() -> Status {
        Status::from_code(0)
    }

    pub fn code(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f) // instead of write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct State<'a> {
    pub builtins: HashMap<&'a str, Builtin<'a>>,
    pub name: RefCell<String>,
    pub depth: u32,
    pub debug: Cell<bool>,
    pub interactive: bool,
    pub running: Cell<bool>,
    pub status: Cell<Status>,
    pub lastpid: Cell<unistd::Pid>,
}

impl<'a> State<'a> {
    pub fn new(
        builtins: HashMap<&'a str, Builtin<'a>>,
        name: &str,
        interactive: bool,
    ) -> State<'a> {
        State {
            builtins,
            name: RefCell::new(String::from(name)),
            depth: 0,
            debug: Cell::new(false),
            interactive,
            running: Cell::new(true),
            status: Cell::new(Status(0)),
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
            status: Cell::new(Status(0)),
            lastpid: Cell::new(unistd::Pid::from_raw(0)),
        }
    }

    pub fn terminate(&self) {
        self.running.set(false);
    }

    pub fn set_status_code(&self, code: i32) {
        self.status.set(Status(code));
    }

    pub fn set_status(&self, status: &Status) {
        self.set_status_code(status.0);
    }

    pub fn set_name(&self, name: &str) {
        *self.name.borrow_mut() = String::from(name);
    }

    pub fn find_builtin(&self, name: &str) -> Option<&Builtin> {
        self.builtins.get(name)
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
