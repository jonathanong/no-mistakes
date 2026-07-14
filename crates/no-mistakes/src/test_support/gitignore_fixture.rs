use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub(crate) fn materialize_gitignore_fixture(name: &str) -> TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore")
        .join(name);
    let destination = materialize_saved_fixture(&source);
    let gitignore_fixture = destination.path().join(".gitignore.fixture");
    if gitignore_fixture.exists() {
        std::fs::rename(gitignore_fixture, destination.path().join(".gitignore")).unwrap();
    }
    destination
}

/// Copies a saved fixture to a per-test root so parser instrumentation from
/// parallel tests cannot observe the same absolute source paths.
pub(crate) fn materialize_saved_fixture(source: &Path) -> TempDir {
    let destination = TempDir::new().expect("create fixture destination");
    for entry in ignore::WalkBuilder::new(source)
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .require_git(false)
        .build()
        .map(Result::unwrap)
        .filter(|entry| entry.path() != source)
    {
        let relative = entry.path().strip_prefix(source).unwrap();
        let target = destination.path().join(relative);
        if entry
            .file_type()
            .is_some_and(|file_type| file_type.is_dir())
        {
            std::fs::create_dir_all(&target).unwrap();
        } else {
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
    destination
}
