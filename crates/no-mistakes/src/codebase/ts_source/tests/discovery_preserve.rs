use super::{git_add_all, git_init, write};
use tempfile::TempDir;

#[test]
fn discover_files_preserving_roots_walks_preserved_skip_dir_subtrees() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "src/main.mts", "");
    write(dir.path(), "fixtures/app/src/lib.rs", "");
    write(dir.path(), "fixtures/other/src/lib.rs", "");

    let files = crate::codebase::ts_source::discover_files_preserving_roots(
        dir.path(),
        &["fixtures".to_string()],
        &[dir.path().join("fixtures/app")],
    );

    assert_eq!(
        files,
        vec![
            dir.path().join("fixtures/app/src/lib.rs"),
            dir.path().join("src/main.mts"),
        ]
    );
}

/// `no-mistakes check` calls `discover_files_preserving_roots_from_git_files` (via
/// `check_discovery::discover_check_files`) twice for the same root when
/// `forbidden-dependencies` is configured — once with the real skip-directory filter, once with
/// none. Both must reuse one already-fetched git-visible file list rather than each
/// independently re-spawning `git ls-files`. Constructs a disagreement case (`crates/CLAUDE.md`:
/// "assert on a call count, not value equality" / "construct a case where the two approaches
/// would disagree") rather than checking output equality, which a non-sharing implementation
/// would also satisfy at the time of the call: fetches the git-visible list once, then adds a
/// new git-visible file *after* that fetch. A call that correctly reuses the stale, already-
/// fetched list must not see the new file; a call that (bug) re-fetches internally would see it.
#[test]
fn discover_files_preserving_roots_from_git_files_reuses_a_stale_supplied_list() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "src/main.mts", "");
    write(dir.path(), "fixtures/app/src/lib.rs", "");
    git_add_all(dir.path());

    let git_files = crate::codebase::ts_source::git_visible_files(dir.path())
        .expect("dir was git-initialized, so the git-visible list must be Some");

    // Added to the git index *after* the fetch above, so it must be invisible to any call
    // that correctly reuses `git_files` rather than re-fetching.
    write(dir.path(), "fixtures/app/src/late.rs", "");
    git_add_all(dir.path());

    let skip_filtered = crate::codebase::ts_source::discover_files_preserving_roots_from_git_files(
        dir.path(),
        &["fixtures".to_string()],
        &[dir.path().join("fixtures/app")],
        Some(&git_files),
    );
    let unfiltered = crate::codebase::ts_source::discover_files_preserving_roots_from_git_files(
        dir.path(),
        &[],
        &[dir.path().join("fixtures/app")],
        Some(&git_files),
    );

    for (label, files) in [
        ("skip-filtered", &skip_filtered),
        ("unfiltered", &unfiltered),
    ] {
        assert!(
            !files.contains(&dir.path().join("fixtures/app/src/late.rs")),
            "{label} call must reuse the stale supplied list, not rediscover the file added \
             after the fetch, got {files:?}"
        );
    }
    assert_eq!(
        skip_filtered,
        vec![
            dir.path().join("fixtures/app/src/lib.rs"),
            dir.path().join("src/main.mts"),
        ]
    );
    assert_eq!(
        unfiltered, skip_filtered,
        "no other directories to skip here, so both filters agree"
    );

    // Sanity check that the disagreement is real (the new file genuinely is git-visible, and
    // this test isn't vacuously passing because it was never discoverable at all): a call
    // omitting the supplied list must fall back to a fresh fetch and find it.
    let fresh = crate::codebase::ts_source::discover_files_preserving_roots_from_git_files(
        dir.path(),
        &[],
        &[dir.path().join("fixtures/app")],
        None,
    );
    assert!(
        fresh.contains(&dir.path().join("fixtures/app/src/late.rs")),
        "sanity check: a fresh fetch (git_files: None) must see the file added after the \
         earlier fetch, got {fresh:?}"
    );
}
