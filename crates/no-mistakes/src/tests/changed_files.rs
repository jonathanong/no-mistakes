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
    /// Paths reported specifically by the automatic `--base`/`--head` Git
    /// comparison. Kept separate so other input sources cannot borrow Git's
    /// endpoint provenance. This includes deleted paths and both sides of a
    /// rename, preserving semantic deletion and rename comparisons.
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
    /// so consumers can join on it. Empty when no `--diff*` flag was used.
    pub diff_files: Vec<DiffFile>,
}

pub(crate) fn collect_changed_files(args: &PlanArgs, root: &Path) -> Result<ChangedFiles> {
    let mut files = Vec::new();
    let mut deleted = Vec::new();
    let mut git_provenance_files = Vec::new();
    let mut manual_files = Vec::new();
    let mut authoritative_files = Vec::new();
    let mut diff_files: Vec<DiffFile> = Vec::new();

    for f in &args.changed_file {
        let path = resolve_path(f, root);
        files.push(path.clone());
        authoritative_files.push(path.clone());
        manual_files.push(path);
    }

    if let Some(ref path) = args.changed_files {
        let content = fs::read_to_string(path).with_context(|| {
            format!("Failed to read changed-files list from {}", path.display())
        })?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let path = resolve_path(&PathBuf::from(line), root);
                files.push(path.clone());
                authoritative_files.push(path.clone());
                manual_files.push(path);
            }
        }
    }

    // Track whether the caller supplied explicit file args so that a git-diff
    // failure is non-fatal: the explicit list is still valid input for lockfile
    // analysis, which will emit its own warning about the missing baseline.
    let has_explicit_files = !args.changed_file.is_empty() || args.changed_files.is_some();

    // `--from-git-diff <refspec>` is resolved into base/head once, up front, by
    // `generate_plan` (before this function is ever called) — not here — so
    // that every consumer of args.base/args.head (this git-diff lookup AND
    // `analyze_lockfile_changes`, which reads the same fields directly) sees
    // an identical, already-desugared pair. By the time `args` reaches this
    // function, `args.from_git_diff` is always `None`.
    if let Some(ref base) = args.base {
        match get_git_changed_files(root, base, args.head.as_deref()) {
            Ok(git_files) => {
                for f in git_files.files {
                    let path = root.join(f);
                    files.push(path.clone());
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
    }

    let explicit_diff_start = files.len();
    collect_diff_files(args, root, &mut files, &mut deleted, &mut diff_files)?;
    authoritative_files.extend(files[explicit_diff_start..].iter().cloned());

    let result = normalize_unique(files);
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
        deleted: deleted_result,
        git_files,
        manual_files,
        authoritative_files,
        diff_files,
    })
}

fn normalize_unique(files: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = HashSet::new();
    files
        .into_iter()
        .map(|path| no_mistakes::codebase::ts_resolver::normalize_path(&path))
        .filter(|path| unique.insert(path.clone()))
        .collect()
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

struct GitChangedFiles {
    files: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
}

/// Parse a `git diff` refspec into `(base, optional head)`.
///
/// Accepts three-dot `A...B`, three-dot with an implicit head `A...` (head
/// defaults to `HEAD` downstream in [`get_git_changed_files`]), and a bare
/// base `A` (also defaults head to `HEAD`). This mirrors the merge-base
/// three-dot semantics `git diff` already uses for `--base`/`--head`, so
/// `--from-git-diff` is sugar over that existing path rather than a new
/// comparison mode.
///
/// Two-dot refspecs (`A..B`) are rejected: `git diff A..B` and
/// `git diff A...B` compare different bases (direct vs. merge-base), and
/// silently accepting `..` here would make `--from-git-diff` desugar to a
/// different comparison than the equivalent `--base`/`--head` flags. Callers
/// that want two-dot semantics should keep using `--base`/`--head` directly
/// (which also only supports the three-dot form today).
pub(crate) fn parse_git_diff_refspec(spec: &str) -> Result<(String, Option<String>)> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        anyhow::bail!("--from-git-diff requires a non-empty refspec, e.g. origin/main...HEAD");
    }

    if let Some((base, head)) = trimmed.split_once("...") {
        let base = base.trim();
        let head = head.trim();
        if base.is_empty() {
            anyhow::bail!("--from-git-diff refspec is missing a base before '...': {trimmed}");
        }
        if head.is_empty() {
            return Ok((base.to_string(), None));
        }
        return Ok((base.to_string(), Some(head.to_string())));
    }

    if trimmed.contains("..") {
        anyhow::bail!(
            "--from-git-diff does not support two-dot refspecs ('{trimmed}'); tests plan only \
             compares base...head (merge-base) diffs — use three-dot base...head \
             (e.g. origin/main...HEAD). --base/--head use the same three-dot comparison, \
             so switching to them will not change the diff you get."
        );
    }

    Ok((trimmed.to_string(), None))
}

fn get_git_changed_files(root: &Path, base: &str, head: Option<&str>) -> Result<GitChangedFiles> {
    let head_commit = head.unwrap_or("HEAD");
    let output = run_git(
        &[
            "diff",
            "--relative",
            "--name-status",
            &format!("{}...{}", base, head_commit),
        ],
        root,
    )?;
    Ok(parse_git_name_status(&output))
}

fn parse_git_name_status(output: &str) -> GitChangedFiles {
    let mut files = HashSet::new();
    let mut deleted = HashSet::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split('\t');
        let status = parts.next().unwrap_or_default();
        if status.starts_with('R') {
            if let Some(old_path) = parts.next() {
                files.insert(PathBuf::from(old_path));
                deleted.insert(PathBuf::from(old_path));
            }
            if let Some(new_path) = parts.next() {
                files.insert(PathBuf::from(new_path));
            }
            continue;
        }
        if let Some(path) = parts.next() {
            let path = PathBuf::from(path);
            files.insert(path.clone());
            if status == "D" {
                deleted.insert(path);
            }
        }
    }
    let mut files: Vec<_> = files.into_iter().collect();
    files.sort();
    let mut deleted: Vec<_> = deleted.into_iter().collect();
    deleted.sort();
    GitChangedFiles { files, deleted }
}

fn run_git(args: &[&str], root: &Path) -> Result<String> {
    let mut command = std::process::Command::new("git");
    command.args(args).current_dir(root);
    let output = crate::invocation::command_output(&mut command)?;
    if !output.status.success() {
        anyhow::bail!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_name_status_preserves_deleted_and_renamed_paths() {
        let changed =
            parse_git_name_status("M\talive.cs\nD\tdeleted.cs\nR100\told-name.cs\tnew-name.cs\n");

        assert_eq!(
            changed.files,
            vec![
                PathBuf::from("alive.cs"),
                PathBuf::from("deleted.cs"),
                PathBuf::from("new-name.cs"),
                PathBuf::from("old-name.cs"),
            ]
        );
        assert_eq!(
            changed.deleted,
            vec![PathBuf::from("deleted.cs"), PathBuf::from("old-name.cs")]
        );
    }

    #[test]
    fn refspec_three_dot_splits_base_and_head() {
        let (base, head) = parse_git_diff_refspec("origin/main...HEAD").unwrap();
        assert_eq!(base, "origin/main");
        assert_eq!(head.as_deref(), Some("HEAD"));
    }

    #[test]
    fn refspec_three_dot_with_trailing_dots_defaults_head() {
        let (base, head) = parse_git_diff_refspec("origin/main...").unwrap();
        assert_eq!(base, "origin/main");
        assert_eq!(head, None);
    }

    #[test]
    fn refspec_bare_base_defaults_head() {
        let (base, head) = parse_git_diff_refspec("origin/main").unwrap();
        assert_eq!(base, "origin/main");
        assert_eq!(head, None);
    }

    #[test]
    fn refspec_trims_surrounding_whitespace() {
        let (base, head) = parse_git_diff_refspec("  origin/main ... HEAD  ").unwrap();
        assert_eq!(base, "origin/main");
        assert_eq!(head.as_deref(), Some("HEAD"));
    }

    #[test]
    fn refspec_rejects_two_dot_form() {
        let err = parse_git_diff_refspec("origin/main..HEAD").unwrap_err();
        assert!(
            err.to_string().contains("two-dot"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn refspec_rejects_empty_string() {
        let err = parse_git_diff_refspec("   ").unwrap_err();
        assert!(
            err.to_string().contains("non-empty"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn refspec_rejects_missing_base_before_three_dots() {
        let err = parse_git_diff_refspec("...HEAD").unwrap_err();
        assert!(
            err.to_string().contains("missing a base"),
            "unexpected error: {err}"
        );
    }
}
