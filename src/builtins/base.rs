use std::io; // Result
use std::str::FromStr;

use crate::state::{State, Status};

pub fn do_help(state: &State, _args: &[&str]) -> io::Result<Status> {
    for (_, b) in &state.builtins {
        println!("{:16}{}", b.command, b.hint);
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

pub fn do_loglevel(_: &State, args: &[&str]) -> io::Result<Status> {
    if args.len() > 1 {
        if let Ok(level) = log::LevelFilter::from_str(args[1]) {
            log::set_max_level(level);
        }
    }
    println!("Log level is {}", log::max_level());
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
