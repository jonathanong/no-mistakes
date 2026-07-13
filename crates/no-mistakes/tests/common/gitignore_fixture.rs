use std::path::PathBuf;

pub fn materialize(name: &str) -> tempfile::TempDir {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/gitignore")
        .join(name);
    let destination = tempfile::TempDir::new().expect("create fixture destination");
    for entry in ignore::WalkBuilder::new(&source)
        .hidden(false)
        .require_git(false)
        .build()
        .map(Result::unwrap)
        .filter(|entry| entry.path() != source)
    {
        let relative = entry.path().strip_prefix(&source).unwrap();
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
    let gitignore_fixture = destination.path().join(".gitignore.fixture");
    std::fs::rename(gitignore_fixture, destination.path().join(".gitignore")).unwrap();
    destination
}
