use super::super::diff_parser::{DiffFile, DiffFileStatus};
use super::super::PlanArgs;
use anyhow::{Context, Result};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Whether the caller supplied an explicit unified-diff source
/// (`--diff`/`--diff-stdin`/`--diff-command`/the programmatic inline
/// `diff_content`). Shared with `lockfile_changes::is_diff_only_mode` and
/// with the base/head streaming-vs-name-status branch in `collect_changed_files`,
/// so both consumers agree on exactly which inputs count as "an explicit diff was
/// given."
pub(in crate::tests) fn has_explicit_diff_source(args: &PlanArgs) -> bool {
    args.diff.is_some()
        || args.diff_stdin
        || args.diff_command.is_some()
        || args.diff_content.is_some()
}

pub(super) fn collect_diff_files(
    args: &PlanArgs,
    root: &Path,
    files: &mut Vec<PathBuf>,
    deleted: &mut Vec<PathBuf>,
    diff_files_out: &mut Vec<DiffFile>,
) -> Result<()> {
    let diff_content = read_diff_content(args, root)?;
    let Some(content) = diff_content else {
        return Ok(());
    };

    let diff_files = super::super::diff_parser::parse_unified_diff_checked(&content)
        .context("provided unified diff contains a malformed path")?;
    apply_diff_files(&diff_files, root, files, deleted);
    diff_files_out.extend(diff_files);
    Ok(())
}

fn read_diff_content(args: &PlanArgs, root: &Path) -> Result<Option<String>> {
    if let Some(ref diff_path) = args.diff {
        let content = fs::read_to_string(diff_path)
            .with_context(|| format!("Failed to read diff file from {}", diff_path.display()))?;
        return Ok(Some(content));
    }

    if args.diff_stdin {
        let mut content = String::new();
        std::io::stdin()
            .read_to_string(&mut content)
            .context("Failed to read diff from stdin")?;
        return Ok(Some(content));
    }

    if let Some(ref cmd) = args.diff_command {
        let content = super::super::diff_parser::run_diff_command(cmd, root)?;
        return Ok(Some(content));
    }

    if let Some(ref content) = args.diff_content {
        return Ok(Some(content.clone()));
    }

    Ok(None)
}

pub(super) fn apply_diff_files(
    diff_files: &[DiffFile],
    root: &Path,
    files: &mut Vec<PathBuf>,
    deleted: &mut Vec<PathBuf>,
) {
    for df in diff_files {
        let path = if df.path.is_absolute() {
            df.path.clone()
        } else {
            root.join(&df.path)
        };
        files.push(path.clone());

        if df.status == DiffFileStatus::Deleted {
            deleted.push(path);
        }

        if matches!(df.status, DiffFileStatus::Renamed | DiffFileStatus::Copied) {
            if let Some(ref old) = df.old_path {
                let old_abs = if old.is_absolute() {
                    old.clone()
                } else {
                    root.join(old)
                };
                files.push(old_abs.clone());
                if df.status == DiffFileStatus::Renamed {
                    deleted.push(old_abs);
                }
            }
        }
    }
}
