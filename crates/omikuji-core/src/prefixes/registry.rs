use std::collections::BTreeMap;
use std::path::Path;

pub type Values = BTreeMap<String, String>;

const HIVES: [&str; 2] = ["system.reg", "user.reg"];

pub fn read_key(prefix: &Path, key: &str) -> Option<Values> {
    for candidate in key_views(key) {
        for hive in HIVES {
            let Ok(body) = std::fs::read_to_string(prefix.join(hive)) else {
                continue;
            };
            match find_key(&body, &candidate) {
                Some(values) if !values.is_empty() => return Some(values),
                _ => {}
            }
        }
    }
    None
}

fn key_views(key: &str) -> Vec<String> {
    let key = key.trim_matches('\\');
    let mut out = vec![key.to_string()];
    if let Some(rest) = key.strip_prefix("Software\\")
        && !rest.starts_with("Wow6432Node\\")
    {
        out.push(format!("Software\\Wow6432Node\\{rest}"));
    }
    out
}

fn find_key(body: &str, key: &str) -> Option<Values> {
    let mut inside = false;
    let mut values = Values::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix('[') {
            if inside {
                break;
            }
            let name = rest.split(']').next().unwrap_or_default();
            inside = unescape(name).eq_ignore_ascii_case(key);
            continue;
        }
        if inside && let Some((name, value)) = parse_value(line) {
            values.insert(name, value);
        }
    }
    inside.then_some(values)
}

fn parse_value(line: &str) -> Option<(String, String)> {
    let (name, rest) = match line.strip_prefix("@=") {
        Some(rest) => (String::new(), rest),
        None => {
            let rest = line.strip_prefix('"')?;
            let end = closing_quote(rest)?;
            (unescape(&rest[..end]), rest[end + 1..].strip_prefix('=')?)
        }
    };
    let rest = match rest.split_once(':') {
        Some((kind, tail)) if kind.starts_with("str(") => tail,
        _ => rest,
    };
    let body = rest.strip_prefix('"')?;
    let end = closing_quote(body)?;
    Some((name, unescape(&body[..end])))
}

fn closing_quote(s: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            '"' => return Some(i),
            _ => {}
        }
    }
    None
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.extend(chars.next()),
            _ => out.push(c),
        }
    }
    out
}
