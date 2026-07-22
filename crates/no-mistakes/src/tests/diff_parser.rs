use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DiffFileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HunkLineKind {
    Removed,
    Added,
    Context,
}

/// One positioned unified-diff hunk. Keeping the ranges lets consumers
/// reconstruct a complete before/after file from the on-disk side without
/// guessing where repeated hunk text belongs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub lines: Vec<(HunkLineKind, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffFile {
    pub path: PathBuf,
    pub status: DiffFileStatus,
    pub old_path: Option<PathBuf>,
    pub removed_lines: Vec<String>,
    pub added_lines: Vec<String>,
    /// Hunk context lines (those that begin with a space inside a `@@` body).
    /// Used by the diff-aware coverage hint scanners so a multi-line call like
    /// `router.push(\n  "/old"\n);` matches when the literal is on a `-` line
    /// but the `router.push(` token is only on context.
    pub context_lines: Vec<String>,
    /// All hunk lines in source order, tagged by kind. Preserves the relative
    /// position of `-`, `+`, and ` ` lines so multi-line regex scans can see
    /// the literal in the same surrounding shape it had in the file (e.g. a
    /// `router.push(` on a context line *before* a removed `"/old"`), which
    /// the per-kind vectors above can't reconstruct on their own.
    pub hunk_lines: Vec<(HunkLineKind, String)>,
    /// Positioned hunks in source order. Unlike `hunk_lines`, this preserves
    /// the exact coordinates needed to apply the patch in either direction.
    pub hunks: Vec<DiffHunk>,
}

impl DiffFile {
    /// Iterate hunk lines that survive in the post-diff state plus the
    /// removed lines, preserving source order. Used by the multi-line scan
    /// so a removed literal can match against the surrounding call shape
    /// that lives on context lines around it.
    pub fn removed_with_context_in_order(&self) -> Vec<String> {
        self.hunk_lines
            .iter()
            .filter(|(kind, _)| matches!(kind, HunkLineKind::Removed | HunkLineKind::Context))
            .map(|(_, text)| text.clone())
            .collect()
    }

    /// Symmetric view used to check whether a value extracted from the
    /// "removed ∪ context" buffer still appears on the post-diff side
    /// (added ∪ context, in source order).
    pub fn added_with_context_in_order(&self) -> Vec<String> {
        self.hunk_lines
            .iter()
            .filter(|(kind, _)| matches!(kind, HunkLineKind::Added | HunkLineKind::Context))
            .map(|(_, text)| text.clone())
            .collect()
    }
}

pub(crate) fn parse_unified_diff(diff_text: &str) -> Vec<DiffFile> {
    let mut parser = DiffStreamParser::new();
    for line in diff_text.lines() {
        parser.push_line(line);
    }
    parser.finish()
}

/// Incremental (push-based) unified-diff parser. Lets a streaming producer
/// (e.g. `git diff` piped a chunk at a time, see `invocation::child::stream`)
/// feed one line at a time without buffering the whole patch in memory, while
/// producing the exact same [`DiffFile`] records as [`parse_unified_diff`].
/// [`parse_unified_diff`] itself is just this parser fed every line of a
/// fully-materialized `&str` — the state machine below is the single source
/// of truth for both call shapes.
pub(crate) struct DiffStreamParser {
    results: Vec<DiffFile>,
    current: Option<PendingDiffFile>,
}

/// Per-`diff --git` block accumulator. Mirrors the locals of the old
/// pull-parser's outer-loop body, just carried across `push_line` calls
/// instead of within one loop iteration. `minus_path`/`plus_path` are owned
/// (not borrowed `&str`) because a streaming caller's line buffer may not
/// outlive a single `push_line` call.
struct PendingDiffFile {
    old_path: Option<PathBuf>,
    new_path: PathBuf,
    rename_from: Option<PathBuf>,
    rename_to: Option<PathBuf>,
    minus_path: Option<String>,
    plus_path: Option<String>,
    /// Set on a header-phase `deleted file mode` line. Git omits `--- `/
    /// `+++ ` entirely for a hunkless deletion (an empty or binary file), so
    /// this is the only signal available for those cases.
    deleted_file_mode: bool,
    /// Set on a header-phase `new file mode` line, the additive counterpart
    /// to `deleted_file_mode` (distinct from a pure `new mode` line, which
    /// marks a mode-only change on an existing file).
    new_file_mode: bool,
    removed_lines: Vec<String>,
    added_lines: Vec<String>,
    context_lines: Vec<String>,
    hunk_lines: Vec<(HunkLineKind, String)>,
    hunks: Vec<DiffHunk>,
    current_hunk: Option<DiffHunk>,
    in_hunk: bool,
}

impl DiffStreamParser {
    pub(crate) fn new() -> Self {
        Self {
            results: Vec::new(),
            current: None,
        }
    }

    /// Feed one line of unified-diff text (without its trailing newline, same
    /// convention as `str::lines()`).
    pub(crate) fn push_line(&mut self, line: &str) {
        if line.starts_with("diff --git ") {
            self.finalize_current();
            let (old_path, new_path) = parse_diff_header(line);
            self.current = Some(PendingDiffFile::new(old_path, new_path));
            return;
        }
        // Lines before the first `diff --git ` header are ignored, matching
        // the old pull-parser's `if !line.starts_with(...) { continue }`.
        if let Some(pending) = self.current.as_mut() {
            pending.push_line(line);
        }
    }

    fn finalize_current(&mut self) {
        if let Some(pending) = self.current.take() {
            self.results.push(pending.finish());
        }
    }

    /// Finalize any in-progress block and return the parsed, deduplicated
    /// diff files (same shape and ordering as [`parse_unified_diff`]).
    pub(crate) fn finish(mut self) -> Vec<DiffFile> {
        self.finalize_current();
        dedup_diff_files(self.results)
    }
}

impl PendingDiffFile {
    fn new(old_path: Option<PathBuf>, new_path: PathBuf) -> Self {
        Self {
            old_path,
            new_path,
            rename_from: None,
            rename_to: None,
            minus_path: None,
            plus_path: None,
            deleted_file_mode: false,
            new_file_mode: false,
            removed_lines: Vec::new(),
            added_lines: Vec::new(),
            context_lines: Vec::new(),
            hunk_lines: Vec::new(),
            hunks: Vec::new(),
            current_hunk: None,
            in_hunk: false,
        }
    }

    fn push_line(&mut self, line: &str) {
        // Once inside a hunk, payload lines whose body happens to start
        // with `--- ` or `+++ ` (e.g. an actual `--- foo` line in the file
        // content) must NOT be re-classified as path headers, or we'd
        // overwrite the captured paths and drop the line from
        // removed/added accumulation.
        if self.in_hunk {
            if let Some(rest) = line.strip_prefix('-') {
                self.removed_lines.push(rest.to_string());
                self.hunk_lines
                    .push((HunkLineKind::Removed, rest.to_string()));
                if let Some(hunk) = self.current_hunk.as_mut() {
                    hunk.lines.push((HunkLineKind::Removed, rest.to_string()));
                }
            } else if let Some(rest) = line.strip_prefix('+') {
                self.added_lines.push(rest.to_string());
                self.hunk_lines
                    .push((HunkLineKind::Added, rest.to_string()));
                if let Some(hunk) = self.current_hunk.as_mut() {
                    hunk.lines.push((HunkLineKind::Added, rest.to_string()));
                }
            } else if let Some(rest) = line.strip_prefix(' ') {
                self.context_lines.push(rest.to_string());
                self.hunk_lines
                    .push((HunkLineKind::Context, rest.to_string()));
                if let Some(hunk) = self.current_hunk.as_mut() {
                    hunk.lines.push((HunkLineKind::Context, rest.to_string()));
                }
            } else if line.starts_with("@@") {
                if let Some(hunk) = self.current_hunk.take() {
                    self.hunks.push(hunk);
                }
                self.current_hunk = parse_hunk_header(line);
            }
            return;
        }
        if let Some(rest) = line.strip_prefix("rename from ") {
            self.rename_from = Some(PathBuf::from(rest));
        } else if let Some(rest) = line.strip_prefix("rename to ") {
            self.rename_to = Some(PathBuf::from(rest));
        } else if let Some(rest) = line.strip_prefix("--- ") {
            self.minus_path = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            self.plus_path = Some(rest.to_string());
        } else if line.starts_with("deleted file mode") {
            self.deleted_file_mode = true;
        } else if line.starts_with("new file mode") {
            self.new_file_mode = true;
        } else if line.starts_with("@@") {
            self.in_hunk = true;
            self.current_hunk = parse_hunk_header(line);
        }
    }

    fn finish(mut self) -> DiffFile {
        if let Some(hunk) = self.current_hunk.take() {
            self.hunks.push(hunk);
        }

        if let (Some(from), Some(to)) = (self.rename_from, self.rename_to) {
            return DiffFile {
                path: to,
                status: DiffFileStatus::Renamed,
                old_path: Some(from),
                removed_lines: self.removed_lines,
                added_lines: self.added_lines,
                context_lines: self.context_lines,
                hunk_lines: self.hunk_lines,
                hunks: self.hunks,
            };
        }

        let status = match (self.minus_path.as_deref(), self.plus_path.as_deref()) {
            (Some("/dev/null"), _) => DiffFileStatus::Added,
            (_, Some("/dev/null")) => DiffFileStatus::Deleted,
            // A hunkless deletion/addition (an empty or binary file) has no
            // `--- `/`+++ ` lines at all — git relies on the mode-change
            // header lines instead, so they're the only signal left.
            (None, None) if self.deleted_file_mode => DiffFileStatus::Deleted,
            (None, None) if self.new_file_mode => DiffFileStatus::Added,
            _ => DiffFileStatus::Modified,
        };

        // Prefer the unambiguous `---`/`+++` path over the header-derived
        // `old_path`/`new_path`: `diff --git a/X b/Y` is split on the first
        // literal " b/", which misparses a path that itself contains that
        // substring (e.g. `a b/file.ts`), while a `--- `/`+++ ` line is a
        // single path with no such split needed. Fall back to the header
        // only when there is no hunk at all to read a path from.
        let path = match status {
            DiffFileStatus::Deleted => self
                .minus_path
                .as_deref()
                .filter(|p| *p != "/dev/null")
                .map(strip_ab_prefix)
                .unwrap_or_else(|| self.old_path.unwrap_or(self.new_path)),
            _ => self
                .plus_path
                .as_deref()
                .filter(|p| *p != "/dev/null")
                .map(strip_ab_prefix)
                .unwrap_or(self.new_path),
        };

        DiffFile {
            path,
            status,
            old_path: None,
            removed_lines: self.removed_lines,
            added_lines: self.added_lines,
            context_lines: self.context_lines,
            hunk_lines: self.hunk_lines,
            hunks: self.hunks,
        }
    }
}

/// Strip a `--- `/`+++ ` line's leading `a/`/`b/` prefix (forced by
/// `stream_git_diff`'s `--src-prefix=a/ --dst-prefix=b/`) and git's trailing
/// tab disambiguator, appended only when the path itself contains
/// whitespace (e.g. `--- a/a b/file.ts\t`), so it doesn't become part of the
/// path.
fn strip_ab_prefix(raw: &str) -> PathBuf {
    let raw = raw.strip_suffix('\t').unwrap_or(raw);
    PathBuf::from(
        raw.strip_prefix("a/")
            .or_else(|| raw.strip_prefix("b/"))
            .unwrap_or(raw),
    )
}

fn parse_hunk_header(line: &str) -> Option<DiffHunk> {
    // @@ -old_start,old_count +new_start,new_count @@ optional section
    let mut parts = line.split_whitespace();
    let old = parts.next()?;
    let old_range = parts.next()?;
    let new_range = parts.next()?;
    if old != "@@" || parts.next()? != "@@" {
        return None;
    }
    let (old_start, old_count) = parse_hunk_range(old_range.strip_prefix('-')?)?;
    let (new_start, new_count) = parse_hunk_range(new_range.strip_prefix('+')?)?;
    Some(DiffHunk {
        old_start,
        old_count,
        new_start,
        new_count,
        lines: Vec::new(),
    })
}

fn parse_hunk_range(range: &str) -> Option<(usize, usize)> {
    let (start, count) = range
        .split_once(',')
        .map_or((range, "1"), |(start, count)| (start, count));
    Some((start.parse().ok()?, count.parse().ok()?))
}

fn parse_diff_header(line: &str) -> (Option<PathBuf>, PathBuf) {
    let rest = line.strip_prefix("diff --git ").unwrap_or("");
    // For anything but a rename, git shows the *same* path on both sides
    // (`a/P b/P`), so the correct split is wherever the two halves match —
    // not necessarily the first " b/" substring, which a path containing
    // that literal text (e.g. `a b/file.ts`) would misparse. This is the
    // only path source hunkless deletions/additions have (no `--- `/`+++ `
    // lines to fall back on), so it must be resolved here rather than left
    // to the `---`/`+++` preference in `PendingDiffFile::finish`.
    if let Some(path) = split_matching_ab_halves(rest) {
        return (Some(PathBuf::from(path)), PathBuf::from(path));
    }
    let (a_part, b_part) = match rest.split_once(" b/") {
        Some((a, b)) => (a.strip_prefix("a/").unwrap_or(a), b),
        None => return (None, PathBuf::from(rest)),
    };
    (Some(PathBuf::from(a_part)), PathBuf::from(b_part))
}

/// Find the `" b/"` occurrence in a `diff --git a/X b/Y` header's `a/X b/Y`
/// remainder where stripping the leading `a/` from the left half yields a
/// string identical to the right half — i.e. `X == Y`, true for every
/// non-rename diff. Tries every occurrence (not just the first), so a path
/// containing the literal substring `" b/"` still resolves correctly. A
/// genuine rename (`X != Y`) never matches here and falls through to the
/// naive first-occurrence split, which is fine: a rename's real path comes
/// from `rename from`/`rename to` lines, not this header.
fn split_matching_ab_halves(rest: &str) -> Option<&str> {
    let a_part = rest.strip_prefix("a/")?;
    let mut search_from = 0;
    while let Some(offset) = a_part[search_from..].find(" b/") {
        let split_at = search_from + offset;
        let candidate = &a_part[..split_at];
        let remainder = &a_part[split_at + " b/".len()..];
        if candidate == remainder {
            return Some(candidate);
        }
        search_from = split_at + 1;
    }
    None
}

fn dedup_diff_files(files: Vec<DiffFile>) -> Vec<DiffFile> {
    let mut index: std::collections::HashMap<PathBuf, usize> = std::collections::HashMap::new();
    let mut out: Vec<DiffFile> = Vec::new();
    for f in files {
        if let Some(&i) = index.get(&f.path) {
            out[i].removed_lines.extend(f.removed_lines);
            out[i].added_lines.extend(f.added_lines);
            out[i].context_lines.extend(f.context_lines);
            out[i].hunk_lines.extend(f.hunk_lines);
            out[i].hunks.extend(f.hunks);
        } else {
            index.insert(f.path.clone(), out.len());
            out.push(f);
        }
    }
    out
}

pub(crate) fn run_diff_command(command: &str, root: &Path) -> Result<String> {
    let mut child_command = std::process::Command::new("sh");
    child_command.args(["-c", command]).current_dir(root);
    let output = crate::invocation::command_output(&mut child_command)
        .with_context(|| format!("failed to run diff command: {command}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("diff command failed: {stderr}");
    }
    Ok(String::from_utf8(output.stdout)?)
}

#[cfg(test)]
#[path = "diff_parser/tests.rs"]
mod tests;
