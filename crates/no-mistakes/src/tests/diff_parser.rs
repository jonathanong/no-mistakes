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
    let mut results: Vec<DiffFile> = Vec::new();
    let mut lines = diff_text.lines().peekable();

    while let Some(line) = lines.next() {
        if !line.starts_with("diff --git ") {
            continue;
        }
        let (old_path, new_path) = parse_diff_header(line);
        let mut rename_from: Option<PathBuf> = None;
        let mut rename_to: Option<PathBuf> = None;
        let mut minus_path: Option<&str> = None;
        let mut plus_path: Option<&str> = None;
        let mut removed_lines: Vec<String> = Vec::new();
        let mut added_lines: Vec<String> = Vec::new();
        let mut context_lines: Vec<String> = Vec::new();
        let mut hunk_lines: Vec<(HunkLineKind, String)> = Vec::new();
        let mut in_hunk = false;

        while let Some(&next) = lines.peek() {
            if next.starts_with("diff --git ") {
                break;
            }
            let next = lines.next().unwrap();
            // Once inside a hunk, payload lines whose body happens to start
            // with `--- ` or `+++ ` (e.g. an actual `--- foo` line in the
            // file content) must NOT be re-classified as path headers, or
            // we'd overwrite the captured paths and drop the line from
            // removed/added accumulation.
            if in_hunk {
                if let Some(rest) = next.strip_prefix('-') {
                    removed_lines.push(rest.to_string());
                    hunk_lines.push((HunkLineKind::Removed, rest.to_string()));
                } else if let Some(rest) = next.strip_prefix('+') {
                    added_lines.push(rest.to_string());
                    hunk_lines.push((HunkLineKind::Added, rest.to_string()));
                } else if let Some(rest) = next.strip_prefix(' ') {
                    context_lines.push(rest.to_string());
                    hunk_lines.push((HunkLineKind::Context, rest.to_string()));
                } else if next.starts_with("@@") {
                    // a follow-up hunk header: stay in_hunk
                }
                continue;
            }
            if let Some(rest) = next.strip_prefix("rename from ") {
                rename_from = Some(PathBuf::from(rest));
            } else if let Some(rest) = next.strip_prefix("rename to ") {
                rename_to = Some(PathBuf::from(rest));
            } else if let Some(rest) = next.strip_prefix("--- ") {
                minus_path = Some(rest);
            } else if let Some(rest) = next.strip_prefix("+++ ") {
                plus_path = Some(rest);
            } else if next.starts_with("@@") {
                in_hunk = true;
            }
        }

        if let (Some(from), Some(to)) = (rename_from, rename_to) {
            results.push(DiffFile {
                path: to,
                status: DiffFileStatus::Renamed,
                old_path: Some(from),
                removed_lines,
                added_lines,
                context_lines,
                hunk_lines,
            });
            continue;
        }

        let status = match (minus_path, plus_path) {
            (Some("/dev/null"), _) => DiffFileStatus::Added,
            (_, Some("/dev/null")) => DiffFileStatus::Deleted,
            _ => DiffFileStatus::Modified,
        };

        let path = match status {
            DiffFileStatus::Deleted => old_path.unwrap_or(new_path),
            _ => new_path,
        };

        results.push(DiffFile {
            path,
            status,
            old_path: None,
            removed_lines,
            added_lines,
            context_lines,
            hunk_lines,
        });
    }

    dedup_diff_files(results)
}

fn parse_diff_header(line: &str) -> (Option<PathBuf>, PathBuf) {
    let rest = line.strip_prefix("diff --git ").unwrap_or("");
    let (a_part, b_part) = match rest.split_once(" b/") {
        Some((a, b)) => (a.strip_prefix("a/").unwrap_or(a), b),
        None => return (None, PathBuf::from(rest)),
    };
    (Some(PathBuf::from(a_part)), PathBuf::from(b_part))
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
mod tests {
    use super::*;

    #[test]
    fn parse_simple_modified() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
index abc..def 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {}
+// new line
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("src/main.rs"));
        assert_eq!(files[0].status, DiffFileStatus::Modified);
        assert_eq!(files[0].removed_lines, Vec::<String>::new());
        assert_eq!(files[0].added_lines, vec!["// new line".to_string()]);
    }

    #[test]
    fn parse_captures_rename_pair_in_hunk_body() {
        let diff = "\
diff --git a/web/components/search-bar.tsx b/web/components/search-bar.tsx
--- a/web/components/search-bar.tsx
+++ b/web/components/search-bar.tsx
@@ -1,3 +1,3 @@
 export function SearchBar() {
-  return <form data-pw=\"search-bar\" />;
+  return <form data-pw=\"renamed-search-bar\" />;
 }
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].removed_lines,
            vec!["  return <form data-pw=\"search-bar\" />;".to_string()]
        );
        assert_eq!(
            files[0].added_lines,
            vec!["  return <form data-pw=\"renamed-search-bar\" />;".to_string()]
        );
    }

    #[test]
    fn parse_keeps_hunk_payload_lines_starting_with_dashes() {
        // Payload lines whose body begins with `--- ` or `+++ ` must be
        // captured as removed/added content, not misclassified as a new
        // path header overwriting the diff header we already parsed.
        let diff = "\
diff --git a/changelog.md b/changelog.md
--- a/changelog.md
+++ b/changelog.md
@@ -1,4 +1,4 @@
 # Changelog
-- old bullet
+- new bullet
-- divider removed
++ divider added
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("changelog.md"));
        assert_eq!(
            files[0].removed_lines,
            vec!["- old bullet".to_string(), "- divider removed".to_string()]
        );
        assert_eq!(
            files[0].added_lines,
            vec!["- new bullet".to_string(), "+ divider added".to_string()]
        );
    }

    #[test]
    fn parse_ignores_minus_plus_headers_in_hunk_capture() {
        let diff = "\
diff --git a/a.ts b/a.ts
--- a/a.ts
+++ b/a.ts
@@ -1 +1 @@
-old line
+new line
";
        let files = parse_unified_diff(diff);
        assert_eq!(files[0].removed_lines, vec!["old line".to_string()]);
        assert_eq!(files[0].added_lines, vec!["new line".to_string()]);
    }

    #[test]
    fn parse_multi_hunk_accumulates() {
        let diff = "\
diff --git a/a.ts b/a.ts
--- a/a.ts
+++ b/a.ts
@@ -1,3 +1,3 @@
 ctx
-rm1
+add1
@@ -10,3 +10,3 @@
 ctx
-rm2
+add2
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].removed_lines,
            vec!["rm1".to_string(), "rm2".to_string()]
        );
        assert_eq!(
            files[0].added_lines,
            vec!["add1".to_string(), "add2".to_string()]
        );
    }

    #[test]
    fn parse_new_file() {
        let diff = "\
diff --git a/new.ts b/new.ts
new file mode 100644
--- /dev/null
+++ b/new.ts
@@ -0,0 +1 @@
+export const x = 1;
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("new.ts"));
        assert_eq!(files[0].status, DiffFileStatus::Added);
    }

    #[test]
    fn parse_deleted_file() {
        let diff = "\
diff --git a/old.ts b/old.ts
deleted file mode 100644
--- a/old.ts
+++ /dev/null
@@ -1 +0,0 @@
-export const x = 1;
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("old.ts"));
        assert_eq!(files[0].status, DiffFileStatus::Deleted);
    }

    #[test]
    fn parse_renamed_file() {
        let diff = "\
diff --git a/old-name.ts b/new-name.ts
similarity index 100%
rename from old-name.ts
rename to new-name.ts
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("new-name.ts"));
        assert_eq!(files[0].status, DiffFileStatus::Renamed);
        assert_eq!(files[0].old_path, Some(PathBuf::from("old-name.ts")));
    }

    #[test]
    fn parse_multi_file_diff() {
        let diff = "\
diff --git a/a.mts b/a.mts
--- a/a.mts
+++ b/a.mts
@@ -1 +1,2 @@
 export const a = 1;
+export const a2 = 2;
diff --git a/b.mts b/b.mts
--- a/b.mts
+++ b/b.mts
@@ -1 +1,2 @@
 export const b = 1;
+export const b2 = 2;
diff --git a/c.mts b/c.mts
new file mode 100644
--- /dev/null
+++ b/c.mts
@@ -0,0 +1 @@
+export const c = 1;
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].path, PathBuf::from("a.mts"));
        assert_eq!(files[0].status, DiffFileStatus::Modified);
        assert_eq!(files[1].path, PathBuf::from("b.mts"));
        assert_eq!(files[1].status, DiffFileStatus::Modified);
        assert_eq!(files[2].path, PathBuf::from("c.mts"));
        assert_eq!(files[2].status, DiffFileStatus::Added);
    }

    #[test]
    fn parse_empty_diff() {
        let files = parse_unified_diff("");
        assert!(files.is_empty());
    }

    #[test]
    fn deduplicates_paths() {
        let diff = "\
diff --git a/x.ts b/x.ts
--- a/x.ts
+++ b/x.ts
@@ -1 +1 @@
-old
+new
diff --git a/x.ts b/x.ts
--- a/x.ts
+++ b/x.ts
@@ -2 +2 @@
-old2
+new2
";
        let files = parse_unified_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].removed_lines,
            vec!["old".to_string(), "old2".to_string()]
        );
        assert_eq!(
            files[0].added_lines,
            vec!["new".to_string(), "new2".to_string()]
        );
    }

    #[test]
    fn run_diff_command_captures_stdout() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_diff_command("echo 'hello world'", dir.path()).unwrap();
        assert_eq!(result.trim(), "hello world");
    }

    #[test]
    fn run_diff_command_fails_on_error() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_diff_command("exit 1", dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn run_diff_command_respects_expired_invocation_deadline() {
        let dir = tempfile::tempdir().unwrap();
        let _deadline =
            crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();

        let result = run_diff_command("touch spawned", dir.path());

        assert!(result.is_err());
        assert!(!dir.path().join("spawned").exists());
    }
}
