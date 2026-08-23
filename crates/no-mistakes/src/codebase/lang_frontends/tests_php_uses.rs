use super::*;
use std::path::PathBuf;

fn store_for(files: &[PathBuf]) -> crate::codebase::ts_source::SourceStore {
    crate::codebase::ts_source::SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(files),
    ))
}

fn files_under(root: &std::path::Path) -> Vec<PathBuf> {
    let repo = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    );
    crate::codebase::ts_source::discover_visible_paths(&repo)
        .into_iter()
        .map(|path| {
            let absolute = if path.is_absolute() {
                path
            } else {
                repo.join(path)
            };
            crate::codebase::ts_resolver::normalize_path(&absolute)
        })
        .filter(|path| path.starts_with(root))
        .collect()
}

#[test]
fn php_collects_alias_and_group_use_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/php-use-aliases"),
    );
    let files = files_under(&root);
    let store = store_for(&files);
    let facts = collect_php_facts(&root, &files, &[".".into()], Some("laravel"), &store);
    let uses = facts
        .files
        .values()
        .find(|file| file.path.ends_with("Uses.php"))
        .expect("uses");
    assert!(uses
        .imports
        .iter()
        .any(|import| import == "Job=App.Jobs.SomeJob"));
    assert!(uses
        .imports
        .iter()
        .any(|import| import == "Dto=App.Dto.UserDto"));
    assert!(uses
        .imports
        .iter()
        .any(|import| import == "App.Dto.Missing"));
}
