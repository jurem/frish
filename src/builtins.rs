use nix::unistd;

use std::env;
use std::fmt;
use std::fs; // portable FS functions
use std::io;

use crate::common::{handle_ioerror, State};

type BuiltinFun = fn(&mut State, Vec<&str>);

pub struct Builtin {
    pub fun: BuiltinFun,
    pub cmd: String,
    pub help: String,
}

impl Builtin {
    pub fn new(fun: BuiltinFun, cmd: &str, help: &str) -> Builtin {
        Builtin {
            fun: fun,
            cmd: String::from(cmd),
            help: String::from(help),
        }
    }
}

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builtin")
            .field("cmd", &self.cmd)
            .field("help", &self.help)
            .finish()
    }
}

// ********** builtin commands **********

pub fn do_help(state: &mut State, _tokens: Vec<&str>) {
    state.status = 0;
    for b in &state.builtins {
        println!("{:16}{}", b.cmd, b.help);
    }
}

pub fn do_name(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    if tokens.len() > 1 {
        state.name = String::from(tokens[1]);
    } else {
        println!("{}", state.name);
    }
}

pub fn do_debug(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    if tokens.len() > 1 {
        state.debug = tokens[1] == "on";
    }
    println!("Debug is {}", if state.debug { "on" } else { "off" });
}

pub fn do_status(state: &mut State, _tokens: Vec<&str>) {
    println!("{}", state.status);
    state.status = 0;
}

pub fn do_print(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    for (i, t) in tokens.iter().enumerate() {
        if i == 0 {
            continue;
        };
        print!("{}", t);
        if i < tokens.len() {
            print!(" ");
        }
    }
}

pub fn do_echo(state: &mut State, tokens: Vec<&str>) {
    do_print(state, tokens);
    println!("");
}

pub fn do_pid(state: &mut State, _tokens: Vec<&str>) {
    state.status = 0;
    println!("{}", unistd::getpid());
}

pub fn do_ppid(state: &mut State, _tokens: Vec<&str>) {
    state.status = 0;
    println!("{}", unistd::getppid());
}

pub fn do_exit(state: &mut State, tokens: Vec<&str>) {
    if tokens.len() > 1 {
        state.status = tokens[1].parse::<i32>().unwrap();
    }
    state.running = false;
}

pub fn do_dir_change(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    let path = if tokens.len() == 1 { "/" } else { tokens[1] };
    if let Err(err) = env::set_current_dir(&path) {
        // if let Err(_) = unistd::chdir(path) {
        handle_ioerror(state, &err);
    }
}

pub fn do_dir_where(state: &mut State, _tokens: Vec<&str>) {
    state.status = 0;
    match env::current_dir() {
        Ok(path) => println!("{}", path.display()),
        Err(err) => handle_ioerror(state, &err),
    };
}

pub fn do_dir_make(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    for t in &tokens[1..] {
        // let path = std::path::PathBuf::from(t);
        // if let Err(e) = nix::unistd::mkdir(&path, nix::sys::stat::Mode::S_IRWXU) {
        if let Err(err) = fs::create_dir(t) {
            handle_ioerror(state, &err);
        }
    }
}

pub fn do_dir_remove(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    for t in &tokens[1..] {
        if let Err(err) = fs::remove_dir(t) {
            handle_ioerror(state, &err)
        }
    }
}

fn get_dir_entries(tokens: Vec<&str>) -> std::io::Result<fs::ReadDir> {
    let path = if tokens.len() > 1 {
        std::path::PathBuf::from(tokens[1])
    } else {
        env::current_dir()?
    };
    fs::read_dir(path)
}

pub fn do_dir_list(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    match get_dir_entries(tokens) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    print!("{}  ", entry.file_name().to_str().unwrap())
                }
            }
            println!();
        }
        Err(err) => handle_ioerror(state, &err),
    }
}

pub fn do_dir_inspect(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    match get_dir_entries(tokens) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        println!(
                            "{:?} {}  ",
                            metadata.len(),
                            entry.file_name().to_str().unwrap()
                        )
                    }
                }
            }
        }
        Err(err) => handle_ioerror(state, &err),
    }
}

pub fn do_link_hard(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    if let Err(err) = fs::hard_link(tokens[1], tokens[2]) {
        handle_ioerror(state, &err);
    }
}

pub fn do_link_soft(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    //    fs:: soft_link(tokens[1], tokens[2])
    if let Err(err) = std::os::unix::fs::symlink(tokens[1], tokens[2]) {
        handle_ioerror(state, &err);
    }
}

pub fn do_link_read(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    for t in &tokens[1..] {
        match fs::read_link(&t) {
            Ok(path) => println!("{}", path.display()),
            Err(err) => handle_ioerror(state, &err),
        }
    }
}

pub fn do_unlink(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    for t in &tokens[1..] {
        if let Err(err) = fs::remove_file(t) {
            handle_ioerror(state, &err)
        }
    }
}

pub fn do_rename(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;
    if let Err(err) = fs::rename(tokens[1], tokens[2]) {
        handle_ioerror(state, &err)
    }
}

pub fn do_cpcat(state: &mut State, tokens: Vec<&str>) {
    state.status = 0;

    let mut fin: Box<dyn io::Read + 'static> = if tokens[1] == "-" {
        Box::new(io::stdin())
    } else {
        match fs::OpenOptions::new().read(true).open(tokens[1]) {
            Ok(file) => Box::new(file),
            Err(err) => {
                handle_ioerror(state, &err);
                return;
            }
        }
    };

    let mut fout: Box<dyn io::Write + 'static> = if tokens[2] == "-" {
        Box::new(io::stdout())
    } else {
        match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(tokens[2])
        {
            Ok(file) => Box::new(file),
            Err(err) => {
                handle_ioerror(state, &err);
                return;
            }
        }
    };
    let mut buf = [0; 4096];
    while let Ok(count) = fin.read(&mut buf) {
        if count == 0 {
            break;
        }
        fout.write(&buf[0..count]);
    }
}

// ********** default builtins **********

pub fn default_builtins() -> Vec<Builtin> {
    vec![
        Builtin::new(do_help, "help", "Print short help"),
        Builtin::new(do_name, "name", "Print or change shell name"),
        Builtin::new(do_debug, "debug", "Print or change debug mode"),
        Builtin::new(do_status, "status", "Print status of the last command"),
        Builtin::new(do_print, "print", "Print arguments"),
        Builtin::new(do_echo, "echo", "Print arguments and the newline"),
        Builtin::new(do_pid, "pid", "Print PID"),
        Builtin::new(do_ppid, "ppid", "Print PPID"),
        Builtin::new(do_exit, "exit", "Exit from the shell"),
        Builtin::new(do_dir_change, "dir.change", "Change current directory"),
        Builtin::new(do_dir_where, "dir.where", "Print current working directory"),
        Builtin::new(do_dir_make, "dir.make", "Make directories"),
        Builtin::new(do_dir_remove, "dir.remove", "Remove directories"),
        Builtin::new(do_dir_list, "dir.list", "List directory"),
        Builtin::new(do_dir_inspect, "dir.inspect", "Inspect directory"),
        Builtin::new(do_link_hard, "link.hard", "Create hard link"),
        Builtin::new(do_link_soft, "link.soft", "Create symbolic/soft link"),
        Builtin::new(do_link_read, "link.read", "Print symbolic link target"),
        Builtin::new(do_unlink, "unlink", "Unlink files"),
        Builtin::new(do_rename, "rename", "Rename file"),
        Builtin::new(do_cpcat, "cpcat", "Copy file"),
    ]
}
