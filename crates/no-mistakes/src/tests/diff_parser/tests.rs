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
    let _deadline = crate::invocation::install_test_deadline(std::time::Duration::ZERO).unwrap();

    let result = run_diff_command("touch spawned", dir.path());

    assert!(result.is_err());
    assert!(!dir.path().join("spawned").exists());
}

// The streaming git-diff producer (`invocation::child::stream`) feeds
// `DiffStreamParser` one line at a time instead of a fully-materialized
// `&str`. These tests prove that pushing lines individually — across a
// rename, a deletion, and a multi-hunk modification in the same
// patch — produces the exact same `Vec<DiffFile>` as `parse_unified_diff`
// on the equivalent full text, so the streaming path can never silently
// diverge from the inline-diff path it must match.
#[test]
fn stream_parser_matches_pull_parser_across_rename_delete_and_multi_hunk() {
    let diff = "\
diff --git a/src/helper.test.ts b/src/helper.renamed.test.ts
similarity index 100%
rename from src/helper.test.ts
rename to src/helper.renamed.test.ts
diff --git a/src/removed.ts b/src/removed.ts
deleted file mode 100644
index d471050..0000000
--- a/src/removed.ts
+++ /dev/null
@@ -1 +0,0 @@
-export const gone = 1;
diff --git a/src/multi.ts b/src/multi.ts
--- a/src/multi.ts
+++ b/src/multi.ts
@@ -1,3 +1,3 @@
 ctx
-rm1
+add1
@@ -10,3 +10,3 @@
 ctx
-rm2
+add2
";
    let expected = parse_unified_diff(diff);

    let mut parser = DiffStreamParser::new();
    for line in diff.lines() {
        parser.push_line(line);
    }
    let streamed = parser.finish();

    assert_eq!(streamed, expected);
    assert_eq!(streamed.len(), 3);
}

// Lines before the first `diff --git ` header (e.g. a `git format-patch`
// preamble, or the `commit <sha>` line some diff commands prefix) must be
// silently ignored rather than panicking on a `None` current-file state.
#[test]
fn stream_parser_ignores_lines_before_first_header() {
    let mut parser = DiffStreamParser::new();
    parser.push_line("commit abc123");
    parser.push_line("Author: Test <test@test.com>");
    parser.push_line("");
    parser.push_line("diff --git a/a.ts b/a.ts");
    parser.push_line("--- a/a.ts");
    parser.push_line("+++ b/a.ts");
    parser.push_line("@@ -1 +1 @@");
    parser.push_line("-old line");
    parser.push_line("+new line");

    let streamed = parser.finish();
    assert_eq!(streamed.len(), 1);
    assert_eq!(streamed[0].path, PathBuf::from("a.ts"));
    assert_eq!(streamed[0].removed_lines, vec!["old line".to_string()]);
}
