use std::collections::HashMap;
use std::fmt;
use std::io;

use crate::state::{State, Status};

mod base;
mod dir;
mod file;
mod process;

type BuiltinHandler = fn(&State, &[&str]) -> io::Result<Status>;

#[derive(Clone)]
pub struct Builtin<'a> {
    pub command: &'a str, //String,
    pub handler: BuiltinHandler,
    pub hint: String,
}

impl<'a> Builtin<'a> {
    pub fn new(command: &'a str, handler: BuiltinHandler, hint: &str) -> Builtin<'a> {
        Builtin {
            command, //: String::from(command),
            handler,
            hint: String::from(hint),
        }
    }
}

impl<'a> fmt::Debug for Builtin<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builtin")
            .field("cmd", &self.command)
            .field("hint", &self.hint)
            .finish()
    }
}

// ********** default builtins **********

pub fn default_builtins<'a>() -> Vec<Builtin<'a>> {
    vec![
        Builtin::new("help", base::do_help, "Print short help"),
        Builtin::new("name", base::do_name, "Print or change shell name"),
        Builtin::new(
            "loglevel",
            base::do_loglevel,
            "Print or change logging level",
        ),
        Builtin::new(
            "status",
            process::do_status,
            "Print status of the last command",
        ),
        Builtin::new("print", base::do_print, "Print arguments"),
        Builtin::new("echo", base::do_echo, "Print arguments and the newline"),
        Builtin::new("pid", process::do_pid, "Print PID"),
        Builtin::new("ppid", process::do_ppid, "Print PPID"),
        Builtin::new("exit", process::do_exit, "Exit from the current shell"),
        Builtin::new("dir.change", dir::do_dir_change, "Change current directory"),
        Builtin::new(
            "dir.where",
            dir::do_dir_where,
            "Print current working directory",
        ),
        Builtin::new("dir.make", dir::do_dir_make, "Make directories"),
        Builtin::new("dir.remove", dir::do_dir_remove, "Remove directories"),
        Builtin::new("dir.list", dir::do_dir_list, "List directory"),
        Builtin::new("dir.inspect", dir::do_dir_inspect, "Inspect directory"),
        Builtin::new("link.hard", file::do_link_hard, "Create hard link"),
        Builtin::new("link.soft", file::do_link_soft, "Create symbolic/soft link"),
        Builtin::new(
            "link.read",
            file::do_link_read,
            "Print symbolic link target",
        ),
        Builtin::new("unlink", file::do_unlink, "Unlink files"),
        Builtin::new("rename", file::do_rename, "Rename file"),
        Builtin::new("cpcat", file::do_cpcat, "Copy file"),
        Builtin::new(
            "depth",
            process::do_depth,
            "Print the depth of the current subshell",
        ),
        Builtin::new(
            "subshell",
            process::do_subshell,
            "Run a subshell with a command",
        ),
        Builtin::new("pipes", process::do_pipes, "Create a pipeline"),
    ]
}

pub fn default_hm<'a>() -> HashMap<&'a str, Builtin<'a>> {
    let mut map = HashMap::new();
    for b in default_builtins() {
        map.insert(b.command, b);
    }
    map
}
