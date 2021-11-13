use nix::unistd;
use std::io;

use crate::common::{State, Status};
use crate::exec::{eval, read_eval_loop};

pub fn do_status(state: &State, _args: &[&str]) -> io::Result<Status> {
    println!("{}", state.status.get());
    Ok(Status::success())
}

pub fn do_pid(_: &State, _args: &[&str]) -> io::Result<Status> {
    println!("{}", unistd::getpid());
    Ok(Status::success())
}

pub fn do_ppid(_: &State, _args: &[&str]) -> io::Result<Status> {
    println!("{}", unistd::getppid());
    Ok(Status::success())
}

pub fn do_exit(state: &State, args: &[&str]) -> io::Result<Status> {
    state.terminate();
    let status = if args.len() > 1 {
        args[1].parse::<i32>().unwrap_or(0)
    } else {
        0
    };
    Ok(Status::from_code(status))
}

pub fn do_depth(state: &State, _: &[&str]) -> io::Result<Status> {
    println!("{}", state.depth);
    Ok(Status::success())
}

pub fn do_subshell(state: &State, args: &[&str]) -> io::Result<Status> {
    let state = state.sub();
    if args.len() > 1 {
        eval(&state, args[1]);
    } else {
        read_eval_loop(&state);
    }
    Ok(Status::from(&state.status.get()))
}
