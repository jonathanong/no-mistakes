use super::diff_parser::{DiffFile, DiffFileStatus};
use super::PlanArgs;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

pub(crate) struct ChangedFiles {
    pub files: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    /// Per-file hunk bodies parsed from the provided unified diff (if any).
    /// Each entry's `path` is the same absolute path that appears in `files`,
    /// so consumers can join on it. Empty when no `--diff*` flag was used.
    pub diff_files: Vec<DiffFile>,
}

pub(crate) fn collect_changed_files(args: &PlanArgs, root: &Path) -> Result<ChangedFiles> {
    let mut files = Vec::new();
    let mut deleted = Vec::new();
    let mut diff_files: Vec<DiffFile> = Vec::new();

    for f in &args.changed_file {
        files.push(resolve_path(f, root));
    }

    if let Some(ref path) = args.changed_files {
        let content = fs::read_to_string(path).with_context(|| {
            format!("Failed to read changed-files list from {}", path.display())
        })?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                files.push(resolve_path(&PathBuf::from(line), root));
            }
        }
    }

    if let Some(ref base) = args.base {
        match get_git_changed_files(root, base, args.head.as_deref()) {
            Ok(git_files) => {
                for f in git_files {
                    files.push(root.join(f));
                }
            }
            // When explicit --changed-file args were provided the base ref is
            // still forwarded to lockfile analysis; a git-diff failure here is
            // non-fatal so that analysis can report the appropriate warning.
            Err(_) if !files.is_empty() => {}
            Err(e) => return Err(e),
        }
    }

    collect_diff_files(args, root, &mut files, &mut deleted, &mut diff_files)?;

    let mut unique = HashSet::new();
    let mut result = Vec::new();
    for f in files {
        let normalized = no_mistakes::codebase::ts_resolver::normalize_path(&f);
        if unique.insert(normalized.clone()) {
            result.push(normalized);
        }
    }

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
            df
        })
        .collect();

    Ok(ChangedFiles {
        files: result,
        deleted: deleted_result,
        diff_files,
    })
}

fn collect_diff_files(
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

    let diff_files = super::diff_parser::parse_unified_diff(&content);
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
        let content = super::diff_parser::run_diff_command(cmd, root)?;
        return Ok(Some(content));
    }

    if let Some(ref content) = args.diff_content {
        return Ok(Some(content.clone()));
    }

    Ok(None)
}

fn apply_diff_files(
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

        if df.status == DiffFileStatus::Renamed {
            if let Some(ref old) = df.old_path {
                let old_abs = if old.is_absolute() {
                    old.clone()
                } else {
                    root.join(old)
                };
                files.push(old_abs.clone());
                deleted.push(old_abs);
            }
        }
    }
}

fn resolve_path(path: &Path, root: &Path) -> PathBuf {
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    abs.canonicalize()
        .unwrap_or_else(|_| no_mistakes::codebase::ts_resolver::normalize_path(&abs))
}

pub(crate) fn existing_changed_files(changed: &ChangedFiles) -> Vec<PathBuf> {
    changed
        .files
        .iter()
        .filter(|f| file_is_present(f))
        .cloned()
        .collect()
}

fn file_is_present(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory
            ) =>
        {
            false
        }
        Err(_) => true,
    }
}

fn get_git_changed_files(root: &Path, base: &str, head: Option<&str>) -> Result<Vec<PathBuf>> {
    let head_commit = head.unwrap_or("HEAD");
    let output = run_git(
        &[
            "diff",
            "--relative",
            "--name-only",
            &format!("{}...{}", base, head_commit),
        ],
        root,
    )?;
    let mut changed = HashSet::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            changed.insert(PathBuf::from(trimmed));
        }
    }
    let mut result: Vec<_> = changed.into_iter().collect();
    result.sort();
    Ok(result)
}

fn run_git(args: &[&str], root: &Path) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}
