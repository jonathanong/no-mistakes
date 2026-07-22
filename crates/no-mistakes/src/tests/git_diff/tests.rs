use super::*;
use crate::tests::diff_parser::{parse_unified_diff, DiffFileStatus};
use std::process::{Command, Output};

fn git(args: &[&str], root: &Path) -> Output {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

fn init_repo(root: &Path) {
    git(&["init", "-q", "-b", "main"], root);
    git(&["config", "user.email", "test@test.com"], root);
    git(&["config", "user.name", "Test"], root);
}

fn commit_all(root: &Path, message: &str) {
    git(&["add", "-A"], root);
    git(&["commit", "-q", "-m", message], root);
}

/// Base/head repo mirroring the issue's fixture shape: a selector literal
/// change, a same-content rename, and a deletion — enough to exercise
/// modified/renamed/deleted facts and (for the equivalence test) a plain
/// piped `git diff` in the same invocation.
fn two_commit_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/helper.ts"), "export const x = 1;\n").unwrap();
    std::fs::write(root.join("src/helper.test.ts"), "test old\n").unwrap();
    std::fs::write(root.join("src/removed.ts"), "export const gone = 1;\n").unwrap();
    commit_all(root, "base");

    git(
        &["mv", "src/helper.test.ts", "src/helper.renamed.test.ts"],
        root,
    );
    git(&["rm", "-q", "src/removed.ts"], root);
    commit_all(root, "head");
    dir
}

#[test]
fn stream_git_diff_matches_plain_git_diff_piped_through_the_inline_parser() {
    let dir = two_commit_repo();
    let root = dir.path();

    let streamed = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();

    let plain = git(&["diff", "HEAD~1...HEAD"], root);
    assert!(plain.status.success());
    let expected = parse_unified_diff(&String::from_utf8(plain.stdout).unwrap());

    assert_eq!(streamed, expected);
}

#[test]
fn stream_git_diff_reports_rename_and_deletion_facts() {
    let dir = two_commit_repo();
    let root = dir.path();

    let diff = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();

    let renamed = diff
        .iter()
        .find(|f| f.path == Path::new("src/helper.renamed.test.ts"))
        .expect("rename should be present");
    assert_eq!(renamed.status, DiffFileStatus::Renamed);
    assert_eq!(
        renamed.old_path.as_deref(),
        Some(Path::new("src/helper.test.ts"))
    );

    let deleted = diff
        .iter()
        .find(|f| f.path == Path::new("src/removed.ts"))
        .expect("deletion should be present");
    assert_eq!(deleted.status, DiffFileStatus::Deleted);
}

#[test]
fn stream_git_diff_rejects_invalid_base_ref() {
    let dir = two_commit_repo();
    let root = dir.path();

    let error = stream_git_diff(root, "not-a-real-ref", "HEAD").unwrap_err();
    let git_diff_error = error.downcast_ref::<GitDiffError>().unwrap();
    assert_eq!(git_diff_error.code(), "git-merge-base-unavailable");
}

#[test]
fn stream_git_diff_rejects_invalid_head_ref() {
    let dir = two_commit_repo();
    let root = dir.path();

    let error = stream_git_diff(root, "HEAD~1", "not-a-real-ref").unwrap_err();
    let git_diff_error = error.downcast_ref::<GitDiffError>().unwrap();
    assert_eq!(git_diff_error.code(), "git-merge-base-unavailable");
}

#[test]
fn stream_git_diff_reports_not_a_repository_outside_a_git_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    let error = stream_git_diff(root, "HEAD~1", "HEAD").unwrap_err();
    let git_diff_error = error.downcast_ref::<GitDiffError>().unwrap();
    assert_eq!(git_diff_error.code(), "git-not-a-repository");
}

// A shallow clone whose fetched refs resolve individually (both `origin/main`
// and the local branch tip are real, fetched refs) but whose merge base was
// cut by the fetch depth. This is the realistic CI shape — not an invalid
// ref — so it must classify as `git-shallow-history`, not
// `git-merge-base-unavailable`. See the plan's advisor-flagged verification:
// `git diff <base>...<head>` itself fails (not just `git merge-base`) with
// "no merge base" once history is truncated, even though both refs verify.
#[test]
fn stream_git_diff_reports_shallow_history_when_merge_base_is_truncated() {
    let origin_dir = tempfile::tempdir().unwrap();
    let origin = origin_dir.path();
    init_repo(origin);
    std::fs::write(origin.join("f.txt"), "a\n").unwrap();
    commit_all(origin, "c1");
    std::fs::write(origin.join("f.txt"), "a\nb\n").unwrap();
    commit_all(origin, "c2");
    git(&["checkout", "-q", "-b", "feature"], origin);
    std::fs::write(origin.join("f.txt"), "a\nb\nc\n").unwrap();
    commit_all(origin, "c3-feature");

    let shallow_dir = tempfile::tempdir().unwrap();
    let shallow = shallow_dir.path();
    let clone = Command::new("git")
        .args([
            "clone",
            "--quiet",
            "--no-local",
            "--depth=1",
            "--branch",
            "feature",
            &format!("file://{}", origin.display()),
            shallow.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(clone.status.success(), "{clone:?}");
    let fetch = Command::new("git")
        .args([
            "fetch",
            "--quiet",
            "--depth=1",
            "origin",
            "main:refs/remotes/origin/main",
        ])
        .current_dir(shallow)
        .output()
        .unwrap();
    assert!(fetch.status.success(), "{fetch:?}");

    let error = stream_git_diff(shallow, "origin/main", "HEAD").unwrap_err();
    let git_diff_error = error.downcast_ref::<GitDiffError>().unwrap();
    assert_eq!(git_diff_error.code(), "git-shallow-history");
}

// A single unified-diff line without a newline beyond `MAX_DIFF_LINE_BYTES`
// must be rejected as malformed rather than buffered without bound.
#[test]
fn stream_git_diff_rejects_a_pathologically_long_line() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    std::fs::write(root.join("big.txt"), "old\n").unwrap();
    commit_all(root, "base");
    let huge_line = "x".repeat(MAX_DIFF_LINE_BYTES + 1024);
    std::fs::write(root.join("big.txt"), format!("{huge_line}\n")).unwrap();
    commit_all(root, "head");

    let error = stream_git_diff(root, "HEAD~1", "HEAD").unwrap_err();
    let git_diff_error = error.downcast_ref::<GitDiffError>().unwrap();
    assert_eq!(git_diff_error.code(), "git-malformed-output");
}

#[test]
fn git_diff_error_display_embeds_the_stable_code_token() {
    let error = GitDiffError {
        code: "git-shallow-history",
        message: "detail".to_string(),
    };
    assert_eq!(error.to_string(), "detail [git-shallow-history]");
}

// Compatibility: spaces and non-ASCII paths. `-c core.quotePath=false` is
// the whole reason a path like this comes back as raw UTF-8 instead of a
// C-style-quoted, backslash-escaped string the parser would otherwise need
// to unescape.
#[test]
fn stream_git_diff_handles_spaces_and_non_ascii_paths() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    let path = "src/héllo world.ts";
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join(path), "export const x = 1;\n").unwrap();
    commit_all(root, "base");
    std::fs::write(root.join(path), "export const x = 2;\n").unwrap();
    commit_all(root, "head");

    let diff = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].path, Path::new(path));
}

// Compatibility: a binary file change has no `@@` hunks — just a
// "Binary files ... differ" line — and must not be treated as malformed
// output or crash the parser.
#[test]
fn stream_git_diff_handles_binary_changes_without_hunks() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    std::fs::write(root.join("image.png"), [0u8, 1, 2, 255, 0, 254]).unwrap();
    commit_all(root, "base");
    std::fs::write(root.join("image.png"), [9u8, 8, 7, 6, 5, 4]).unwrap();
    commit_all(root, "head");

    let diff = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].path, Path::new("image.png"));
    assert_eq!(diff[0].status, DiffFileStatus::Modified);
    assert!(diff[0].hunk_lines.is_empty());
}

// Compatibility: a mode-only change (no content change) has no `@@` hunks
// either, and on some platforms produces no `--- `/`+++ ` lines at all.
#[cfg(unix)]
#[test]
fn stream_git_diff_handles_mode_only_changes() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    std::fs::write(root.join("script.sh"), "echo hi\n").unwrap();
    commit_all(root, "base");
    let mut perms = std::fs::metadata(root.join("script.sh"))
        .unwrap()
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(root.join("script.sh"), perms).unwrap();
    commit_all(root, "head");

    let diff = stream_git_diff(root, "HEAD~1", "HEAD");
    // A mode-only change must complete without error; whether the parser
    // surfaces a `DiffFile` for it is secondary to not crashing/hanging.
    assert!(diff.is_ok(), "{diff:?}");
}

// Compatibility: a path beginning with `-` must not be misread as a git
// flag. `stream_git_diff` never passes per-file paths to git (it diffs the
// whole `<base>...<head>` range), but the *parser* must still correctly
// read a `diff --git a/-weird.ts b/-weird.ts` header line.
// Regression for a review finding on #587: `diff.noprefix`/`diff.mnemonicPrefix`
// config would otherwise drop or rename the `a/`/`b/` prefixes the header
// split and `---`/`+++` path stripping both assume. `--src-prefix=a/
// --dst-prefix=b/` on the invocation must win over repo config.
#[test]
fn stream_git_diff_ignores_diff_noprefix_config() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    git(&["config", "diff.noprefix", "true"], root);
    std::fs::write(root.join("x.txt"), "one\n").unwrap();
    commit_all(root, "base");
    std::fs::write(root.join("x.txt"), "two\n").unwrap();
    commit_all(root, "head");

    let diff = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].path, Path::new("x.txt"));
    assert_eq!(diff[0].status, DiffFileStatus::Modified);
}

// Regression for a review finding on #587: hunkless deletions (empty or
// binary files) have no `--- `/`+++ ` lines at all — only `deleted file
// mode` — and must still be reported as `Deleted`, not silently dropped to
// `Modified`.
#[test]
fn stream_git_diff_reports_hunkless_empty_and_binary_deletions() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    std::fs::write(root.join("empty.txt"), "").unwrap();
    std::fs::write(root.join("bin.dat"), [0u8, 1, 2, 255]).unwrap();
    commit_all(root, "base");
    std::fs::remove_file(root.join("empty.txt")).unwrap();
    std::fs::remove_file(root.join("bin.dat")).unwrap();
    commit_all(root, "head");

    let diff = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();
    assert_eq!(diff.len(), 2);
    for path in ["empty.txt", "bin.dat"] {
        let file = diff
            .iter()
            .find(|f| f.path == Path::new(path))
            .unwrap_or_else(|| panic!("{path} should be present"));
        assert_eq!(file.status, DiffFileStatus::Deleted);
    }
}

// Regression for a review finding on #587: a path that itself contains the
// literal substring " b/" must not be corrupted by the header line's
// ambiguous split — the `--- `/`+++ ` lines (which git also disambiguates
// with a trailing tab, since the path contains whitespace) must win.
#[test]
fn stream_git_diff_handles_a_path_containing_the_header_split_delimiter() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    std::fs::create_dir_all(root.join("a b")).unwrap();
    let path = "a b/file.ts";
    std::fs::write(root.join(path), "one\n").unwrap();
    commit_all(root, "base");
    std::fs::write(root.join(path), "two\n").unwrap();
    commit_all(root, "head");

    let diff = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].path, Path::new(path));
}

// Regression for a review finding on #587: a hunkless deletion (no
// `--- `/`+++ ` lines) under a directory whose name contains the literal
// substring " b/" must still resolve to the correct path, via
// `parse_diff_header`'s equal-halves disambiguation rather than the
// `---`/`+++` preference (which doesn't apply here — there's no hunk).
#[test]
fn stream_git_diff_resolves_hunkless_deletion_with_an_ambiguous_path() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    std::fs::create_dir_all(root.join("a b")).unwrap();
    let path = "a b/empty.txt";
    std::fs::write(root.join(path), "").unwrap();
    commit_all(root, "base");
    std::fs::remove_file(root.join(path)).unwrap();
    commit_all(root, "head");

    let diff = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].path, Path::new(path));
    assert_eq!(diff[0].status, DiffFileStatus::Deleted);
}

#[test]
fn stream_git_diff_handles_a_path_starting_with_a_dash() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    init_repo(root);
    std::fs::write(root.join("-weird.ts"), "export const x = 1;\n").unwrap();
    commit_all(root, "base");
    std::fs::write(root.join("-weird.ts"), "export const x = 2;\n").unwrap();
    commit_all(root, "head");

    let diff = stream_git_diff(root, "HEAD~1", "HEAD").unwrap();
    assert_eq!(diff.len(), 1);
    assert_eq!(diff[0].path, Path::new("-weird.ts"));
}
