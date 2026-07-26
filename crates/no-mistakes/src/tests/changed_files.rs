use super::diff_parser::{DiffFile, DiffFileStatus};
use super::PlanArgs;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

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
        let head = args.head.as_deref().unwrap_or("HEAD");
        // An explicit `--diff*` input already supplies hunks; streaming
        // base/head as well would feed the same files through
        // `DiffStreamParser` twice — `dedup_diff_files` *extends* (not
        // replaces) hunk_lines for a repeated path, so every hunk would be
        // double-counted. In that combined case, base/head contributes only
        // file/deleted discovery (its pre-streaming behavior), matching
        // today's `--diff-stdin --base X --head Y` combination.
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

/// Whether the caller supplied an explicit unified-diff source
/// (`--diff`/`--diff-stdin`/`--diff-command`/the programmatic inline
/// `diff_content`). Shared with `lockfile_changes::is_diff_only_mode` and
/// with the base/head streaming-vs-name-status branch above, so both
/// consumers agree on exactly which inputs count as "an explicit diff was
/// given."
pub(super) fn has_explicit_diff_source(args: &PlanArgs) -> bool {
    args.diff.is_some()
        || args.diff_stdin
        || args.diff_command.is_some()
        || args.diff_content.is_some()
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

    let diff_files = super::diff_parser::parse_unified_diff_checked(&content)
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

fn resolve_manual_path(path: &Path, root: &Path) -> Result<(PathBuf, PathBuf)> {
    let lexical = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let lexical = no_mistakes::codebase::ts_resolver::normalize_path(&lexical);
    if !lexical.starts_with(root) {
        anyhow::bail!(
            "changed file `{}` is outside the project root `{}`",
            path.display(),
            root.display()
        )
    }
    let analysis = lexical.canonicalize().unwrap_or_else(|_| lexical.clone());
    if !analysis.starts_with(root) {
        anyhow::bail!(
            "changed file `{}` resolves outside the project root `{}`",
            path.display(),
            root.display()
        )
    }
    Ok((lexical, analysis))
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

#[derive(Debug)]
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
fn get_git_changed_files(root: &Path, base: &str, head: Option<&str>) -> Result<GitChangedFiles> {
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
        let classification =
            super::git_diff::classify_git_diff_failure(root, base, head_commit, &output.stderr)?;
        return Err(classification.into());
    }
    parse_git_name_status_z(&output.stdout)
}

fn parse_git_name_status_z(output: &[u8]) -> Result<GitChangedFiles> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_name_status_preserves_deleted_and_renamed_paths() {
        let changed = parse_git_name_status_z(
            b"M\0alive.cs\0D\0deleted.cs\0R100\0old-name.cs\0new-name.cs\0",
        )
        .unwrap();

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
    fn git_name_status_preserves_both_copy_paths_without_marking_the_source_deleted() {
        let changed = parse_git_name_status_z(b"C100\0source.cs\0copied.cs\0").unwrap();

        assert_eq!(
            changed.files,
            vec![PathBuf::from("copied.cs"), PathBuf::from("source.cs")]
        );
        assert!(changed.deleted.is_empty());
    }

    #[test]
    fn git_name_status_nul_format_preserves_newlines_spaces_unicode_and_leading_dashes() {
        let changed = parse_git_name_status_z(
            "M\0line\nbreak.cs\0M\0space name.cs\0M\0日本語.cs\0M\0-leading.cs\0".as_bytes(),
        )
        .unwrap();

        assert_eq!(
            changed.files,
            vec![
                PathBuf::from("-leading.cs"),
                PathBuf::from("line\nbreak.cs"),
                PathBuf::from("space name.cs"),
                PathBuf::from("日本語.cs"),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn manual_symlink_keeps_lexical_inventory_path_and_resolves_analysis_target() {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("target.ts"), "export const value = 1;\n").unwrap();
        std::os::unix::fs::symlink("target.ts", root.path().join("alias.ts")).unwrap();
        let canonical_root = root.path().canonicalize().unwrap();

        let (inventory, analysis) =
            resolve_manual_path(Path::new("alias.ts"), &canonical_root).unwrap();

        assert_eq!(inventory, canonical_root.join("alias.ts"));
        assert_eq!(analysis, canonical_root.join("target.ts"));
    }

    #[test]
    fn manual_changed_path_rejects_lexical_and_resolved_root_escape() {
        let root = tempfile::tempdir().unwrap();
        let canonical_root = root.path().canonicalize().unwrap();
        let lexical = resolve_manual_path(Path::new("../outside.ts"), &canonical_root).unwrap_err();
        assert!(lexical.to_string().contains("outside the project root"));

        #[cfg(unix)]
        {
            let outside = tempfile::NamedTempFile::new().unwrap();
            std::os::unix::fs::symlink(outside.path(), root.path().join("escape.ts")).unwrap();
            let resolved =
                resolve_manual_path(Path::new("escape.ts"), &canonical_root).unwrap_err();
            assert!(resolved
                .to_string()
                .contains("resolves outside the project root"));
        }
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

    // Regression for a review finding on #587: combined mode (an explicit
    // `--diff*` input alongside `--base`/`--head`) used to surface a
    // generic `git command failed` message on an invalid ref instead of the
    // same stable diagnostic code the primary streaming path reports.
    #[test]
    fn combined_mode_git_failure_reports_a_stable_diagnostic_code() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        crate::test_support::git_init(root);
        fs::write(root.join("f.txt"), "one\n").unwrap();
        crate::test_support::git_commit_all(root, "base");

        let error = get_git_changed_files(root, "not-a-real-ref", Some("HEAD")).unwrap_err();
        let git_diff_error = error
            .downcast_ref::<crate::tests::git_diff::GitDiffError>()
            .expect("expected a GitDiffError");
        assert_eq!(git_diff_error.code(), "git-merge-base-unavailable");
    }
}
