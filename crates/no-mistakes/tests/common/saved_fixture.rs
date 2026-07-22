use std::path::{Path, PathBuf};

pub fn materialize(category: &str, name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(category)
        .join(name);
    let destination = tempfile::TempDir::new().expect("create fixture destination");
    copy_tree(&source, destination.path());
    destination
}

fn copy_tree(source: &Path, destination: &Path) {
    for entry in ignore::WalkBuilder::new(source)
        .hidden(false)
        .require_git(false)
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
