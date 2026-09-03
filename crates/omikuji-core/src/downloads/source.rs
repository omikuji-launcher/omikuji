use anyhow::{Result, anyhow};
use async_trait::async_trait;

use super::DownloadEntry;

#[async_trait]
pub trait DownloadSource: Send + Sync {
    // impl must call report_progress and check_control periodically;
    // return Ok(()) on control signal; worker decides paused vs cancelled
    async fn install(&self, entry: &DownloadEntry) -> Result<()>;

    async fn update(&self, _entry: &DownloadEntry) -> Result<()> {
        Err(anyhow!("this source does not support in-place updates"))
    }

    fn supports_repair(&self) -> bool {
        false
    }

    fn supports_import(&self) -> bool {
        false
    }

    async fn repair(&self, _entry: &DownloadEntry) -> Result<()> {
        Err(anyhow!("this source does not support repair"))
    }

    async fn import_existing(&self, _entry: &DownloadEntry) -> Result<()> {
        Err(anyhow!(
            "this source does not support importing existing installs"
        ))
    }

    // scratch left behind by a cancelled download
    fn cleanup_state(&self, _entry: &DownloadEntry) {}

    // partial work a retry must not resume from
    fn reset_for_retry(&self, _entry: &DownloadEntry) {}
}
