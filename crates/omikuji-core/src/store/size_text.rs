pub fn parse_labeled(line: &str, label: &str) -> Option<u64> {
    let idx = line.find(label)?;
    parse(&line[idx + label.len()..])
}

fn unit_multiplier(s: &str) -> Option<f64> {
    // longest prefix first so "GiB" beats "B"
    if s.starts_with("GiB") {
        Some(1024.0 * 1024.0 * 1024.0)
    } else if s.starts_with("MiB") {
        Some(1024.0 * 1024.0)
    } else if s.starts_with("KiB") {
        Some(1024.0)
    } else if s.starts_with("GB") {
        Some(1_000_000_000.0)
    } else if s.starts_with("MB") {
        Some(1_000_000.0)
    } else if s.starts_with("KB") {
        Some(1000.0)
    } else if s.starts_with('B') {
        Some(1.0)
    } else {
        None
    }
}

fn parse(s: &str) -> Option<u64> {
    let trimmed = s.trim_start();
    let num_end = trimmed
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(trimmed.len());
    let num: f64 = trimmed[..num_end].parse().ok()?;
    let rest = trimmed[num_end..].trim_start();
    let mult = unit_multiplier(rest)?;
    Some((num * mult) as u64)
}
