use log::{debug, info};
use nix::{
    sys::wait::{waitpid, WaitStatus},
    unistd::{execvp, fork, ForkResult, Pid},
};
use std::{convert::Infallible, io, io::Write};

use crate::builtins::Builtin;
use crate::common::{Command, State, Status};
use crate::shell::redirect::{redirect_stdin, redirect_stdout, restore_stdin, restore_stdout};

pub fn print_prompt(state: &State) {
    if state.interactive {
        print!("{}> ", state.name.borrow());
        io::stdout().flush().expect("Cannot flush stdout");
    }
}

fn wait_process(pid: Pid) -> io::Result<Status> {
    debug!("Waiting for {}.\n", pid);
    match waitpid(pid, None)? {
        WaitStatus::Exited(_, code) => Ok(Status::from_code(code)),
        _ => Err(io::Error::last_os_error()),
    }
}

fn fork_child_wait<F: FnMut()>(child: &mut F) -> io::Result<Status> {
    match unsafe { fork()? } {
        ForkResult::Parent { child } => wait_process(child),
        ForkResult::Child => {
            child();
            std::process::exit(127);
        }
    }
}

fn fork_child<F: FnMut()>(child: &mut F) -> io::Result<Pid> {
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
            execvp(cprog, &args).unwrap();
            //cmd.args[0].with_nix_path(|cprog| execvp(cprog, args.as_ref().unwrap()))
            Err(std::io::Error::last_os_error())
        })
}

fn run_external(cmd: &Command) -> io::Result<Status> {
    debug!("Running external command");
    if cmd.background {
        fork_child(&mut || {
            exec_external(cmd).unwrap();
        })
        .unwrap();
        Ok(Status::success())
    } else {
        fork_child_wait(&mut || {
            exec_external(cmd).unwrap();
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

fn run_builtin(builtin: &Builtin, state: &State, cmd: &Command) -> io::Result<Status> {
    info!("Running builtin");
    if cmd.background {
        state.lastpid.set(fork_child(&mut || {
            exec_builtin(state, builtin, cmd).unwrap();
        })?);
        Ok(Status::success())
    } else {
        exec_builtin(state, builtin, cmd)
    }
}
