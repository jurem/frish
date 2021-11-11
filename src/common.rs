use std::io;

#[derive(Debug)]
pub struct State {
    pub name: String,
    pub debug: bool,
    pub interactive: bool,
    pub running: bool,
    pub fd_terminal: i32,
    pub status: i32,
}

impl State {
    pub fn new(name: &str) -> State {
        State {
            name: String::from(name),
            debug: false,
            interactive: true,
            running: true,
            fd_terminal: 0,
            status: 0,
        }
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

pub fn debug(state: &State, msg: &str) {
    if state.debug {
        eprintln!("{}", msg);
    }
}

#[macro_export]
macro_rules! log {
    ($( $args:expr ),*) => { eprintln!( $( $args ),* ); }
}
