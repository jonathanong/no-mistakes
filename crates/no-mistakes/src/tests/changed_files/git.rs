use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(super) struct GitChangedFiles {
    pub(super) files: Vec<PathBuf>,
    pub(super) deleted: Vec<PathBuf>,
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

/// Name-status-only changed-file discovery for base/head. Used only when the
/// caller *also* supplied an explicit `--diff*` input (hunks already come
/// from that source in `collect_changed_files`); the streaming hunk producer
/// in `git_diff::stream_git_diff` is the primary base/head path otherwise.
///
/// On a nonzero exit this classifies the failure the same way
/// `stream_git_diff` does (see `git_diff::classify_git_diff_failure`) so
/// combined mode (`--diff-stdin --base --head`) surfaces the same stable
/// diagnostic codes as the primary streaming path, instead of a generic
/// `git command failed` message.
pub(super) fn get_git_changed_files(
    root: &Path,
    base: &str,
    head: Option<&str>,
) -> Result<GitChangedFiles> {
    let head_commit = head.unwrap_or("HEAD");
    let mut command = std::process::Command::new("git");
    command
        .args([
            "diff",
            "--relative",
            "--name-status",
            "-z",
            &format!("{base}...{head_commit}"),
        ])
        .current_dir(root);
    let output = crate::invocation::command_output(&mut command)?;
    if !output.status.success() {
        let classification = super::super::git_diff::classify_git_diff_failure(
            root,
            base,
            head_commit,
            &output.stderr,
        )?;
        return Err(classification.into());
    }
    parse_git_name_status_z(&output.stdout)
}

pub(super) fn parse_git_name_status_z(output: &[u8]) -> Result<GitChangedFiles> {
    let mut files = HashSet::new();
    let mut deleted = HashSet::new();
    let mut fields = output.split(|byte| *byte == b'\0');
    while let Some(status) = fields.next() {
        if status.is_empty() {
            continue;
        }
        let status = std::str::from_utf8(status).context("git diff returned a non-UTF-8 status")?;
        let first_path = fields
            .next()
            .filter(|path| !path.is_empty())
            .context("git diff --name-status returned a status without a path")?;
        let first_path = std::str::from_utf8(first_path)
            .context("git diff returned a non-UTF-8 changed path")?;
        if status.starts_with('R') || status.starts_with('C') {
            files.insert(PathBuf::from(first_path));
            if status.starts_with('R') {
                deleted.insert(PathBuf::from(first_path));
            }
            let second_path = fields
                .next()
                .filter(|path| !path.is_empty())
                .context("git diff --name-status returned a rename/copy without a destination")?;
            let second_path = std::str::from_utf8(second_path)
                .context("git diff returned a non-UTF-8 changed path")?;
            files.insert(PathBuf::from(second_path));
            continue;
        }
        let path = PathBuf::from(first_path);
        files.insert(path.clone());
        if status == "D" {
            deleted.insert(path);
        }
    }
    let mut files: Vec<_> = files.into_iter().collect();
    files.sort();
    let mut deleted: Vec<_> = deleted.into_iter().collect();
    deleted.sort();
    Ok(GitChangedFiles { files, deleted })
}
