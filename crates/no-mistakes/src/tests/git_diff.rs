//! Streams `git diff <base>...<head>` into the same unified-diff parser
//! `--diff-stdin` uses, so `--base`/`--head`/`--from-git-diff` planning gets
//! identical hunks, rename/delete facts, and selector/route/queue/HTTP
//! coverage hints instead of only file names (see
//! `changed_files::collect_changed_files`). On failure, classifies the
//! Git-input problem into a stable, greppable diagnostic code so a caller
//! never mistakes a broken revision range for "nothing changed."

use super::diff_parser::DiffFile;
use super::diff_parser::DiffStreamParser;
use crate::invocation::stream_command_lines;
use std::path::Path;
use std::process::Command;

/// A single diff line without a newline beyond this many bytes is rejected
/// as malformed rather than grown without bound.
const MAX_DIFF_LINE_BYTES: usize = 16 * 1024 * 1024;

/// Stable, greppable diagnostic for a Git-input failure. `Display` embeds the
/// `[code]` token verbatim so it survives both the CLI's `eprintln!("error:
/// {error:#}")` and Node's `napi::Error::from_reason(format!("{error:#}"))` —
/// callers should match on the bracketed code, not the prose.
pub(crate) struct GitDiffError {
    code: &'static str,
    message: String,
}

impl GitDiffError {
    #[cfg(test)]
    pub(crate) fn code(&self) -> &'static str {
        self.code
    }
}

impl std::fmt::Debug for GitDiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for GitDiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.message, self.code)
    }
}

impl std::error::Error for GitDiffError {}

pub(crate) fn stream_git_diff(
    root: &Path,
    base: &str,
    head: &str,
) -> anyhow::Result<Vec<DiffFile>> {
    let mut command = Command::new("git");
    command
        .args([
            "-c",
            "core.quotePath=false",
            "diff",
            "--relative",
            "-M",
            "--unified=3",
            "--no-color",
            // A repo-local `diff.external`/`GIT_EXTERNAL_DIFF` or a
            // `.gitattributes` textconv driver can otherwise replace the
            // parseable unified-diff body with arbitrary output the parser
            // can't read, silently dropping real changed files.
            "--no-ext-diff",
            "--no-textconv",
            // Force git's own default prefixes so `diff.noprefix`/
            // `diff.mnemonicPrefix` config can't strip or rename them —
            // both the header split and the `---`/`+++` path stripping
            // below assume `a/`/`b/`.
            "--src-prefix=a/",
            "--dst-prefix=b/",
            &format!("{base}...{head}"),
        ])
        .current_dir(root);

    let mut parser = DiffStreamParser::new();
    let outcome = stream_command_lines(&mut command, MAX_DIFF_LINE_BYTES, |line| {
        parser.push_line(line);
        Ok(())
    });

    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(io_error) if io_error.kind() == std::io::ErrorKind::InvalidData => {
            return Err(GitDiffError {
                code: "git-malformed-output",
                message: io_error.to_string(),
            }
            .into());
        }
        // Spawn failures and invocation-deadline timeouts are not Git-input
        // problems; propagate the raw `io::Error` unwrapped so existing
        // timeout detection (`invocation::timeout_exit_code`, which
        // downcasts to `std::io::Error` with kind `TimedOut`) still applies.
        Err(io_error) => return Err(io_error.into()),
    };

    if outcome.status.success() {
        return Ok(parser.finish());
    }

    Err(classify_git_diff_failure(root, base, head, &outcome.stderr)?.into())
}

/// Classifies a failed `git diff <base>...<head>` by re-running small,
/// cheap Git probes — only reached on failure, so the common (successful)
/// path pays for nothing beyond the one streamed `git diff`. Also reused by
/// `changed_files::get_git_changed_files`'s name-status lookup (the
/// "combined" `--diff-stdin --base --head` mode) so both base/head paths
/// classify a Git failure into the same stable codes.
///
/// Returns the raw `io::Error` (never a `GitDiffError`) if a probe itself
/// hits the invocation deadline, so the timeout — not a misclassified
/// `git-merge-base-unavailable`/`git-shallow-history` — survives in the
/// error chain for `invocation::timeout_exit_code` to detect.
pub(super) fn classify_git_diff_failure(
    root: &Path,
    base: &str,
    head: &str,
    stderr: &[u8],
) -> std::io::Result<GitDiffError> {
    let stderr_text = String::from_utf8_lossy(stderr).trim().to_string();

    if super::lockfile_changes::find_git_root(root).is_none() {
        return Ok(GitDiffError {
            code: "git-not-a-repository",
            message: format!("`{}` is not inside a Git repository", root.display()),
        });
    }

    for (label, git_ref) in [("base", base), ("head", head)] {
        if !git_ref_resolves(root, git_ref)? {
            return Ok(GitDiffError {
                code: "git-merge-base-unavailable",
                message: format!(
                    "{label} ref `{git_ref}` does not resolve to a commit: {stderr_text}"
                ),
            });
        }
    }

    // Both refs resolve individually, yet `git diff` itself failed: the only
    // remaining Git-diagnosed cause is an unreachable merge base — either
    // because history was fetched shallowly (common in CI checkouts) or the
    // two refs are simply unrelated.
    if is_shallow_repository(root)? {
        return Ok(GitDiffError {
            code: "git-shallow-history",
            message: format!(
                "no merge base between `{base}` and `{head}` in a shallow clone; fetch \
                 more history (e.g. `git fetch --unshallow` or a deeper `--depth`): {stderr_text}"
            ),
        });
    }

    Ok(GitDiffError {
        code: "git-exit-failure",
        message: format!("git diff {base}...{head} failed: {stderr_text}"),
    })
}

/// `Ok(bool)` reports whether the ref resolves; a probe timeout propagates
/// as `Err` instead of being folded into "does not resolve".
fn git_ref_resolves(root: &Path, git_ref: &str) -> std::io::Result<bool> {
    let mut command = Command::new("git");
    command
        .args([
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{git_ref}^{{commit}}"),
        ])
        .current_dir(root);
    match crate::invocation::command_output(&mut command) {
        Ok(output) => Ok(output.status.success()),
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => Err(error),
        Err(_) => Ok(false),
    }
}

/// `Ok(bool)` reports whether the repo is shallow; a probe timeout
/// propagates as `Err` instead of being folded into "not shallow".
fn is_shallow_repository(root: &Path) -> std::io::Result<bool> {
    let mut command = Command::new("git");
    command
        .args(["rev-parse", "--is-shallow-repository"])
        .current_dir(root);
    match crate::invocation::command_output(&mut command) {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
        }
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => Err(error),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
#[path = "git_diff/tests.rs"]
mod tests;
