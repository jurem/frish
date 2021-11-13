use nix::{fcntl, unistd};
use std::fs;
use std::io; // portable FS functions, TODO: use only nix::*

use crate::common::{report_nixerror, State, Status};

pub fn do_link_hard(_: &State, args: &[&str]) -> io::Result<Status> {
    unistd::linkat(
        None,
        args[1],
        None,
        args[2],
        unistd::LinkatFlags::NoSymlinkFollow,
    )?;
    Ok(Status::success())
}

pub fn do_link_soft(_: &State, args: &[&str]) -> io::Result<Status> {
    unistd::symlinkat(args[1], None, args[2])?;
    Ok(Status::success())
}

pub fn do_link_read(_: &State, args: &[&str]) -> io::Result<Status> {
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
    Ok(Status::from_code(status))
}

pub fn do_unlink(_: &State, args: &[&str]) -> io::Result<Status> {
    let mut status = 0;
    for arg in &args[1..] {
        let path = std::path::PathBuf::from(arg);
        if let Err(err) = unistd::unlink(&path) {
            report_nixerror(&err);
            status = nix::errno::errno();
        }
    }
    Ok(Status::from_code(status))
}

pub fn do_rename(_: &State, args: &[&str]) -> io::Result<Status> {
    fcntl::renameat(None, args[1], None, args[2])?;
    Ok(Status::success())
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

pub fn do_cpcat(_: &State, args: &[&str]) -> io::Result<Status> {
    if args.len() < 3 {
        return Ok(Status::from_code(127));
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
    Ok(Status::success())
}
