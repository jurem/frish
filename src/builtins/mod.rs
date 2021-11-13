use std::fmt;
use std::io;

use crate::common::{State, Status};

mod base;
mod dir;
mod file;
mod process;

type BuiltinHandler = fn(&State, &[&str]) -> io::Result<Status>;

#[derive(Clone)]
pub struct Builtin {
    pub handler: BuiltinHandler,
    pub command: String,
    pub help: String,
}

impl Builtin {
    pub fn new(fun: BuiltinHandler, cmd: &str, help: &str) -> Builtin {
        Builtin {
            handler: fun,
            command: String::from(cmd),
            help: String::from(help),
        }
    }
}

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builtin")
            .field("cmd", &self.command)
            .field("help", &self.help)
            .finish()
    }
}

// ********** default builtins **********

pub fn default_builtins() -> Vec<Builtin> {
    vec![
        Builtin::new(base::do_help, "help", "Print short help"),
        Builtin::new(base::do_name, "name", "Print or change shell name"),
        Builtin::new(
            base::do_loglevel,
            "loglevel",
            "Print or change logging level",
        ),
        Builtin::new(
            process::do_status,
            "status",
            "Print status of the last command",
        ),
        Builtin::new(base::do_print, "print", "Print arguments"),
        Builtin::new(base::do_echo, "echo", "Print arguments and the newline"),
        Builtin::new(process::do_pid, "pid", "Print PID"),
        Builtin::new(process::do_ppid, "ppid", "Print PPID"),
        Builtin::new(process::do_exit, "exit", "Exit from the current shell"),
        Builtin::new(dir::do_dir_change, "dir.change", "Change current directory"),
        Builtin::new(
            dir::do_dir_where,
            "dir.where",
            "Print current working directory",
        ),
        Builtin::new(dir::do_dir_make, "dir.make", "Make directories"),
        Builtin::new(dir::do_dir_remove, "dir.remove", "Remove directories"),
        Builtin::new(dir::do_dir_list, "dir.list", "List directory"),
        Builtin::new(dir::do_dir_inspect, "dir.inspect", "Inspect directory"),
        Builtin::new(file::do_link_hard, "link.hard", "Create hard link"),
        Builtin::new(file::do_link_soft, "link.soft", "Create symbolic/soft link"),
        Builtin::new(
            file::do_link_read,
            "link.read",
            "Print symbolic link target",
        ),
        Builtin::new(file::do_unlink, "unlink", "Unlink files"),
        Builtin::new(file::do_rename, "rename", "Rename file"),
        Builtin::new(file::do_cpcat, "cpcat", "Copy file"),
        Builtin::new(
            process::do_depth,
            "depth",
            "Print the depth of the current subshell",
        ),
        Builtin::new(
            process::do_subshell,
            "subshell",
            "Run a subshell with a command",
        ),
    ]
}
