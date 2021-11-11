use crate::common::Command;

pub fn tokenize(line: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
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

pub fn parse(line: &str) -> Option<Command> {
    let tokens = tokenize(&line);
    if tokens.len() == 0 {
        None
    } else {
        let mut last = tokens.len() - 1;
        // check for background
        let back = tokens[last] == "&";
        if back {
            last -= 1;
        }
        // check for output redirection
        let out = if tokens[last].starts_with(">") {
            last -= 1;
            Some(&tokens[last + 1][1..])
        } else {
            None
        };
        // check for input redirection
        let inr = if tokens[last].starts_with("<") {
            last -= 1;
            Some(&tokens[last + 1][1..])
        } else {
            None
        };
        // return
        Some(Command {
            args: tokens[0..last + 1].to_vec(),
            background: back,
            inredirect: inr,
            outredirect: out,
        })
    }
}
