use log::{debug, info};
use nix::{
    sys::wait::{waitpid, WaitStatus},
    unistd::{execvp, fork, ForkResult, Pid},
};
use std::{convert::Infallible, io, io::Write};

use crate::builtins::Builtin;
use crate::common::report_error;
use crate::common::Command;
use crate::shell::redirect::{redirect_stdin, redirect_stdout, restore_stdin, restore_stdout};
use crate::state::{State, Status};

pub fn print_prompt(state: &State) {
    if state.interactive {
        print!("{}> ", state.name.borrow());
        io::stdout().flush().expect("Cannot flush stdout");
    }
}

pub fn wait_process(pid: Pid) -> io::Result<Status> {
    debug!("Waiting for {}.\n", pid);
    match waitpid(pid, None)? {
        WaitStatus::Exited(_, code) => Ok(Status::from_code(code)),
        _ => Err(io::Error::last_os_error()),
    }
}

pub fn fork_child_wait<F: FnMut()>(child: &mut F) -> io::Result<Status> {
    match unsafe { fork()? } {
        ForkResult::Parent { child } => wait_process(child),
        ForkResult::Child => {
            child();
            std::process::exit(127);
        }
    }
}

pub fn fork_child<F: FnMut()>(child: &mut F) -> io::Result<Pid> {
    match unsafe { fork()? } {
        ForkResult::Parent { child } => Ok(child),
        ForkResult::Child => {
            child();
            std::process::exit(0);
        }
    }
}

fn exec_external(cmd: &Command) -> Result<Infallible, io::Error> {
    redirect_stdin(cmd.inredirect)
        .and_then(|_| redirect_stdout(cmd.outredirect))
        .and_then(|_| {
            let args = &cmd.args;
            let args = args
                .into_iter()
                .map(|&arg| std::ffi::CString::new(arg))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let cprog = &args[0];
            //cmd.args[0].with_nix_path(|cprog| execvp(cprog, args.as_ref().unwrap()))
            execvp(cprog, &args);
            Err(std::io::Error::last_os_error())
        })
}

pub fn run_external(state: &State, cmd: &Command) -> io::Result<Status> {
    debug!("Running external command: '{}'", cmd.args[0]);
    if cmd.background {
        state.lastpid.set(
            fork_child(&mut || {
                if let Err(err) = exec_external(cmd) {
                    report_error(&err);
                }
            })
            .unwrap(),
        );
        Ok(Status::success())
    } else {
        fork_child_wait(&mut || {
            if let Err(err) = exec_external(cmd) {
                report_error(&err);
            }
        })
    }
}

fn exec_builtin(state: &State, builtin: &Builtin, cmd: &Command) -> io::Result<Status> {
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

pub fn run_builtin(builtin: &Builtin, state: &State, cmd: &Command) -> io::Result<Status> {
    info!("Running builtin command: '{}'", cmd.args[0]);
    if cmd.background {
        state.lastpid.set(fork_child(&mut || {
            exec_builtin(state, builtin, cmd).unwrap();
        })?);
        Ok(Status::success())
    } else {
        exec_builtin(state, builtin, cmd)
    }
}
