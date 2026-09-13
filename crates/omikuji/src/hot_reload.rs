use crate::qml_tree;
use std::fmt::Write;
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

const MODULE: &str = "omikuji";

pub fn source_dir() -> Option<PathBuf> {
    match std::env::var("OMIKUJI_QML_HOTRELOAD") {
        Ok(val) if !val.is_empty() => Some(match val.as_str() {
            "1" | "true" => PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            path => PathBuf::from(path),
        }),
        _ => None,
    }
}

// qt locks a plugin module's namespace, so hot reload shadows the qrc module instead of registering into it
pub struct DiskModule {
    import_path: PathBuf,
}

impl DiskModule {
    pub fn mount(source_dir: &Path) -> io::Result<Self> {
        let import_path = dirs::runtime_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("omikuji-hot-reload");
        let module_dir = import_path.join(MODULE);
        fs::create_dir_all(&module_dir)?;

        let link = module_dir.join("qml");
        if link.symlink_metadata().is_ok() {
            fs::remove_file(&link)?;
        }
        symlink(source_dir.join("qml"), &link)?;

        fs::write(module_dir.join("qmldir"), qmldir(&module_dir)?)?;
        Ok(Self { import_path })
    }

    pub fn import_path(&self) -> &Path {
        &self.import_path
    }

    pub fn file_url(&self, rel: &str) -> String {
        format!("file://{}", self.import_path.join(MODULE).join(rel).display())
    }
}

fn qmldir(module_dir: &Path) -> io::Result<String> {
    let mut out = format!("module {MODULE}\noptional plugin {MODULE}\nclassname {MODULE}_plugin\n");
    for qml in qml_tree::walk_files(&module_dir.join("qml"), &["qml"])? {
        let Some(name) = qml.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let rel = qml.strip_prefix(module_dir).unwrap_or(&qml).display();
        let singleton = if qml_tree::is_singleton(&qml) { "singleton " } else { "" };
        let _ = writeln!(out, "{singleton}{name} 1.0 {rel}");
    }
    Ok(out)
}
