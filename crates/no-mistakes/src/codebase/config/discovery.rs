use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::find_automatic_config_path;

pub(super) fn find_codebase_config_path(start: &Path) -> Result<Option<PathBuf>> {
    let mut current = start.to_path_buf();
    loop {
        if let Some(path) = find_automatic_config_path(&current, &[".no-mistakes"])? {
            return Ok(Some(path));
        }
        if !current.pop() {
            return Ok(None);
        }
    }
}
