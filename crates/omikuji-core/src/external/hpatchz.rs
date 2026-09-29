use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::downloads::io_stats::track_child;
use crate::fs_util::{is_executable, set_executable};

pub fn binary_path() -> Result<PathBuf> {
    let p = crate::runtime_dir().join("hpatchz");
    if !p.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "hpatchz not found at {}, install it from Settings > Components",
                p.display()
            ),
        ));
    }
    Ok(p)
}

// aag-core checks stdout for "patch ok!", same approach here
pub fn patch(file: &Path, patch: &Path, output: &Path) -> Result<()> {
    let bin = binary_path()?;

    if !is_executable(&bin) {
        let _ = set_executable(&bin);
    }

    let child = Command::new(&bin)
        .arg("-f")
        .arg(file)
        .arg(patch)
        .arg(output)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    track_child(child.id());
    let out = child.wait_with_output()?;

    if String::from_utf8_lossy(&out.stdout).contains("patch ok!") {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(Error::other(format!("hpatchz failed: {}", err.trim())))
    }
}
