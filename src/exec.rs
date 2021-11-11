use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use nix::sys::wait::waitpid;
use nix::unistd::{close, dup, dup2, execvp, fork, ForkResult, Pid};
use nix::NixPath;
use std::os::unix::io::RawFd;

use std::io;

use crate::builtins::Builtin;
use crate::common::{debug, report_error};
use crate::common::{Command, State};
use crate::parser;

fn fork_child<F>(child: &mut F) -> io::Result<Pid>
where
    F: FnMut(),
{
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => Ok(child),
        Ok(ForkResult::Child) => {
            child();
            std::process::exit(127);
        }
        Err(_) => Err(io::Error::from_raw_os_error(nix::errno::errno())),
    }
}

fn wait_process(state: &State, pid: Pid) -> i32 {
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

fn restore_stdin(fdold: Option<RawFd>) {
    if let Some(fdinold) = fdold {
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
        dup2(fdold, 1).unwrap();
        close(fdold).unwrap();
    }
}

fn exec_external(prog: &str, args: &[&str]) -> Result<std::convert::Infallible, nix::errno::Errno> {
    let args = args
        .into_iter()
        .map(|&arg| std::ffi::CString::new(arg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // let args = args. into_iter().map(|arg| arg.as_ptr()).collect();
    prog.with_nix_path(|cprog| execvp(cprog, args.as_ref()))?
}

pub fn run_external(state: &State, cmd: Command) -> io::Result<i32> {
    debug(state, "Running external command");
    match unsafe { fork() }? {
        ForkResult::Parent { child } => {
            let status = if cmd.background {
                0
            } else {
                wait_process(state, child)
            };
            Ok(status)
        }
        ForkResult::Child => {
            redirect_stdin(cmd.inredirect)
                .and_then(|_| redirect_stdout(cmd.outredirect))
                .and_then::<std::convert::Infallible, _>(|_| {
                    exec_external(cmd.args[0], &cmd.args)?;
                    Err(std::io::Error::last_os_error())
                })?;

            // exit from child if redirect or exec fails
            std::process::exit(127);
        }
    }
}

pub fn run_builtin(builtin: &Builtin, state: &State, cmd: Command) -> io::Result<i32> {
    if cmd.background {
        debug(state, "Running builtin in background");
        state.lastpid.set(fork_child(&mut || {
            if redirect_stdin(cmd.inredirect).is_ok() || redirect_stdout(cmd.outredirect).is_ok() {
                (builtin.handler)(state, &cmd.args).unwrap();
            }
        })?);
        Ok(0)
    } else {
        debug(state, "Running builtin in foreround");
        let fdinold = redirect_stdin(cmd.inredirect)?;
        let fdoutold = redirect_stdout(cmd.outredirect).or_else(|err| {
            restore_stdin(fdinold);
            Err(err)
        })?;
        let status = (builtin.handler)(state, &cmd.args)?;
        debug(state, "Restoring stdin");
        restore_stdin(fdinold);
        restore_stdout(fdoutold);
        Ok(status)
    }
}

pub fn eval(state: &State, cmdstr: &str) {
    if let Some(cmd) = parser::parse(&cmdstr) {
        let res = match state.find_builtin(cmd.args[0]) {
            Some(builtin) => run_builtin(builtin, state, cmd),
            None => run_external(state, cmd),
        };
        match res {
            Ok(status) => state.set_status(status),
            Err(err) => {
                state.set_status(nix::errno::errno());
                report_error(&err);
            }
        }
    } else {
        debug(state, "No command given.");
    }
}

pub fn subshell(state: &State) {
    let mut line = String::new();
    match io::stdin().read_line(&mut line) {
        Ok(0) => state.terminate(),
        Ok(_) => eval(state, &line),
        Err(err) => {
            state.set_status(nix::errno::errno());
            report_error(&err);
        }
    }
}
