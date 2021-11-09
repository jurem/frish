use nix::sys::wait::waitpid;
use nix::unistd::{execvp, fork, ForkResult, Pid};

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

fn exec_external(prog: &str, args: Vec<&str>) {
    let prog = std::ffi::CString::new(prog).unwrap();
    let args = args
        .into_iter()
        .map(|arg| std::ffi::CString::new(arg))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // let args = args. into_iter().map(|arg| arg.as_ptr()).collect();
    match execvp(prog.as_ref(), args.as_ref()) {
        Err(_) => eprintln!("Invalid command"),
        Ok(_) => {}
    }
}

pub fn run_external(state: &mut State, tokens: Vec<&str>) {
    debug(state, "Executing external command");
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            if !state.background {
                state.status = wait_process(state, child);
            }
        }
        Ok(ForkResult::Child) => {
            exec_external(tokens[0], tokens);
        }
        Err(err) => {
            let err = std::io::Error::from(err);
            handle_ioerror(state, &err);
        }
    }
}

pub fn run_command(state: &mut State, command: &str) {
    let mut tokens = parser::tokenize(&command);
    if tokens.len() == 0 {
        debug(&state, "No command given.");
    } else {
        parser::parse(&mut tokens, state);
        if state.debug {
            eprintln!("Tokens: {:?}", tokens);
            eprintln!(
                "Modifiers: {} {} {}",
                state.inredirect, state.outredirect, state.background
            );
        }
        match state.find_builtin(tokens[0]) {
            Some(builtin) => (builtin.fun)(state, tokens),
            None => run_external(state, tokens),
        }
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
