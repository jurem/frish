use nix::fcntl;
use nix::sys::stat;
use nix::unistd;

use std::fmt;
use std::fs; // portable FS functions
use std::io;

use crate::common::{handle_ioerror, handle_nixerror, State};

type BuiltinFun = fn(&mut State, &[&str]);

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

pub fn do_help(state: &mut State, _args: &[&str]) {
    state.status = 0;
    // for b in &state.builtins {
    //     println!("{:16}{}", b.cmd, b.help);
    // }
}

pub fn do_name(state: &mut State, args: &[&str]) {
    state.status = 0;
    if args.len() > 1 {
        state.name = String::from(args[1]);
    } else {
        println!("{}", state.name);
    }
}

pub fn do_debug(state: &mut State, args: &[&str]) {
    state.status = 0;
    if args.len() > 1 {
        state.debug = args[1] == "on";
    }
    println!("Debug is {}", if state.debug { "on" } else { "off" });
}

pub fn do_status(state: &mut State, _args: &[&str]) {
    println!("{}", state.status);
    state.status = 0;
}

pub fn do_print(state: &mut State, args: &[&str]) {
    state.status = 0;
    let last = args.last().unwrap();
    for arg in args.iter().skip(1) {
        print!("{}", arg);
        if arg != last {
            print!(" ");
        };
    }
}

pub fn do_echo(state: &mut State, args: &[&str]) {
    do_print(state, args);
    println!("");
}

pub fn do_pid(state: &mut State, _args: &[&str]) {
    state.status = 0;
    println!("{}", unistd::getpid());
}

pub fn do_ppid(state: &mut State, _args: &[&str]) {
    state.status = 0;
    println!("{}", unistd::getppid());
}

pub fn do_exit(state: &mut State, args: &[&str]) {
    if args.len() > 1 {
        state.status = args[1].parse::<i32>().unwrap_or(0);
    }
    state.running = false;
}

pub fn do_dir_change(state: &mut State, args: &[&str]) {
    state.status = 0;
    let path = if args.len() == 1 { "/" } else { args[1] };
    if let Err(err) = unistd::chdir(path) {
        handle_nixerror(state, &err);
    }
}

pub fn do_dir_where(state: &mut State, _args: &[&str]) {
    state.status = 0;
    match unistd::getcwd() {
        Ok(path) => println!("{}", path.display()),
        Err(err) => handle_nixerror(state, &err),
    };
}

pub fn do_dir_make(state: &mut State, args: &[&str]) {
    state.status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = unistd::mkdir(&path, stat::Mode::S_IRWXU) {
            handle_nixerror(state, &err);
        }
    }
}

use nix::NixPath;

fn rmdir<P: ?Sized + NixPath>(path: &P) -> nix::Result<()> {
    let res = path.with_nix_path(|cstr| unsafe { libc::rmdir(cstr.as_ptr()) })?;
    nix::errno::Errno::result(res).map(drop)
}

pub fn do_dir_remove(state: &mut State, args: &[&str]) {
    state.status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = rmdir(&path) {
            handle_nixerror(state, &err)
        }
    }
}

fn get_dir_entries(args: &[&str]) -> std::io::Result<fs::ReadDir> {
    let path = if args.len() > 1 {
        std::path::PathBuf::from(args[1])
    } else {
        unistd::getcwd()?
    };
    fs::read_dir(path)
}

pub fn do_dir_list(state: &mut State, args: &[&str]) {
    state.status = 0;
    match get_dir_entries(args) {
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

pub fn do_dir_inspect(state: &mut State, args: &[&str]) {
    state.status = 0;
    match get_dir_entries(args) {
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

pub fn do_link_hard(state: &mut State, args: &[&str]) {
    state.status = 0;
    if let Err(err) = unistd::linkat(
        None,
        args[1],
        None,
        args[2],
        unistd::LinkatFlags::NoSymlinkFollow,
    ) {
        handle_nixerror(state, &err);
    }
}

pub fn do_link_soft(state: &mut State, args: &[&str]) {
    state.status = 0;
    if let Err(err) = unistd::symlinkat(args[1], None, args[2]) {
        handle_nixerror(state, &err);
    }
}

pub fn do_link_read(state: &mut State, args: &[&str]) {
    state.status = 0;
    for arg in &args[1..] {
        let arg = std::path::PathBuf::from(arg);
        match fcntl::readlink(&arg) {
            Ok(path) => println!("{}", path.to_str().unwrap()),
            Err(err) => handle_nixerror(state, &err),
        }
    }
}

pub fn do_unlink(state: &mut State, args: &[&str]) {
    state.status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = unistd::unlink(&path) {
            handle_nixerror(state, &err)
        }
    }
}

pub fn do_rename(state: &mut State, args: &[&str]) {
    state.status = 0;
    if let Err(err) = fcntl::renameat(None, args[1], None, args[2]) {
        handle_nixerror(state, &err)
    }
}

// pub fn do_cpcat1(state: &mut State, args: &[&str]) {
//     state.status = 0;
//     let fin = if args[1] == "-" {
//         0
//     } else {
//         match fcntl::open(args[1], fcntl::OFlag::O_RDONLY, stat::Mode::S_IRWXU) {
//             Ok(fd) => fd,
//             Err(err) => {
//                 handle_nixerror(state, &err);
//                 return;
//             }
//         }
//     };
// }

pub fn do_cpcat(state: &mut State, args: &[&str]) {
    state.status = 0;

    let mut fin: Box<dyn io::Read + 'static> = if args[1] == "-" {
        Box::new(io::stdin())
    } else {
        match fs::OpenOptions::new().read(true).open(args[1]) {
            Ok(file) => Box::new(file),
            Err(err) => {
                handle_ioerror(state, &err);
                return;
            }
        }
    };

    let mut fout: Box<dyn io::Write + 'static> = if args[2] == "-" {
        Box::new(io::stdout())
    } else {
        match fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(args[2])
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
        if fout.write(&buf[0..count]).is_err() {
            break;
        }
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

pub fn find_builtin<'a>(builtins: &'a [Builtin], name: &str) -> Option<&'a Builtin> {
    builtins.iter().find(|&b| b.cmd == name)
}
