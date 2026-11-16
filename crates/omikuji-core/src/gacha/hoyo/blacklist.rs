use serde::Deserialize;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::Path;

#[derive(Deserialize)]
struct BlacklistLine {
    #[serde(rename = "fileName")]
    file_name: String,
}

#[derive(Default)]
pub struct DownloadBlacklist {
    files: HashSet<String>,
    stems: HashSet<String>,
}

impl DownloadBlacklist {
    pub fn load(root: &Path, rel: &str) -> Self {
        let lines = match super::read_json_lines::<BlacklistLine>(&root.join(rel)) {
            Ok(lines) => lines,
            Err(e) if e.kind() == ErrorKind::NotFound => return Self::default(),
            Err(e) => {
                tracing::warn!("{e} - installing without the download blacklist");
                return Self::default();
            }
        };
        let files: HashSet<String> = lines.into_iter().map(|line| line.file_name).collect();
        let stems = files
            .iter()
            .filter_map(|file| file.rsplit_once('.'))
            .map(|(stem, _)| stem.to_owned())
            .collect();
        Self { files, stems }
    }

    pub fn contains(&self, name: &str) -> bool {
        match sidecar_owner(name) {
            Some(owner) => self.stems.contains(owner),
            None => self.files.contains(name),
        }
    }
}

fn sidecar_owner(name: &str) -> Option<&str> {
    let (owner, hash) = name.strip_suffix(".hash")?.rsplit_once('_')?;
    (hash.len() == 32 && hash.bytes().all(|b| b.is_ascii_hexdigit())).then_some(owner)
}
