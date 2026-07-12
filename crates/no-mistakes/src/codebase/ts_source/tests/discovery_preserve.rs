use super::{git_add_all, git_init, write};
use crate::codebase::ts_source::GIT_LS_FILES_CALLS;
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

/// `no-mistakes check` calls `discover_files_preserving_roots_from_git_files`
/// (via `check_discovery::discover_check_files`) twice for the same root when
/// `forbidden-dependencies` is configured — once with the real skip-directory
/// filter, once with none. Both must reuse one already-fetched git-visible file
/// list rather than each independently spawning `git ls-files`. Asserts on the
/// spawn count, not output equality, since a non-sharing implementation would
/// still return the identical file lists while spawning `git` twice as often
/// (see `crates/CLAUDE.md`: "assert on a call count, not value equality").
#[test]
fn discover_files_preserving_roots_from_git_files_reuses_a_supplied_list() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "src/main.mts", "");
    write(dir.path(), "fixtures/app/src/lib.rs", "");
    git_add_all(dir.path());

    GIT_LS_FILES_CALLS.with(|calls| calls.set(0));

    let git_files = crate::codebase::ts_source::git_visible_files(dir.path())
        .expect("dir was git-initialized, so the git-visible list must be Some");
    assert_eq!(
        GIT_LS_FILES_CALLS.with(|calls| calls.get()),
        1,
        "fetching the list directly must spawn git exactly once"
    );

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

    assert_eq!(
        GIT_LS_FILES_CALLS.with(|calls| calls.get()),
        1,
        "reusing the supplied list for two calls must not spawn git again"
    );
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

    // Sanity check that the counter genuinely tracks spawns rather than being
    // stuck: a third call with `git_files: None` must fetch again and bump it.
    let _ = crate::codebase::ts_source::discover_files_preserving_roots_from_git_files(
        dir.path(),
        &[],
        &[dir.path().join("fixtures/app")],
        None,
    );
    assert_eq!(
        GIT_LS_FILES_CALLS.with(|calls| calls.get()),
        2,
        "omitting the supplied list must fall back to a fresh git ls-files spawn"
    );
}
