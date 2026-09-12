use super::walk_files;
use crate::codebase::ts_source::relative_slash_path;
use crate::test_support::materialize_gitignore_fixture;

fn ancestor_star_repo(with_worktree_git: bool) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = materialize_gitignore_fixture("ancestor-star-ignore");
    let repo = dir.path().join("repo");
    if with_worktree_git {
        std::fs::rename(repo.join(".git.fixture"), repo.join(".git")).unwrap();
    }
    (dir, repo)
}

#[test]
fn ancestor_star_gitignore_does_not_hide_ad_hoc_directory_files() {
    let (_dir, repo) = ancestor_star_repo(false);
    let files: Vec<String> = walk_files(&repo, &[])
        .iter()
        .map(|path| relative_slash_path(&repo, path))
        .collect();
    assert!(files.contains(&"src/visible.mts".to_string()), "{files:?}");
}

#[test]
fn ancestor_star_gitignore_does_not_hide_git_worktree_files() {
    let (_dir, repo) = ancestor_star_repo(true);
    let files: Vec<String> = walk_files(&repo, &[])
        .iter()
        .map(|path| relative_slash_path(&repo, path))
        .collect();
    assert!(files.contains(&"src/visible.mts".to_string()), "{files:?}");
}
