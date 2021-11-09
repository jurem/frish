use crate::common::State;

pub fn tokenize(line: &str) -> Vec<&str> {
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

pub fn parse(tokens: &mut Vec<&str>, state: &mut State) {
    // check for background
    let last = tokens.len() - 1;
    state.background = tokens[last] == "&";
    if state.background {
        tokens.remove(last);
    }
    // check for output redirection
    let last = tokens.len() - 1;
    if tokens[last].starts_with(">") {
        state.outredirect = String::from(&tokens[last][1..]);
        tokens.remove(last);
    }
    // check for input redirection
    let last = tokens.len() - 1;
    if tokens[last].starts_with("<") {
        state.inredirect = String::from(&tokens[last][1..]);
        tokens.remove(last);
    }
}
