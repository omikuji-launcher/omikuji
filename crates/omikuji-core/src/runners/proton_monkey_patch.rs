/*
proton writes its own d3d dll overrides into WINEDLLOVERRIDES at the very end of its env setup after every hook it exposes so we cant override them AT ALL.

user_settings.py is a file proton imports when present, and it is the one place our code runs inside proton's own process.
we use it to swap the session's override table for a dict that refuses the keys we pinned, so proton can set d3d11=n as many times as it wants and none of them stick MUAHAHAHA

the file is inert unless OMIKUJI_PROTON_DLLS_PIN is set, so a runner carrying it behaves like a stock one for steam, for other launchers, and for our own games that pin nothing.

the hooks it needs are present in GE-Proton 7 through 11, cachyos 9 through 11, dwproton, and valve's own builds. proton wraps the import in try/except and the patch guards itself
so something that changes it simply will make the ENVs not get loaded, which is fine i guess.

also fuck proton for real like what the gargantuan fuck is this bullshit. like either im a idiot and i couldnt find better or this is just bad. but hey, this works :P
*/

use serde::Serialize;
use std::path::{Path, PathBuf};

pub const PIN_VAR: &str = "OMIKUJI_PROTON_DLLS_PIN";

const SOURCE: &str = include_str!("proton_monkey_patch.py");
const MARKER: &str = "# omikuji-proton-monkey-patch";
const FILE_NAME: &str = "user_settings.py";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchState {
    NotProton,
    Ready,
    Foreign,
    Unsupported,
}

impl PatchState {
    pub fn pins_apply(self) -> bool {
        self == Self::Ready
    }
}

fn patch_path(runner_dir: &Path) -> PathBuf {
    runner_dir.join(FILE_NAME)
}

fn hooks_present(runner_dir: &Path) -> bool {
    let Ok(script) = std::fs::read_to_string(runner_dir.join("proton")) else {
        return false;
    };
    script.contains("import user_settings") && script.contains("self.dlloverrides")
}

pub fn status(runner_dir: &Path) -> PatchState {
    if !super::is_proton_dir(runner_dir) {
        return PatchState::NotProton;
    }
    match std::fs::read_to_string(patch_path(runner_dir)) {
        Ok(existing) if existing.starts_with(MARKER) => PatchState::Ready,
        Ok(_) => PatchState::Foreign,
        Err(_) if hooks_present(runner_dir) => PatchState::Ready,
        Err(_) => PatchState::Unsupported,
    }
}

pub fn ensure_installed(runner_dir: &Path) -> PatchState {
    if !super::is_proton_dir(runner_dir) {
        return PatchState::NotProton;
    }
    let path = patch_path(runner_dir);
    match std::fs::read_to_string(&path) {
        Ok(existing) if existing == SOURCE => return PatchState::Ready,
        Ok(existing) if !existing.starts_with(MARKER) => {
            tracing::warn!(
                "{} already has its own {}, translation layer toggles will not apply",
                runner_dir.display(),
                FILE_NAME
            );
            return PatchState::Foreign;
        }
        _ => {}
    }
    if !hooks_present(runner_dir) {
        return PatchState::Unsupported;
    }
    match std::fs::write(&path, SOURCE) {
        Ok(()) => PatchState::Ready,
        Err(e) => {
            tracing::warn!("could not write {}: {}", path.display(), e);
            PatchState::Unsupported
        }
    }
}
