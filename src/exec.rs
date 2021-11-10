use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::sys::wait::waitpid;
use nix::unistd::{close, dup, dup2, execvp, fork, ForkResult, Pid};
use std::os::unix::io::RawFd;

use crate::builtins::{find_builtin, Builtin};
use crate::common::State;
use crate::common::{debug, handle_ioerror};
use crate::parser;

fn wait_process(state: &mut State, pid: Pid) -> i32 {
    let msg = format!("Waiting for {}.\n", pid);
    debug(state, &msg);
    match waitpid(pid, None) {
        Err(_) => 255,
        Ok(_) => 0, // Ok(status) => match status {
                    //     WaitStatus::Exited(pid, st) => st
                    // }
                    // Some(satus) => {
                    //     if (WIFEXITED(status))
                    //     return WEXITSTATUS(status);
                    // else if (WIFSIGNALED(status))
                    //     return WTERMSIG(status);
                    // return -1;
    }
}

fn redirect_in(infile: &str) -> Option<RawFd> {
    if infile.is_empty() {
        None
    } else {
        let fdin = open(infile, OFlag::O_RDONLY, Mode::S_IRWXU).unwrap();
        let fdinold = dup(0).unwrap();
        dup2(fdin, 0).unwrap();
        close(fdin).unwrap();
        Some(fdinold)
    }
}

fn restore_in(fdinold: Option<RawFd>) {
    if let Some(fdinold) = fdinold {
        dup2(fdinold, 0).unwrap();
        close(fdinold).unwrap();
    }
}

fn exec_external(prog: &str, args: &[&str]) {
    let prog = std::ffi::CString::new(prog).unwrap();
    let args = args
        .into_iter()
        .map(|&arg| std::ffi::CString::new(arg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // let args = args. into_iter().map(|arg| arg.as_ptr()).collect();
    match execvp(prog.as_ref(), args.as_ref()) {
        Err(_) => eprintln!("Invalid command"),
        Ok(_) => {}
    }
}
// let res = path.with_nix_path(|cstr| {
//     unsafe { libc::chdir(cstr.as_ptr()) }
// })?;

pub fn run_external(state: &mut State, args: &[&str]) {
    debug(state, "Executing external command");
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            if !state.background {
                state.status = wait_process(state, child);
            }
        }
        Ok(ForkResult::Child) => {
            exec_external(args[0], args);
            std::process::exit(127);
        }
        Err(err) => {
            let err = std::io::Error::from(err);
            handle_ioerror(state, &err);
        }
    }
}

pub fn run_builtin(builtin: &Builtin, state: &mut State, args: &[&str]) {
    (builtin.fun)(state, args);
}

pub fn forkit(_: &mut State) -> Pid {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => child,
        Ok(ForkResult::Child) => Pid::from_raw(0),
        Err(_) => {
            // handle_error(state, err);
            // exit(state.status);
            Pid::from_raw(-1)
        }
    }
}
