use std::io;
use std::io::Write;
use std::process::exit;

pub mod builtins;
pub mod common;

use crate::common::State;

fn print_prompt(state: &State) {
    if state.interactive {
        print!("{}> ", state.name);
        std::io::stdout().flush().expect("Cannot flush stdout");
    }
}

fn tokenize(line: &str) -> Vec<&str> {
    let mut tokens = vec![];
    let mut pos = 0;
    let mut iter = line.chars();
    let mut ch = iter.next();
    loop {
        match ch {
            // skip whitespace
            Some(ch1) if ch1.is_whitespace() => {
                pos += 1;
                ch = iter.next();
                continue;
            }
            // single or double quotes
            Some('"') | Some('\'') => {
                pos += 1;
                let token_start = pos;
                let mut ch1 = iter.next();
                loop {
                    match ch1 {
                        Some(ch2) => {
                            pos += 1;
                            ch1 = iter.next();
                            if ch2 == ch.unwrap() {
                                break;
                            }
                        }
                        None => break,
                    };
                }
                tokens.push(&line[token_start..pos - 1]);
            }
            // word
            Some(_) => {
                let token_start = pos;
                loop {
                    pos += 1;
                    ch = iter.next();
                    match ch {
                        Some(ch1) if ch1.is_whitespace() => break,
                        Some(_) => continue,
                        None => break,
                    };
                }
                tokens.push(&line[token_start..pos]);
            }
            None => break,
        }
        pos += 1;
        ch = iter.next();
    }
    return tokens;
}

fn init(_state: &State) {}

fn done() {}

fn main() {
    let mut state = State::new("frish");
    init(&state);
    // run
    while state.running {
        print_prompt(&state);
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(_len) => {
                let tokens = tokenize(&line);
                if tokens.len() == 0 {
                    continue;
                }
                println!("{:?}", tokens);
                match state.find(tokens[0]) {
                    Some(builtin) => (builtin.fun)(&mut state, tokens),
                    None => continue,
                }
            }
            Err(err) => println!("error: {}", err),
        }
    }
    // done
    done();
    exit(state.status);
}
