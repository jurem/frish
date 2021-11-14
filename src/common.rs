use std::io;

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
