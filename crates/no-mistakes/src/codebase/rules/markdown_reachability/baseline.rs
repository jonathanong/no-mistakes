use super::RULE_ID;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct BaselineEntry {
    pub(super) state: String,
    #[serde(default)]
    pub(super) depth: Option<usize>,
}

impl BaselineEntry {
    pub(super) fn depth(depth: usize) -> Self {
        Self {
            state: "depth".to_string(),
            depth: Some(depth),
        }
    }

    pub(super) fn unreachable() -> Self {
        Self {
            state: "unreachable".to_string(),
            depth: None,
        }
    }
}

pub(super) fn read_baseline(
    root: &Path,
    path: Option<&Path>,
    tracked_files: &[PathBuf],
) -> Result<BTreeMap<String, BaselineEntry>> {
    let Some(path) = path else {
        return Ok(BTreeMap::new());
    };
    let baseline_path = crate::codebase::ts_resolver::normalize_path(&root.join(path));
    if !tracked_files
        .iter()
        .any(|file| crate::codebase::ts_resolver::normalize_path(file) == baseline_path)
    {
        anyhow::bail!(
            "{RULE_ID} options.baselineFile must reference a tracked repository file: {}",
            path.display()
        )
    }
    let content = std::fs::read_to_string(&baseline_path)
        .context(format!("read {RULE_ID} baseline {}", path.display()))?;
    serde_json::from_str(&content).context("parse markdown-reachability baseline JSON")
}
