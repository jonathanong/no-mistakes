use super::{LockInventoryBaseline, LockInventoryBaselineSite, LockInventoryReport, RULE_ID};
use crate::codebase::postgres::normalize_schema_catalog_path;
use crate::codebase::ts_source::{is_portably_absolute_path, JsonLoadError, SourceStore};
use anyhow::{anyhow, ensure, Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

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
    Ok(())
}

pub(super) fn compare(report: &mut LockInventoryReport, baseline: Option<&LockInventoryBaseline>) {
    let previous: BTreeMap<_, _> = baseline
        .into_iter()
        .flat_map(|baseline| &baseline.sites)
        .map(|site| (identity(site), site))
        .collect();
    let mut current = BTreeSet::new();
    for site in &report.sites {
        let entry = (
            site.file.as_str(),
            site.category.as_str(),
            site.key.as_str(),
        );
        if !previous.contains_key(&entry) {
            report.added.push(site.clone());
        }
        current.insert(entry);
    }
    if report.unanalyzed.is_empty() {
        report.removed.extend(
            previous
                .into_iter()
                .filter(|(key, _)| !current.contains(key))
                .map(|(_, entry)| entry.clone()),
        );
    }
}

fn identity(site: &LockInventoryBaselineSite) -> (&str, &str, &str) {
    (&site.file, &site.category, &site.key)
}
