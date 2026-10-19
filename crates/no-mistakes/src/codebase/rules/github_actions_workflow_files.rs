use super::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use anyhow::Result;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// `Some(true)` is an action file. `Some(false)` is a workflow. `None` is ignored.
pub(super) fn gha_kind(rel: &str) -> Option<bool> {
    let yaml = rel.ends_with(".yml") || rel.ends_with(".yaml");
    if !yaml {
        return None;
    }
    // GitHub only runs a workflow file directly under `.github/workflows/`.
    if let Some(name) = rel.strip_prefix(".github/workflows/") {
        if !name.is_empty() && !name.contains('/') {
            return Some(false);
        }
        return None;
    }
    if rel.starts_with(".github/actions/")
        && (rel.ends_with("/action.yml") || rel.ends_with("/action.yaml"))
    {
        return Some(true);
    }
    None
}

pub(super) fn scan_files(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    judge: &(dyn Fn(&str, &str) -> Vec<RuleFinding> + Sync),
) -> Result<Vec<RuleFinding>> {
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            let rel = relative_slash_path(root, path);
            if gha_kind(&rel).is_none() {
                return Vec::new();
            }
            let Some(content) = super::read_source(sources, path) else {
                return Vec::new();
            };
            judge(&rel, &content)
        })
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}
