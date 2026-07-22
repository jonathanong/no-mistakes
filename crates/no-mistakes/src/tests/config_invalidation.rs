//! Request-scoped comparison for automatic and explicit v2 config changes.
//!
//! A changed configuration must remain a conservative full-suite trigger when
//! we cannot read both complete revisions. When both revisions are available,
//! however, only the requested framework is invalidated.

use super::changed_files::ChangedFiles;
use super::diff_parser::{DiffFile, DiffFileStatus};
use super::{PlanArgs, TestFramework};
use anyhow::{Context, Result};
use no_mistakes::config::v2::schema::NoMistakesConfig;
use std::fs;
use std::path::{Path, PathBuf};

mod reconstruction;
mod semantics;

#[cfg(test)]
use reconstruction::apply_unified_hunks;
use reconstruction::reconstruct_diff_sources;

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
        comparisons.push(parse_comparison(sources_from_git(
            root,
            base,
            args.head.as_deref(),
            args.config.as_deref(),
        )?)?);
    }
    if structured_diff_changes_config {
        comparisons.push(parse_comparison(sources_from_diff(
            root,
            args.config.as_deref(),
            &collected.diff_files,
        )?)?);
    }
    if comparisons.is_empty() {
        anyhow::bail!("configuration change has no revision or unified diff to compare")
    }
    Ok(Some(ConfigInvalidation {
        comparisons,
        trigger_file,
    }))
}

fn paths_change_config(args: &PlanArgs, root: &Path, paths: &[PathBuf]) -> bool {
    let candidates = config_candidates(args, root);
    paths.iter().any(|path| {
        candidates
            .iter()
            .any(|candidate| same_path(path, candidate))
    })
}

fn manual_config_paths_are_reconstructed(
    args: &PlanArgs,
    root: &Path,
    collected: &ChangedFiles,
) -> bool {
    let candidates = config_candidates(args, root);
    collected
        .manual_files
        .iter()
        .filter(|manual| {
            candidates
                .iter()
                .any(|candidate| same_path(manual, candidate))
        })
        .all(|manual| {
            collected
                .diff_files
                .iter()
                .any(|diff| diff_mentions_path(diff, manual))
        })
}

fn parse_comparison(
    (before, after): (Option<ConfigSource>, Option<ConfigSource>),
) -> Result<ConfigComparison> {
    Ok(ConfigComparison {
        before: parse_endpoint(before)?,
        after: parse_endpoint(after)?,
    })
}

fn diff_changes_config(args: &PlanArgs, root: &Path, diff_files: &[DiffFile]) -> bool {
    let candidates = config_candidates(args, root);
    diff_files.iter().any(|diff| {
        candidates
            .iter()
            .any(|candidate| diff_mentions_path(diff, candidate))
    })
}

fn diff_mentions_path(diff: &DiffFile, path: &Path) -> bool {
    same_path(&diff.path, path)
        || diff
            .old_path
            .as_ref()
            .is_some_and(|old| same_path(old, path))
}

fn config_candidates(args: &PlanArgs, root: &Path) -> Vec<PathBuf> {
    if let Some(config) = args.config.as_deref() {
        return vec![normalize(root, config)];
    }
    automatic_config_candidates(root)
}

fn automatic_config_candidates(root: &Path) -> Vec<PathBuf> {
    crate::config::v2::discover::automatic_v2_config_paths(root)
        .into_iter()
        .map(|path| normalize(root, &path))
        .collect()
}

pub(crate) fn changed_config_path(
    args: &PlanArgs,
    root: &Path,
    collected: &ChangedFiles,
) -> Option<PathBuf> {
    let candidates = config_candidates(args, root);
    collected
        .files
        .iter()
        .find(|path| {
            candidates
                .iter()
                .any(|candidate| same_path(path, candidate))
        })
        .cloned()
        .or_else(|| {
            collected.diff_files.iter().find_map(|diff| {
                candidates
                    .iter()
                    .find(|candidate| {
                        same_path(&diff.path, candidate)
                            || diff
                                .old_path
                                .as_ref()
                                .is_some_and(|old| same_path(old, candidate))
                    })
                    .cloned()
            })
        })
}

fn sources_from_git(
    root: &Path,
    base: &str,
    head: Option<&str>,
    explicit: Option<&Path>,
) -> Result<(Option<ConfigSource>, Option<ConfigSource>)> {
    let head = head.unwrap_or("HEAD");
    let merge_base = run_git(root, &["merge-base", base, head])?;
    let merge_base = merge_base.trim();
    if merge_base.is_empty() {
        anyhow::bail!("git merge-base returned an empty revision")
    }
    Ok((
        git_endpoint(root, merge_base, explicit)?,
        git_endpoint(root, head, explicit)?,
    ))
}

fn git_endpoint(
    root: &Path,
    revision: &str,
    explicit: Option<&Path>,
) -> Result<Option<ConfigSource>> {
    let candidates = endpoint_candidates(root, explicit);
    let mut found = Vec::new();
    for path in candidates {
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("config path {} is outside --root", path.display()))?;
        if let Some(source) = git_show(root, revision, relative)? {
            found.push(ConfigSource { path, source });
        }
    }
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        _ => anyhow::bail!("multiple .no-mistakes config files exist at revision {revision}"),
    }
}

fn sources_from_diff(
    root: &Path,
    explicit: Option<&Path>,
    diff_files: &[DiffFile],
) -> Result<(Option<ConfigSource>, Option<ConfigSource>)> {
    let candidates = endpoint_candidates(root, explicit);
    let mut before = Vec::new();
    let mut after = Vec::new();
    for candidate in candidates {
        if let Some(source) = diff_side_source(&candidate, diff_files, DiffSide::Before)? {
            before.push(source);
        }
        if let Some(source) = diff_side_source(&candidate, diff_files, DiffSide::After)? {
            after.push(source);
        }
    }
    if before.len() > 1 || after.len() > 1 {
        anyhow::bail!("multiple .no-mistakes config files are present in unified diff endpoints")
    }
    Ok((before.pop(), after.pop()))
}

fn endpoint_candidates(root: &Path, explicit: Option<&Path>) -> Vec<PathBuf> {
    explicit.map_or_else(
        || automatic_config_candidates(root),
        |path| vec![normalize(root, path)],
    )
}

#[derive(Clone, Copy)]
enum DiffSide {
    Before,
    After,
}

struct ConfigSource {
    path: PathBuf,
    source: String,
}

fn diff_side_source(
    candidate: &Path,
    diff_files: &[DiffFile],
    side: DiffSide,
) -> Result<Option<ConfigSource>> {
    let matching = diff_files
        .iter()
        .filter(|diff| {
            same_path(&diff.path, candidate)
                || diff
                    .old_path
                    .as_ref()
                    .is_some_and(|old| same_path(old, candidate))
        })
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        anyhow::bail!(
            "multiple unified diff entries match {}",
            candidate.display()
        )
    }
    let Some(diff) = matching.first() else {
        return fs::read_to_string(candidate)
            .map(|source| {
                Some(ConfigSource {
                    path: candidate.to_path_buf(),
                    source,
                })
            })
            .or_else(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(error.into())
                }
            });
    };

    let is_new_path = same_path(&diff.path, candidate);
    let is_old_path = diff
        .old_path
        .as_ref()
        .is_some_and(|old| same_path(old, candidate));
    let endpoint_matches = match (&diff.status, side) {
        (DiffFileStatus::Added, DiffSide::After) => is_new_path,
        (DiffFileStatus::Deleted, DiffSide::Before) => is_new_path,
        (DiffFileStatus::Modified, _) => is_new_path,
        (DiffFileStatus::Renamed, DiffSide::Before) => is_old_path,
        (DiffFileStatus::Renamed, DiffSide::After) => is_new_path,
        _ => false,
    };
    if !endpoint_matches {
        return Ok(None);
    }
    let (before, after) = reconstruct_diff_sources(diff)?;
    let source = match side {
        DiffSide::Before => before,
        DiffSide::After => after,
    };
    Ok(source.map(|source| ConfigSource {
        path: candidate.to_path_buf(),
        source,
    }))
}

fn parse_endpoint(source: Option<ConfigSource>) -> Result<NoMistakesConfig> {
    let Some(source) = source else {
        return Ok(NoMistakesConfig::default());
    };
    crate::config::v2::discover::parse_v2_config_quiet(&source.source, &source.path)
}

fn git_show(root: &Path, revision: &str, path: &Path) -> Result<Option<String>> {
    let spec = format!("{revision}:{}", path.to_string_lossy().replace('\\', "/"));
    let output = std::process::Command::new("git")
        .args(["show", &spec])
        .current_dir(root)
        .output()
        .context("running git show for configuration comparison")?;
    if output.status.success() {
        return Ok(Some(String::from_utf8(output.stdout)?));
    }
    if output.status.code() == Some(128) {
        return Ok(None);
    }
    anyhow::bail!(
        "git show failed: {}",
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run_git(root: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .context("running git for configuration comparison")?;
    if !output.status.success() {
        anyhow::bail!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
    Ok(String::from_utf8(output.stdout)?)
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
