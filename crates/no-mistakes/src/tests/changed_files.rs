use super::diff_parser::DiffFile;
use super::PlanArgs;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

mod diff_inputs;
mod git;
mod paths;

pub(super) use diff_inputs::has_explicit_diff_source;
pub(crate) use git::parse_git_diff_refspec;
pub(crate) use paths::existing_changed_files;

use diff_inputs::{apply_diff_files, collect_diff_files};
use git::get_git_changed_files;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
use git::parse_git_name_status_z;
use paths::{normalize_unique, resolve_manual_path};

pub(crate) struct ChangedFiles {
    pub files: Vec<PathBuf>,
    /// Lexical paths exposed through `TestPlan.changed_files`. Manual
    /// symlinks retain the caller's repository-relative identity while
    /// `files` contains their resolved analysis targets.
    pub inventory_files: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    /// Paths reported specifically by the automatic `--base`/`--head` Git
    /// comparison. Kept separate so other input sources cannot borrow Git's
    /// endpoint provenance. This includes deleted paths and both sides of a
    /// rename or copy, preserving semantic deletion and path-pair comparisons.
    pub git_files: Vec<PathBuf>,
    /// Paths supplied directly through `--changed-file` or `--changed-files`.
    /// These do not provide either endpoint of a configuration comparison.
    pub manual_files: Vec<PathBuf>,
    /// Existing-file candidates named by caller-controlled file/diff inputs. These paths may
    /// be authoritative graph roots even when ignored by automatic repository discovery.
    /// Automatic `--base`/`--head` git-diff results are intentionally excluded.
    pub authoritative_files: Vec<PathBuf>,
    /// Per-file hunk bodies parsed from the provided unified diff (if any).
    /// Each entry's `path` is the same absolute path that appears in `files`,
    /// so consumers can join on it. Populated by an explicit `--diff*` flag,
    /// or — when none was given — by streaming `git diff <base>...<head>`
    /// for `--base`/`--head`/`--from-git-diff` (see `git_diff::stream_git_diff`).
    /// Empty only when neither input was supplied.
    pub diff_files: Vec<DiffFile>,
}

pub(crate) fn collect_changed_files(args: &PlanArgs, root: &Path) -> Result<ChangedFiles> {
    let mut files = Vec::new();
    let mut inventory_files = Vec::new();
    let mut deleted = Vec::new();
    let mut git_provenance_files = Vec::new();
    let mut manual_files = Vec::new();
    let mut authoritative_files = Vec::new();
    let mut diff_files: Vec<DiffFile> = Vec::new();

    for f in &args.changed_file {
        let (inventory_path, analysis_path) = resolve_manual_path(f, root)?;
        inventory_files.push(inventory_path);
        files.push(analysis_path.clone());
        authoritative_files.push(analysis_path.clone());
        manual_files.push(analysis_path);
    }

    if let Some(ref path) = args.changed_files {
        let content = fs::read_to_string(path).with_context(|| {
            format!("Failed to read changed-files list from {}", path.display())
        })?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let (inventory_path, analysis_path) =
                    resolve_manual_path(&PathBuf::from(line), root)?;
                inventory_files.push(inventory_path);
                files.push(analysis_path.clone());
                authoritative_files.push(analysis_path.clone());
                manual_files.push(analysis_path);
            }
        }
    }

    let has_explicit_files = !args.changed_file.is_empty() || args.changed_files.is_some();

    if let Some(ref base) = args.base {
        let head = args.head.as_deref().unwrap_or("HEAD");
        if has_explicit_diff_source(args) {
            match get_git_changed_files(root, base, args.head.as_deref()) {
                Ok(git_files) => {
                    for f in git_files.files {
                        let path = root.join(f);
                        files.push(path.clone());
                        inventory_files.push(path.clone());
                        git_provenance_files.push(path);
                    }
                    for f in git_files.deleted {
                        deleted.push(root.join(f));
                    }
                }
                Err(e) if has_explicit_files => {
                    eprintln!("warning: git diff failed ({e}); using explicit --changed-file list");
                }
                Err(e) => return Err(e),
            }
        } else {
            match super::git_diff::stream_git_diff(root, base, head) {
                Ok(diff) => {
                    let git_start = files.len();
                    apply_diff_files(&diff, root, &mut files, &mut deleted);
                    inventory_files.extend(files[git_start..].iter().cloned());
                    git_provenance_files.extend(files[git_start..].iter().cloned());
                    diff_files.extend(diff);
                }
                Err(e) if has_explicit_files => {
                    eprintln!(
                        "warning: git diff failed ({e:#}); using explicit --changed-file list"
                    );
                }
                Err(e) => return Err(e),
            }
        }
    }

    let explicit_diff_start = files.len();
    collect_diff_files(args, root, &mut files, &mut deleted, &mut diff_files)?;
    inventory_files.extend(files[explicit_diff_start..].iter().cloned());
    authoritative_files.extend(files[explicit_diff_start..].iter().cloned());

    let result = normalize_unique(files);
    let inventory_files = normalize_unique(inventory_files);
    let git_files = normalize_unique(git_provenance_files);
    let manual_files = normalize_unique(manual_files);
    let authoritative_files = normalize_unique(authoritative_files);

    let mut unique_deleted = HashSet::new();
    let mut deleted_result = Vec::new();
    for f in deleted {
        let normalized = no_mistakes::codebase::ts_resolver::normalize_path(&f);
        if unique_deleted.insert(normalized.clone()) {
            deleted_result.push(normalized);
        }
    }

    let diff_files = diff_files
        .into_iter()
        .map(|mut df| {
            let absolute = if df.path.is_absolute() {
                df.path.clone()
            } else {
                root.join(&df.path)
            };
            df.path = no_mistakes::codebase::ts_resolver::normalize_path(&absolute);
            if let Some(old_path) = df.old_path.take() {
                let absolute = if old_path.is_absolute() {
                    old_path
                } else {
                    root.join(old_path)
                };
                df.old_path = Some(no_mistakes::codebase::ts_resolver::normalize_path(
                    &absolute,
                ));
            }
            df
        })
        .collect();

    Ok(ChangedFiles {
        files: result,
        inventory_files,
        deleted: deleted_result,
        git_files,
        manual_files,
        authoritative_files,
        diff_files,
    })
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
#[path = "changed_files_tests.rs"]
mod tests;
