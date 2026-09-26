use super::candidates::{filtered_shell_files, filtered_shell_files_with_sources};
use super::{run_shellcheck, Options, RuleFinding};
use crate::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    rule_filter: &super::super::path_filter::RulePathFilter,
    snapshot: &VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    let shell_candidates =
        select_shell_candidates(root, opts, files, target_roots, rule_filter, snapshot, None);
    if shell_candidates.is_empty() {
        return Ok(Vec::new());
    }
    run_shellcheck(root, opts, &shell_candidates)
}

pub(super) fn scan_with_sources(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    rule_filter: &super::super::path_filter::RulePathFilter,
    sources: &SourceStore,
    snapshot: &VisiblePathSnapshot,
) -> Result<Vec<RuleFinding>> {
    let shell_candidates = select_shell_candidates(
        root,
        opts,
        files,
        target_roots,
        rule_filter,
        snapshot,
        Some(sources),
    );
    if shell_candidates.is_empty() {
        return Ok(Vec::new());
    }
    run_shellcheck(root, opts, &shell_candidates)
}

pub(super) fn select_shell_candidates(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    rule_filter: &super::super::path_filter::RulePathFilter,
    snapshot: &VisiblePathSnapshot,
    sources: Option<&SourceStore>,
) -> Vec<PathBuf> {
    let tracked = opts
        .tracked_only
        .then(|| snapshot.tracked_paths_from(files));
    let files = tracked.as_deref().unwrap_or(files);
    let candidates = match sources {
        Some(sources) => {
            filtered_shell_files_with_sources(root, opts, files, target_roots, rule_filter, sources)
        }
        None => filtered_shell_files(root, opts, files, target_roots, rule_filter),
    };
    if opts.tracked_only {
        snapshot.tracked_paths_from(&candidates)
    } else {
        candidates
    }
}
