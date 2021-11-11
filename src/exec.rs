use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::sys::wait::waitpid;
use nix::unistd::{close, dup, dup2, execvp, fork, ForkResult, Pid};
use nix::NixPath;
use std::os::unix::io::RawFd;

use std::io;

use crate::builtins::Builtin;
use crate::common::debug;
use crate::common::{Command, State};

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

fn redirect_stdin(infile: Option<&str>) -> io::Result<Option<RawFd>> {
    if let Some(infile) = infile {
        crate::log!("Redirecting stdin to {}", infile);
        let fd = open(infile, OFlag::O_RDONLY, Mode::empty())?;
        let fdold = dup(0).unwrap();
        dup2(fd, 0).unwrap();
        close(fd).unwrap();
        Ok(Some(fdold))
    } else {
        Ok(None)
    }
}

fn restore_stdin(fdinold: Option<RawFd>) {
    if let Some(fdinold) = fdinold {
        dup2(fdinold, 0).unwrap();
        close(fdinold).unwrap();
    }
}

fn redirect_stdout(outfile: Option<&str>) -> io::Result<Option<RawFd>> {
    if let Some(outfile) = outfile {
        crate::log!("Redirecting stdout to {}", outfile);
        let flag = OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC;
        let mode = Mode::S_IRUSR | Mode::S_IWUSR;
        let fd = open(outfile, flag, mode)?;
        let fdold = dup(0).unwrap();
        dup2(fd, 1).unwrap();
        close(fd).unwrap();
        Ok(Some(fdold))
    } else {
        Ok(None)
    }
}

fn restore_stdout(fdold: Option<RawFd>) {
    if let Some(fdold) = fdold {
        dup2(fdold, 1).unwrap();
        close(fdold).unwrap();
    }
}

fn exec_external(prog: &str, args: &[&str]) {
    let args = args
        .into_iter()
        .map(|&arg| std::ffi::CString::new(arg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // let args = args. into_iter().map(|arg| arg.as_ptr()).collect();
    prog.with_nix_path(|cprog| match execvp(cprog, args.as_ref()) {
        Err(_) => eprintln!("Invalid command {}", prog),
        Ok(_) => {}
    })
    .unwrap();
}

pub fn run_external(state: &mut State, cmd: Command) -> io::Result<i32> {
    debug(state, "Executing external command");
    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            if !cmd.background {
                state.status = wait_process(state, child);
            }
        }
        ForkResult::Child => {
            exec_external(cmd.args[0], &cmd.args);
            std::process::exit(127);
        }
    }
    Ok(0) //TODO
}

pub fn run_builtin(builtin: &Builtin, state: &mut State, cmd: Command) -> io::Result<i32> {
    debug(state, "Running builtin");
    if cmd.background {
        let status = (builtin.handler)(state, &cmd.args)?;
        Ok(status)
    } else {
        let fdinold = redirect_stdin(cmd.inredirect)?;
        let fdoutold = redirect_stdout(cmd.outredirect).or_else(|err| {
            restore_stdin(fdinold);
            Err(err)
        })?;
        let status = (builtin.handler)(state, &cmd.args)?;
        restore_stdin(fdinold);
        restore_stdout(fdoutold);
        Ok(status)
    }
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
