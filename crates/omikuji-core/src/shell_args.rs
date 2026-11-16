use std::{iter, mem};

pub fn join(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if a.is_empty() {
                "\"\"".to_string()
            } else if a
                .chars()
                .any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '\\'))
            {
                let escaped = a.replace('\\', "\\\\").replace('"', "\\\"");
                format!("\"{}\"", escaped)
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn split(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut started = false;
    for c in s.chars() {
        if escape {
            cur.push(c);
            escape = false;
            started = true;
            continue;
        }
        if c == '\\' && !in_single {
            escape = true;
            continue;
        }
        if c == '\'' && !in_double {
            in_single = !in_single;
            started = true;
            continue;
        }
        if c == '"' && !in_single {
            in_double = !in_double;
            started = true;
            continue;
        }
        if c.is_whitespace() && !in_single && !in_double {
            if started {
                out.push(mem::take(&mut cur));
                started = false;
            }
            continue;
        }
        cur.push(c);
        started = true;
    }
    if started {
        out.push(cur);
    }
    out
}

pub fn split_windows(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut started = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                let mut run = 1;
                while chars.next_if_eq(&'\\').is_some() {
                    run += 1;
                }
                if chars.peek() == Some(&'"') {
                    cur.extend(iter::repeat_n('\\', run / 2));
                    if run % 2 == 1 {
                        cur.push('"');
                        chars.next();
                    }
                } else {
                    cur.extend(iter::repeat_n('\\', run));
                }
                started = true;
            }
            '"' => {
                in_quotes = !in_quotes;
                started = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if started {
                    out.push(mem::take(&mut cur));
                    started = false;
                }
            }
            c => {
                cur.push(c);
                started = true;
            }
        }
    }
    if started {
        out.push(cur);
    }
    out
}
