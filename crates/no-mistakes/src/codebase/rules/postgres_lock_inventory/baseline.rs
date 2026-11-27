use super::{LockInventoryBaseline, LockInventoryBaselineSite, RULE_ID};
use crate::codebase::postgres::normalize_schema_catalog_path;
use crate::codebase::ts_source::{is_portably_absolute_path, JsonLoadError, SourceStore};
use anyhow::{anyhow, ensure, Context, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

mod compare;
pub(super) use compare::compare;

const VERSION: u32 = 1;

#[derive(Debug)]
pub(super) struct InvalidBaseline;

impl std::fmt::Display for InvalidBaseline {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("invalid baseline contents")
    }
}

pub(super) fn relative_path(raw: &str) -> Result<PathBuf> {
    let invalid = || anyhow!("{RULE_ID} baselineFile must be a non-empty repository-relative path");
    if is_portably_absolute_path(Path::new(raw)) || raw.contains('\\') {
        return Err(invalid());
    }
    normalize_schema_catalog_path(raw).map_err(|_| invalid())
}

pub(super) fn read_baseline(
    root: &Path,
    file: &str,
    sources: &SourceStore,
) -> Result<Option<LockInventoryBaseline>> {
    let path = root.join(relative_path(file)?);
    let canonical_root = root.canonicalize().context("resolve repository root")?;
    let context = || format!("read {RULE_ID} baseline {file}");
    let mut exists = true;
    for ancestor in path.ancestors().take_while(|path| *path != root) {
        match ancestor.canonicalize() {
            Ok(canonical) => {
                ensure!(
                    canonical.starts_with(&canonical_root),
                    "{} is outside the repository",
                    ancestor.display()
                );
                break;
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if let Ok(metadata) = ancestor.symlink_metadata() {
                    ensure!(
                        !metadata.is_symlink(),
                        "{} is a broken symbolic link",
                        ancestor.display()
                    );
                }
                exists = false;
            }
            Err(error) => return Err(error).with_context(context),
        }
    }
    if !exists {
        return Ok(None);
    }
    ensure!(
        path.symlink_metadata().with_context(context)?.is_file(),
        "{RULE_ID} baseline {file} must be a regular file"
    );
    let json = match sources.parse_json_path(&path) {
        Ok(json) => json,
        Err(error @ JsonLoadError::Syntax(_)) => {
            return Err(error).context(InvalidBaseline).with_context(context);
        }
        Err(error) => return Err(error).with_context(context),
    };
    let baseline = LockInventoryBaseline::deserialize(json.as_ref())
        .context(InvalidBaseline)
        .with_context(context)?;
    validate(&baseline)
        .context(InvalidBaseline)
        .with_context(context)?;
    Ok(Some(baseline))
}

fn validate(baseline: &LockInventoryBaseline) -> Result<()> {
    ensure!(
        baseline.version == VERSION,
        "unsupported version {}",
        baseline.version
    );
    let mut keys = BTreeSet::new();
    for site in &baseline.sites {
        ensure!(
            !site.file.is_empty() && !site.category.is_empty() && !site.key.is_empty(),
            "empty baseline site identity"
        );
        ensure!(
            keys.insert(identity(site)),
            "duplicate baseline site {} ({})",
            site.key,
            site.file
        );
    }
    let mut unanalyzed_keys = BTreeSet::new();
    for entry in &baseline.unanalyzed {
        ensure!(
            !entry.file.is_empty() && !entry.reason.is_empty(),
            "empty baseline unanalyzed identity"
        );
        ensure!(
            entry.count > 0,
            "baseline unanalyzed count must be positive ({}, {})",
            entry.file,
            entry.reason
        );
        ensure!(
            unanalyzed_keys.insert((entry.file.as_str(), entry.reason.as_str())),
            "duplicate baseline unanalyzed entry {} ({})",
            entry.file,
            entry.reason
        );
    }
    Ok(())
}

pub(super) fn identity(site: &LockInventoryBaselineSite) -> (&str, &str, &str) {
    (&site.file, &site.category, &site.key)
}
