use super::*;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

fn git_init(dir: &Path) {
    let output = Command::new("git")
        .args(["init", "-q", "--initial-branch=main"])
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_add_all(dir: &Path) {
    let output = Command::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write(dir: &Path, path: &str, content: &str) {
    let full = dir.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, content).unwrap();
}

/// Regression test for `source_file_candidates` walking large gitignored
/// directories instead of deriving candidates from the git-visible file
/// list. Before the fix, `collect_app_selectors` always did a raw recursive
/// `WalkDir` walk whose only `.gitignore` awareness was
/// `is_skipped_dir`'s small hardcoded list (`node_modules`, `target`,
/// `dist`, `build`, `.git`), so a source file anywhere under a large
/// gitignored directory with an unrelated name (e.g. a dependency store)
/// would still be visited and returned as a candidate.
#[test]
fn source_file_candidates_does_not_walk_gitignored_directory() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), ".gitignore", "dependency-store/\n");
    write(dir.path(), "app/page.tsx", "<div />");
    write(dir.path(), "dependency-store/nested/trap.tsx", "<div />");
    git_add_all(dir.path());

    let candidates = source_file_candidates(dir.path());

    assert!(candidates.contains(&dir.path().join("app/page.tsx")));
    assert!(!candidates
        .iter()
        .any(|path| path.starts_with(dir.path().join("dependency-store"))));
}

/// Skip directories can be git-tracked on purpose (a fixture deliberately
/// committing a `node_modules` entry to prove it is still skipped), so the
/// skip-directory check must run on the git-derived candidate list too, not
/// only during a live filesystem walk.
#[test]
fn source_file_candidates_excludes_git_tracked_skip_dir() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "app/page.tsx", "<div />");
    write(dir.path(), "node_modules/pkg/page.tsx", "<div />");
    git_add_all(dir.path());

    let candidates = source_file_candidates(dir.path());

    assert!(candidates.contains(&dir.path().join("app/page.tsx")));
    assert!(!candidates
        .iter()
        .any(|path| path.starts_with(dir.path().join("node_modules"))));
}

/// A file can be staged in git's index without existing on disk (e.g.
/// deleted outside of `git rm`). The git-derived path must not hand back a
/// path that cannot actually be read.
#[test]
fn source_file_candidates_skips_missing_git_tracked_file() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "app/page.tsx", "<div />");
    git_add_all(dir.path());
    std::fs::remove_file(dir.path().join("app/page.tsx")).unwrap();

    let candidates = source_file_candidates(dir.path());

    assert!(!candidates.contains(&dir.path().join("app/page.tsx")));
}

/// Symlinked source files are supported on the git-derived path too (git
/// tracks symlinks as their own blob type), matching the previous
/// `WalkDir`-only behavior of following a symlink to a file.
#[test]
#[cfg(unix)]
fn source_file_candidates_includes_git_tracked_symlink_to_file() {
    let dir = TempDir::new().unwrap();
    git_init(dir.path());
    write(dir.path(), "app/target.tsx", "<div />");
    std::os::unix::fs::symlink(
        dir.path().join("app/target.tsx"),
        dir.path().join("app/linked.tsx"),
    )
    .unwrap();
    git_add_all(dir.path());

    let candidates = source_file_candidates(dir.path());

    assert!(candidates.contains(&dir.path().join("app/linked.tsx")));
}

/// Outside a git repository, `source_file_candidates` still falls back to
/// the raw `WalkDir` walk, exercising its skip-dir pruning and file-type
/// checks directly since there is no git-visible file list to derive
/// candidates from.
#[test]
fn source_file_candidates_falls_back_to_walk_outside_git_repositories() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "app/page.tsx", "<div />");
    write(dir.path(), "dist/ignored.tsx", "<div />");

    let candidates = source_file_candidates(dir.path());

    assert!(candidates.contains(&dir.path().join("app/page.tsx")));
    assert!(!candidates
        .iter()
        .any(|path| path.starts_with(dir.path().join("dist"))));
}
