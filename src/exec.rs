use nix::{
    fcntl::{open, OFlag},
    sys::{
        stat::Mode,
        wait::{waitpid, WaitStatus},
    },
    unistd::{close, dup, dup2, execvp, fork, ForkResult, Pid},
};
use std::{convert::Infallible, io, io::Write, os::unix::io::RawFd};

use crate::builtins::Builtin;
use crate::common::report_error;
use crate::common::{Command, State, Status};
use crate::parser;

use log::{debug, info};

fn print_prompt(state: &State) {
    if state.interactive {
        print!("{}> ", state.name.borrow());
        io::stdout().flush().expect("Cannot flush stdout");
    }
}

fn redirect_stdin(infile: Option<&str>) -> io::Result<Option<RawFd>> {
    if let Some(infile) = infile {
        debug!("Redirecting stdin to {}", infile);
        let fd = open(infile, OFlag::O_RDONLY, Mode::empty())?;
        let fdold = dup(0).unwrap();
        dup2(fd, 0).unwrap();
        close(fd).unwrap();
        Ok(Some(fdold))
    } else {
        Ok(None)
    }
}

fn restore_stdin(fdold: Option<RawFd>) {
    if let Some(fdold) = fdold {
        debug!("Restoring stdin");
        dup2(fdold, 0).unwrap();
        close(fdold).unwrap();
    }
}

fn redirect_stdout(outfile: Option<&str>) -> io::Result<Option<RawFd>> {
    if let Some(outfile) = outfile {
        debug!("Redirecting stdout to {}", outfile);
        let flag = OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC;
        let mode = Mode::S_IRUSR | Mode::S_IWUSR;
        let fd = open(outfile, flag, mode)?;
        let fdold = dup(1).unwrap();
        dup2(fd, 1).unwrap();
        close(fd).unwrap();
        Ok(Some(fdold))
    } else {
        Ok(None)
    }
}

fn restore_stdout(fdold: Option<RawFd>) {
    if let Some(fdold) = fdold {
        debug!("Restoring stdout");
        dup2(fdold, 1).unwrap();
        close(fdold).unwrap();
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

pub fn eval(state: &State, cmdstr: &str) {
    if let Some(cmd) = parser::parse(&cmdstr) {
        let res = match state.find_builtin(cmd.args[0]) {
            Some(builtin) => run_builtin(builtin, state, &cmd),
            None => run_external(&cmd),
        };
        match res {
            Ok(status) => state.set_status(&status),
            Err(err) => {
                state.set_status_code(nix::errno::errno());
                report_error(&err);
            }
        }
    } else {
        debug!("No command given.");
    }
}

pub fn read_eval(state: &State) {
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => state.terminate(),
        Ok(_) => eval(state, &line),
        Err(err) => {
            state.set_status_code(nix::errno::errno());
            report_error(&err);
        }
    }
}

pub fn read_eval_loop(state: &State) {
    while state.running.get() {
        print_prompt(&state);
        read_eval(&state);
    }
}
