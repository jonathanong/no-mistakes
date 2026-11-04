//! Request-scoped comparison for automatic and explicit v2 config changes.
//!
//! A changed configuration must remain a conservative full-suite trigger when
//! we cannot read both complete revisions. When both revisions are available,
//! however, only the requested framework is invalidated.

use super::changed_files::ChangedFiles;
use super::diff_parser::{DiffFile, DiffFileStatus};
use super::{PlanArgs, TestFramework};
use anyhow::Result;
use no_mistakes::config::v2::schema::NoMistakesConfig;
use std::path::{Path, PathBuf};

mod endpoints;
mod paths;
mod reconstruction;
mod semantics;
#[cfg(test)]
use endpoints::{diff_side_source, DiffSide};
use endpoints::{parse_endpoint, sources_from_diff, sources_from_git, ConfigSource};
use paths::*;
pub(crate) use paths::{automatic_input, changed_config_path};

#[cfg(test)]
use reconstruction::{apply_unified_hunks, reconstruct_diff_sources};

pub(crate) struct ConfigInvalidation {
    comparisons: Vec<ConfigComparison>,
    trigger_file: PathBuf,
}

struct ConfigComparison {
    before: NoMistakesConfig,
    after: NoMistakesConfig,
}

impl ConfigInvalidation {
    pub(crate) fn framework_changed(&self, framework: TestFramework) -> bool {
        self.comparisons.iter().any(|comparison| {
            semantics::framework_semantics_changed(&comparison.before, &comparison.after, framework)
        })
    }

    pub(crate) fn trigger(&self) -> (String, PathBuf) {
        let relative = self
            .trigger_file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(".no-mistakes.yml");
        (
            format!("Global configuration file changed: {relative}"),
            self.trigger_file.clone(),
        )
    }
}

/// Returns `None` when no effective v2 configuration was changed. Returns an
/// error for a changed config whose pair cannot be reconstructed: callers must
/// conservatively retain the global fallback in that case.
pub(crate) fn compare_changed_config(
    args: &PlanArgs,
    root: &Path,
    collected: &ChangedFiles,
    visible: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Option<ConfigInvalidation>> {
    let Some(trigger_file) = changed_config_path(args, root, collected) else {
        return Ok(None);
    };

    // Base/head streaming populates `diff_files` too (see
    // `changed_files::collect_changed_files`), but reconstructing config
    // sources from those hunks assumes the on-disk checkout is exactly one
    // diff endpoint. That is guaranteed for an explicit `--diff*` input, and
    // deliberately checked for a manually listed config path (below, and via
    // `manual_config_paths_are_reconstructed`) so a `--changed-file` claim
    // that has since diverged from the checkout still fails open. Neither
    // guarantee holds for a config path that only appears via automatic
    // base/head streaming with no manual claim on it — that path already has
    // a checkout-independent comparison via `sources_from_git` below, so
    // forcing diff-side reconstruction there only adds a needless failure
    // mode (checkout at neither endpoint, or a hunkless change) that would
    // discard a comparison that already succeeded.
    let structured_diff_changes_config = (super::changed_files::has_explicit_diff_source(args)
        || paths_change_config(args, root, &collected.manual_files))
        && diff_changes_config(args, root, &collected.diff_files);
    if !manual_config_paths_are_reconstructed(args, root, collected) {
        anyhow::bail!("manually listed configuration path has no matching structured diff endpoint")
    }

    let mut comparisons = Vec::new();
    if let Some(base) = args
        .base
        .as_deref()
        .filter(|_| paths_change_config(args, root, &collected.git_files))
    {
        comparisons.push(parse_comparison(
            root,
            args.config.is_some(),
            sources_from_git(root, base, args.head.as_deref(), args.config.as_deref())?,
        )?);
    }
    if structured_diff_changes_config {
        comparisons.push(parse_comparison(
            root,
            args.config.is_some(),
            sources_from_diff(
                root,
                args.config.as_deref(),
                &collected.diff_files,
                visible,
                sources,
            )?,
        )?);
    }
    if comparisons.is_empty() {
        anyhow::bail!("configuration change has no revision or unified diff to compare")
    }
    Ok(Some(ConfigInvalidation {
        comparisons,
        trigger_file,
    }))
}

fn parse_comparison(
    root: &Path,
    explicit: bool,
    (before, after): (Vec<ConfigSource>, Vec<ConfigSource>),
) -> Result<ConfigComparison> {
    Ok(ConfigComparison {
        before: parse_endpoint(root, before, explicit)?,
        after: parse_endpoint(root, after, explicit)?,
    })
}

fn normalize(root: &Path, path: &Path) -> PathBuf {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    no_mistakes::codebase::ts_resolver::normalize_path(&path)
}

fn same_path(left: &Path, right: &Path) -> bool {
    no_mistakes::codebase::ts_resolver::normalize_path(left)
        == no_mistakes::codebase::ts_resolver::normalize_path(right)
}

#[cfg(test)]
#[path = "config_invalidation/tests.rs"]
mod tests;
