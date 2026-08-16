const CHANGELOG: &str = include_str!("../../../CHANGELOG.md");

type Version = (u32, u32, u32);

fn parse_version(s: &str) -> Option<Version> {
    let s = s.trim().trim_start_matches('v');
    let mut it = s.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next()?.parse().ok()?;
    let patch = it.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

fn sections() -> Vec<(Version, String)> {
    let mut out = Vec::new();
    let mut version: Option<Version> = None;
    let mut body = String::new();
    for line in CHANGELOG.lines() {
        if let Some(rest) = line.trim().strip_prefix("## ") {
            if let Some(v) = version.take() {
                out.push((v, std::mem::take(&mut body)));
            }
            version = parse_version(rest);
        } else if version.is_some() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some(v) = version.take() {
        out.push((v, body));
    }
    out
}

pub fn notes_since(last_seen: &str, current: &str) -> Option<String> {
    let current = parse_version(current)?;
    let last_seen = parse_version(last_seen)?;

    let mut chosen: Vec<(Version, String)> = sections()
        .into_iter()
        .filter(|(v, _)| *v > last_seen && *v <= current)
        .collect();
    if chosen.is_empty() {
        return None;
    }
    chosen.sort_by(|a, b| b.0.cmp(&a.0));

    let multi = chosen.len() > 1;
    let mut result = String::new();
    for (i, ((major, minor, patch), body)) in chosen.iter().enumerate() {
        if multi {
            if i > 0 {
                result.push('\n');
            }
            result.push_str(&format!("## {}.{}.{}\n", major, minor, patch));
        }
        result.push_str(body.trim_end());
        result.push('\n');
    }
    Some(result.trim().to_string())
}
