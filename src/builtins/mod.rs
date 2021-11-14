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
    pub command: &'a str,
    pub handler: BuiltinHandler,
    pub hint: &'a str,
}

impl<'a> Builtin<'a> {
    pub fn new(command: &'a str, handler: BuiltinHandler, hint: &'a str) -> Self {
        Builtin {
            command,
            handler,
            hint,
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

#[derive(Debug, Clone)]
pub struct Builtins<'a> {
    items: Vec<Builtin<'a>>,
    hash: HashMap<&'a str, usize>,
}

impl<'a> Builtins<'a> {
    pub fn new() -> Self {
        let mut hash = HashMap::new();
        let items = default_builtins();
        for (i, b) in items.iter().enumerate() {
            hash.insert(b.command, i);
        }
        Builtins { items, hash }
    }

    pub fn find(&self, name: &str) -> Option<&Builtin> {
        self.hash.get(name).map(|&idx| &self.items[idx])
    }
}

// ********** default builtins **********

#[inline]
fn builtin<'a>(name: &'a str, handler: BuiltinHandler, hint: &'a str) -> Builtin<'a> {
    Builtin::new(name, handler, hint)
}

fn default_builtins<'a>() -> Vec<Builtin<'a>> {
    use base::*;
    use dir::*;
    use file::*;
    use process::*;
    vec![
        // base
        builtin("help", do_help, "Print short help for all builtin commands"),
        builtin("name", do_name, "Print or change the shell name"),
        builtin("loglevel", do_loglevel, "Print or change logging level"),
        builtin("print", do_print, "Print its arguments"),
        builtin("echo", do_echo, "Print its arguments and the newline"),
        // dir
        builtin("dir.change", do_dir_change, "Change the current directory"),
        builtin("dir.where", do_dir_where, "Print current working directory"),
        builtin("dir.make", do_dir_make, "Make directories"),
        builtin("dir.remove", do_dir_remove, "Remove directories"),
        builtin("dir.list", do_dir_list, "List directory"),
        builtin("dir.inspect", do_dir_inspect, "Inspect directory"),
        // file
        builtin("link.hard", do_link_hard, "Create hard link"),
        builtin("link.soft", do_link_soft, "Create symbolic/soft link"),
        builtin("link.read", do_link_read, "Print symbolic link target"),
        builtin("unlink", do_unlink, "Unlink files"),
        builtin("rename", do_rename, "Rename file"),
        builtin("cpcat", do_cpcat, "Copy file"),
        // process
        builtin("pid", do_pid, "Print PID of the current shell"),
        builtin("ppid", do_ppid, "Print PPID of the current shell"),
        builtin("status", do_status, "Print status of the last command"),
        builtin("exit", do_exit, "Exit from the current shell"),
        builtin("depth", do_depth, "Print the depth of the current subshell"),
        builtin("subshell", do_subshell, "Run a subshell with a command"),
        builtin("pipes", do_pipes, "Create a pipeline"),
    ]
}
