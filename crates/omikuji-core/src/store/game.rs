use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreGame {
    pub app_name: String,
    pub title: String,
    pub banner: Option<String>,
    pub coverart: Option<String>,
    pub icon: Option<String>,
    pub is_installed: bool,
    pub install_path: Option<PathBuf>,
}
