use super::{git_commit_all, git_init, materialize_saved_fixture};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Materializes the checked-in base/head trees as adjacent commits for public
/// CLI and N-API topology-impact contract tests.
pub(crate) fn materialize_workflow_topology_impact_fixture(name: &str) -> TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/workflow-topology-impact")
        .join(name);
    let fixture = materialize_saved_fixture(&source);
    let root = fixture.path().join("base");
    git_init(&root);
    git_commit_all(&root, "base");
    replace_tree(&root, &fixture.path().join("head"));
    git_commit_all(&root, "head");
    fixture
}

fn replace_tree(root: &Path, source: &Path) {
    let git_dir = root.join(".git");
    let mut paths = ignore::WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .build()
        .map(Result::unwrap)
        .map(|entry| entry.into_path())
        .filter(|path| path != root && !path.starts_with(&git_dir))
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for path in paths {
        if path.is_dir() {
            std::fs::remove_dir(path).unwrap();
        } else {
            std::fs::remove_file(path).unwrap();
        }
    }
    copy_tree(source, root);
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in ignore::WalkBuilder::new(source)
        .hidden(false)
        .require_git(false)
        .build()
        .map(Result::unwrap)
        .filter(|entry| entry.path() != source)
    {
        let target = destination.join(entry.path().strip_prefix(source).unwrap());
        if entry.file_type().is_some_and(|kind| kind.is_dir()) {
            std::fs::create_dir_all(target).unwrap();
        } else {
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}
