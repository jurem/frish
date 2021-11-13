use nix::unistd::{close, dup, dup2, pipe, Pid};
use std::{io, os::unix::io::RawFd};

use crate::common::State;
use crate::shell::eval;
use crate::shell::exec::fork_child;

pub fn pipes_begin(state: &State, cmdstr: &str) -> io::Result<(RawFd, RawFd)> {
    let fds = pipe()?;
    fork_child(&mut || {
        dup2(fds.1, 1).unwrap();
        close(fds.0).unwrap();
        close(fds.1).unwrap();
        eval(state, cmdstr);
    })?;
    Ok(fds)
}

pub fn pipes_cont(state: &State, cmdstr: &str, fds1: (RawFd, RawFd)) -> io::Result<(RawFd, RawFd)> {
    let fds2 = pipe()?;
    fork_child(&mut || {
        dup2(fds1.0, 0).unwrap();
        dup2(fds2.1, 1).unwrap();
        close(fds1.0).unwrap();
        close(fds1.1).unwrap();
        close(fds2.0).unwrap();
        close(fds2.1).unwrap();
        eval(state, cmdstr);
    })?;
    close(fds1.0).unwrap();
    close(fds1.1).unwrap();
    Ok(fds2)
}

pub fn pipes_end(state: &State, cmdstr: &str, fds: (RawFd, RawFd)) -> io::Result<Pid> {
    let res = fork_child(&mut || {
        dup2(fds.0, 0).unwrap();
        close(fds.0).unwrap();
        close(fds.1).unwrap();
        eval(state, cmdstr);
    });
    close(fds.0).unwrap();
    close(fds.1).unwrap();
    res
}
