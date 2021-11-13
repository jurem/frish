use std::io;

use crate::common::{State, Status};

pub fn do_help(state: &State, _args: &[&str]) -> io::Result<Status> {
    for b in &state.builtins {
        println!("{:16}{}", b.command, b.help);
    }
    Ok(Status::success())
}

pub fn do_name(state: &State, args: &[&str]) -> io::Result<Status> {
    if args.len() > 1 {
        state.set_name(args[1])
    } else {
        println!("{}", state.name.borrow());
    }
    Ok(Status::success())
}

pub fn do_debug(state: &State, args: &[&str]) -> io::Result<Status> {
    if args.len() > 1 {
        state.debug.set(args[1] == "on");
    }
    println!("Debug is {}", if state.debug.get() { "on" } else { "off" });
    Ok(Status::success())
}

pub fn do_print(_: &State, args: &[&str]) -> io::Result<Status> {
    let last = args.last().unwrap();
    for arg in args.iter().skip(1) {
        print!("{}", arg);
        if arg != last {
            print!(" ");
        };
    }
    Ok(Status::success())
}

pub fn do_echo(state: &State, args: &[&str]) -> io::Result<Status> {
    do_print(state, args).and_then(|_| {
        println!("");
        Ok(Status::success())
    })
}
