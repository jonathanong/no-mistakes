use std::path::{Path, PathBuf};
use std::process::Command;

pub fn materialize_rule(category: &str, name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(category)
        .join(name)
        .join("fixture");
    let destination = tempfile::TempDir::new().expect("create fixture destination");
    copy_tree(&source, destination.path());
    git(destination.path(), &["init", "-q", "--initial-branch=main"]);
    git(destination.path(), &["add", "."]);
    destination
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in ignore::WalkBuilder::new(source)
        .hidden(false)
        .ignore(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .require_git(false)
        .parents(false)
        .build()
        .map(Result::unwrap)
        .filter(|entry| entry.path() != source)
    {
        let relative = entry.path().strip_prefix(source).unwrap();
        let target = destination.join(relative);
        if entry
            .file_type()
            .is_some_and(|file_type| file_type.is_dir())
        {
            std::fs::create_dir_all(&target).unwrap();
        } else {
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(["-C", root.to_str().unwrap()])
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
