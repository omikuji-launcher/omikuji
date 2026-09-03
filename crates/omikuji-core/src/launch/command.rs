use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use super::{ProtonVerb, WineVariant};

pub fn wine_command<I, S>(
    exe: &Path,
    env: &HashMap<String, String>,
    variant: WineVariant,
    verb: Option<ProtonVerb>,
    args: I,
) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = Command::new(exe);
    cmd.args(args);
    // replace rather than extend so WINEPREFIX etc. from build_env win over the launcher's own env
    cmd.env_clear();
    cmd.envs(env);
    if let (WineVariant::Proton, Some(verb)) = (variant, verb) {
        cmd.env("PROTON_VERB", verb.as_str());
    }
    cmd
}
