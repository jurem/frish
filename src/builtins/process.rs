use nix::unistd;
use std::io;

use crate::shell::{
    eval::{eval, read_eval_loop},
    exec::wait_process,
    pipes::{pipes_begin, pipes_cont, pipes_end},
};
use crate::state::{State, Status};

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

pub fn do_pipes(state: &State, args: &[&str]) -> io::Result<Status> {
    let mut fds2 = pipes_begin(state, args[1])?;
    for i in 2..args.len() - 1 {
        let fds1 = fds2;
        fds2 = pipes_cont(state, args[i], fds1)?;
    }
    let pid = pipes_end(state, args[args.len() - 1], fds2)?;
    wait_process(pid)?;
    Ok(Status::success())
}
