use nix::{sys::stat, unistd, NixPath};
use std::fs; // portable FS functions, TODO: use only nix::*
use std::io;

use crate::common::report_nixerror;
use crate::state::{State, Status};

pub fn do_dir_change(_: &State, args: &[&str]) -> io::Result<Status> {
    let path = if args.len() == 1 { "/" } else { args[1] };
    unistd::chdir(path)?;
    Ok(Status::success())
}

pub fn do_dir_where(_: &State, _args: &[&str]) -> io::Result<Status> {
    let path = unistd::getcwd()?;
    println!("{}", path.display());
    Ok(Status::success())
}

pub fn do_dir_make(_: &State, args: &[&str]) -> io::Result<Status> {
    let mut status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = unistd::mkdir(&path, stat::Mode::S_IRWXU) {
            report_nixerror(&err);
            status = nix::errno::errno();
        }
    }
    Ok(Status::from_code(status))
}

// missing in libc, used libc::mkdir as an example
#[inline]
fn rmdir<P: ?Sized + NixPath>(path: &P) -> nix::Result<()> {
    let res = path.with_nix_path(|cstr| unsafe { libc::rmdir(cstr.as_ptr()) })?;
    nix::errno::Errno::result(res).map(drop)
}

pub fn do_dir_remove(_: &State, args: &[&str]) -> io::Result<Status> {
    let mut status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = rmdir(&path) {
            report_nixerror(&err);
            status = nix::errno::errno();
        }
    }
    Ok(Status::from_code(status))
}

fn get_dir_entries(args: &[&str]) -> std::io::Result<fs::ReadDir> {
    let path = if args.len() > 1 {
        std::path::PathBuf::from(args[1])
    } else {
        unistd::getcwd()?
    };
    fs::read_dir(path)
}

pub fn do_dir_list(_: &State, args: &[&str]) -> io::Result<Status> {
    let entries = get_dir_entries(args)?;
    for entry in entries {
        if let Ok(entry) = entry {
            print!("{}  ", entry.file_name().to_str().unwrap())
        }
    }
    println!();
    Ok(Status::success())
}

pub fn do_dir_inspect(_: &State, args: &[&str]) -> io::Result<Status> {
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
    Ok(Status::success())
}
