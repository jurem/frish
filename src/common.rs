use crate::builtins::default_builtins;
use crate::builtins::Builtin;

#[derive(Debug)]
pub struct State {
    pub name: String,
    pub debug: bool,
    pub interactive: bool,
    pub running: bool,
    pub status: i32,
    pub builtins: Vec<Builtin>,
}

impl State {
    pub fn new(name: &str) -> State {
        State {
            name: String::from(name),
            debug: false,
            interactive: true,
            running: true,
            status: 0,
            builtins: default_builtins(),
        }
    }

    pub fn find(&self, name: &str) -> Option<&Builtin> {
        self.builtins.iter().find(|&b| b.cmd == name)
    }
}
