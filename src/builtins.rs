use nix::fcntl;
use nix::sys::stat;
use nix::unistd;
use nix::NixPath;

// TODO: eliminate these crates by using only nix::*
use std::fmt;
use std::fs; // portable FS functions
use std::io;

use crate::common::{report_nixerror, State};

type BuiltinHandler = fn(&mut State, &[&str]) -> io::Result<i32>;

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

// ********** builtin commands **********

pub fn do_help(_: &mut State, _args: &[&str]) -> io::Result<i32> {
    // for b in &state.builtins {
    //     println!("{:16}{}", b.cmd, b.help);
    // }
    Ok(0)
}

pub fn do_name(state: &mut State, args: &[&str]) -> io::Result<i32> {
    if args.len() > 1 {
        state.name = String::from(args[1]);
    } else {
        println!("{}", state.name);
    }
    Ok(0)
}

pub fn do_debug(state: &mut State, args: &[&str]) -> io::Result<i32> {
    if args.len() > 1 {
        state.debug = args[1] == "on";
    }
    println!("Debug is {}", if state.debug { "on" } else { "off" });
    Ok(0)
}

pub fn do_status(state: &mut State, _args: &[&str]) -> io::Result<i32> {
    println!("{}", state.status);
    Ok(0)
}

pub fn do_print(_: &mut State, args: &[&str]) -> io::Result<i32> {
    let last = args.last().unwrap();
    for arg in args.iter().skip(1) {
        print!("{}", arg);
        if arg != last {
            print!(" ");
        };
    }
    Ok(0)
}

pub fn do_echo(state: &mut State, args: &[&str]) -> io::Result<i32> {
    do_print(state, args).and_then(|_| {
        println!("");
        Ok(0)
    })
}

pub fn do_pid(_: &mut State, _args: &[&str]) -> io::Result<i32> {
    println!("{}", unistd::getpid());
    Ok(0)
}

pub fn do_ppid(_: &mut State, _args: &[&str]) -> io::Result<i32> {
    println!("{}", unistd::getppid());
    Ok(0)
}

pub fn do_exit(state: &mut State, args: &[&str]) -> io::Result<i32> {
    state.running = false;
    let status = if args.len() > 1 {
        args[1].parse::<i32>().unwrap_or(0)
    } else {
        0
    };
    Ok(status)
}

pub fn do_dir_change(_: &mut State, args: &[&str]) -> io::Result<i32> {
    let path = if args.len() == 1 { "/" } else { args[1] };
    unistd::chdir(path)?;
    Ok(0)
}

pub fn do_dir_where(_: &mut State, _args: &[&str]) -> io::Result<i32> {
    let path = unistd::getcwd()?;
    println!("{}", path.display());
    Ok(0)
}

pub fn do_dir_make(_: &mut State, args: &[&str]) -> io::Result<i32> {
    let mut status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = unistd::mkdir(&path, stat::Mode::S_IRWXU) {
            report_nixerror(&err);
            status = nix::errno::errno();
        }
    }
    Ok(status)
}

// missing in libc, used libc::mkdir as an example
#[inline]
fn rmdir<P: ?Sized + NixPath>(path: &P) -> nix::Result<()> {
    let res = path.with_nix_path(|cstr| unsafe { libc::rmdir(cstr.as_ptr()) })?;
    nix::errno::Errno::result(res).map(drop)
}

pub fn do_dir_remove(_: &mut State, args: &[&str]) -> io::Result<i32> {
    let mut status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = rmdir(&path) {
            report_nixerror(&err);
            status = nix::errno::errno();
        }
    }
    Ok(status)
}

fn get_dir_entries(args: &[&str]) -> std::io::Result<fs::ReadDir> {
    let path = if args.len() > 1 {
        std::path::PathBuf::from(args[1])
    } else {
        unistd::getcwd()?
    };
    fs::read_dir(path)
}

pub fn do_dir_list(_: &mut State, args: &[&str]) -> io::Result<i32> {
    let entries = get_dir_entries(args)?;
    for entry in entries {
        if let Ok(entry) = entry {
            print!("{}  ", entry.file_name().to_str().unwrap())
        }
    }
    println!();
    Ok(0)
}

pub fn do_dir_inspect(_: &mut State, args: &[&str]) -> io::Result<i32> {
    let entries = get_dir_entries(args)?;
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
    Ok(0)
}

pub fn do_link_hard(_: &mut State, args: &[&str]) -> io::Result<i32> {
    unistd::linkat(
        None,
        args[1],
        None,
        args[2],
        unistd::LinkatFlags::NoSymlinkFollow,
    )?;
    Ok(0)
}

pub fn do_link_soft(_: &mut State, args: &[&str]) -> io::Result<i32> {
    unistd::symlinkat(args[1], None, args[2])?;
    Ok(0)
}

pub fn do_link_read(_: &mut State, args: &[&str]) -> io::Result<i32> {
    let mut status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        match fcntl::readlink(&path) {
            Ok(path) => println!("{}", path.to_str().unwrap()),
            Err(err) => {
                report_nixerror(&err);
                status = nix::errno::errno();
            }
        }
    }
    Ok(status)
}

pub fn do_unlink(_: &mut State, args: &[&str]) -> io::Result<i32> {
    let mut status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = unistd::unlink(&path) {
            report_nixerror(&err);
            status = nix::errno::errno();
        }
    }
    Ok(status)
}

pub fn do_rename(_: &mut State, args: &[&str]) -> io::Result<i32> {
    fcntl::renameat(None, args[1], None, args[2])?;
    Ok(0)
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

use std::io::Read;

pub fn do_cpcat(_: &mut State, args: &[&str]) -> io::Result<i32> {
    if args.len() < 3 {
        return Ok(1);
    }
    // open input
    let mut fin: Box<dyn io::Read> = if args[1] == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(fs::OpenOptions::new().read(true).open(args[1])?)
    };
    // open output
    let mut fout: Box<dyn io::Write> = if args[2] == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(args[2])?,
        )
    };
    // do the copy
    let mut buf = [0; 4096];
    loop {
        let count = fin.read(&mut buf)?;
        if count == 0 {
            break;
        }
        fout.write(&buf[0..count])?;
    }
    Ok(0)
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
    builtins.iter().find(|&b| b.command == name)
}
