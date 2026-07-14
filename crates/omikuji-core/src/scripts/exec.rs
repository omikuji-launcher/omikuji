use super::{interpolate, InputKind, Script, Step};
use crate::library::Game;
use crate::wine_tools::WineTool;
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};

pub struct ExecOutcome {
    pub game: Option<Game>,
    pub exe_found: bool,
}

pub fn execute<F: FnMut(&str)>(
    script: &Script,
    values: &HashMap<String, String>,
    mut on_line: F,
) -> Result<ExecOutcome> {
    let display_name = script.game.as_ref().map_or(&script.script.name, |g| &g.name);
    let slug = crate::media::slugify(display_name);
    let id = crate::library::generate_id();

    let prefix = match script.prefix_input() {
        Some(input) => {
            let v = values.get(&input.id).map(String::as_str).unwrap_or("");
            if v.trim().is_empty() {
                bail!("no value for prefix input \"{}\"", input.id);
            }
            PathBuf::from(v)
        }
        None => crate::prefixes_dir().join(format!("{slug}-{id}")),
    };
    let cache = crate::cache_dir().join("scripts").join(format!("{slug}-{id}"));
    std::fs::create_dir_all(&cache)?;

    let mut vars: HashMap<String, String> = HashMap::new();
    for input in &script.inputs {
        let value = values
            .get(&input.id)
            .cloned()
            .unwrap_or_else(|| input.default.clone());
        match input.kind {
            InputKind::Choice if !input.options.contains(&value) => {
                bail!("\"{}\" is not an option of input \"{}\"", value, input.id)
            }
            InputKind::Bool if value != "true" && value != "false" => {
                bail!("input \"{}\" must be true or false", input.id)
            }
            _ => {}
        }
        vars.insert(input.id.clone(), value);
    }
    vars.insert("prefix".into(), prefix.to_string_lossy().into_owned());
    vars.insert("cache".into(), cache.to_string_lossy().into_owned());
    vars.insert(
        "home".into(),
        dirs::home_dir().unwrap_or_default().to_string_lossy().into_owned(),
    );

    let wine_version = script
        .game
        .as_ref()
        .filter(|g| !g.wine_version.is_empty())
        .map(|g| g.wine_version.clone());
    let tool_game = Game::with_options(
        script.script.name.clone(),
        PathBuf::new(),
        Some(prefix.to_string_lossy().into_owned()),
        Some("wine".to_string()),
        wine_version.clone(),
    );

    let total = script.steps.len();
    for (i, step) in script.steps.iter().enumerate() {
        on_line(&format!("[{}/{}] {}", i + 1, total, step.describe()));
        match step {
            Step::InitPrefix => {
                std::fs::create_dir_all(&prefix)?;
                crate::prefixes::bootstrap_prefix(&tool_game, &mut on_line)?;
            }
            Step::Winetricks { verbs } => {
                crate::wine_tools::run_streamed(
                    &tool_game,
                    WineTool::WinetricksVerbs(verbs.clone()),
                    &mut on_line,
                )?;
            }
            Step::Download { url, dest, sha256 } => {
                let url = interpolate(url, &vars)?;
                let dest = PathBuf::from(interpolate(dest, &vars)?);
                download_to(&url, &dest, sha256, &mut on_line)?;
            }
            Step::Extract { archive, dest } => {
                let archive = PathBuf::from(interpolate(archive, &vars)?);
                let dest = PathBuf::from(interpolate(dest, &vars)?);
                extract_archive(&archive, &dest)?;
            }
            Step::RunExe { exe } => {
                let exe = PathBuf::from(interpolate(exe, &vars)?);
                if !exe.is_file() {
                    bail!("exe not found: {}", exe.display());
                }
                let mut child = crate::wine_tools::run(&tool_game, WineTool::RunExe(exe))?;
                on_line("waiting for the program to exit...");
                let status = child.wait()?;
                if !status.success() {
                    on_line(&format!("program exited with {status}, continuing"));
                }
            }
            Step::Shell { run } => {
                shell(&interpolate(run, &vars)?, &prefix, &cache, &mut on_line)?;
            }
        }
    }

    let outcome = match &script.game {
        Some(spec) => {
            let exe = PathBuf::from(interpolate(&spec.exe, &vars)?);
            let exe_found = exe.is_file();
            if !exe_found {
                on_line(&format!("game exe not found at {}", exe.display()));
            }

            let runner = if spec.runner.is_empty() { "wine" } else { &spec.runner };
            let is_wine = runner == "wine";
            let mut game = Game::with_options(
                spec.name.clone(),
                exe,
                is_wine.then(|| prefix.to_string_lossy().into_owned()),
                Some(runner.to_string()),
                if is_wine { wine_version } else { None },
            );
            game.metadata.id = id;
            for (k, v) in &spec.env {
                game.launch.env.insert(k.clone(), interpolate(v, &vars)?);
            }
            for (k, v) in &spec.dll_overrides {
                game.wine.dll_overrides.insert(k.clone(), interpolate(v, &vars)?);
            }
            ExecOutcome { game: Some(game), exe_found }
        }
        None => ExecOutcome { game: None, exe_found: true },
    };

    let _ = std::fs::remove_dir_all(&cache);
    on_line("done");
    Ok(outcome)
}

fn shell<F: FnMut(&str)>(text: &str, prefix: &Path, cwd: &Path, on_line: &mut F) -> Result<()> {
    use std::sync::mpsc;

    let mut child = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("exec 2>&1\n{text}"))
        .current_dir(cwd)
        .env("WINEPREFIX", prefix)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn sh")?;

    let (tx, rx) = mpsc::channel();
    if let Some(out) = child.stdout.take() {
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
    }

    let status = loop {
        match rx.recv_timeout(std::time::Duration::from_millis(200)) {
            Ok(line) => on_line(&line),
            Err(mpsc::RecvTimeoutError::Disconnected) => break child.wait()?,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(status) = child.try_wait()? {
                    while let Ok(line) = rx.try_recv() {
                        on_line(&line);
                    }
                    break status;
                }
            }
        }
    };
    if !status.success() {
        bail!("shell step exited with {status}");
    }
    Ok(())
}

type TarDecoder = fn(std::io::BufReader<std::fs::File>) -> Result<Box<dyn Read>>;

const TAR_DECODERS: &[(&str, TarDecoder)] = &[
    (".tar.gz", |r| Ok(Box::new(flate2::read::GzDecoder::new(r)))),
    (".tgz", |r| Ok(Box::new(flate2::read::GzDecoder::new(r)))),
    (".tar.xz", |r| Ok(Box::new(xz2::read::XzDecoder::new(r)))),
    (".tar.zst", |r| Ok(Box::new(zstd::stream::read::Decoder::new(r)?))),
    (".tar", |r| Ok(Box::new(r))),
];

fn extract_archive(archive: &Path, dest: &Path) -> Result<()> {
    let name = archive
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_lowercase();
    let file = std::fs::File::open(archive)
        .with_context(|| format!("opening {}", archive.display()))?;
    let reader = std::io::BufReader::new(file);
    std::fs::create_dir_all(dest)?;

    if name.ends_with(".zip") {
        zip::ZipArchive::new(reader)?.extract(dest)?;
        return Ok(());
    }
    let Some((_, decode)) = TAR_DECODERS.iter().find(|(s, _)| name.ends_with(s)) else {
        bail!("unsupported archive type: {name} (zip, tar.gz, tar.xz, tar.zst, tar)");
    };
    tar::Archive::new(decode(reader)?).unpack(dest)?;
    Ok(())
}

fn download_to<F: FnMut(&str)>(
    url: &str,
    dest: &Path,
    sha256: &str,
    on_line: &mut F,
) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let client = reqwest::blocking::Client::builder()
        .user_agent("omikuji")
        .timeout(None::<std::time::Duration>)
        .build()?;
    let mut resp = client
        .get(url)
        .send()
        .with_context(|| format!("requesting {url}"))?
        .error_for_status()?;
    let total = resp.content_length().unwrap_or(0);

    let mut file = std::fs::File::create(dest)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 128 * 1024];
    let mut done: u64 = 0;
    let mut last_decile = 0;
    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        hasher.update(&buf[..n]);
        done += n as u64;
        if let Some(decile) = (done * 10).checked_div(total)
            && decile > last_decile
        {
            last_decile = decile;
            on_line(&format!("  {}% ({} / {} MiB)", decile * 10, done >> 20, total >> 20));
        }
    }
    file.flush()?;

    if !sha256.is_empty() {
        let got = format!("{:x}", hasher.finalize());
        if !got.eq_ignore_ascii_case(sha256) {
            let _ = std::fs::remove_file(dest);
            bail!("sha256 mismatch for {url}: expected {sha256}, got {got}");
        }
    }
    Ok(())
}
