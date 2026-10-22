use super::closure::only_or_all;
use super::FamilyPathScope;
use crate::codebase::rules::selected_rule_files;
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn from_rule_ids(
    root: &Path,
    files: &[PathBuf],
    config: &NoMistakesConfig,
    rule_ids: &[&str],
) -> Result<FamilyPathScope> {
    let mut matched = BTreeSet::new();
    let mut any = false;
    for rule_id in rule_ids {
        for application in config.rule_applications(rule_id) {
            any = true;
            for path in selected_rule_files(root, config, application, files)? {
                matched.insert(normalize_path(&path));
            }
        }
    }
    if !any {
        return Ok(FamilyPathScope::All);
    }
    Ok(only_or_all(matched, files.len()))
}
