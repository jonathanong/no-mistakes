use super::*;
use crate::codebase::check_facts::FamilyPathScope;
use std::cell::Cell;
use std::path::PathBuf;

thread_local! {
    pub(super) static NORMALIZATIONS: Cell<usize> = const { Cell::new(0) };
}

macro_rules! record_scope_normalization {
    () => {
        $crate::integration_tests::config::discovery_scope::tests::NORMALIZATIONS
            .with(|count| count.set(count.get() + 1));
    };
}

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests/discovery-scope/fixture"),
    )
}

fn scope(root: &Path, files: &[PathBuf], config: &str) -> (FamilyPathScope, usize) {
    let config = crate::config::v2::load_v2_config(root, Some(&root.join(config))).unwrap();
    NORMALIZATIONS.with(|count| count.set(0));
    let selected = test_discovery_scope(root, files, &config);
    (selected, NORMALIZATIONS.with(Cell::get))
}

#[test]
fn explicit_scope_selects_all_empty_and_middle_excluded_files() {
    let root = fixture();
    let discovered = crate::codebase::ts_source::discover_files(&root, &[]);
    let inventory = crate::codebase::ts_source::FileInventory::from_paths(&discovered);
    let files = inventory.paths();
    let (all, count) = scope(&root, &files, "all-files.yml");
    assert_eq!(all, FamilyPathScope::All);
    assert_eq!(count, files.len());
    let excluded = root.join("helpers/dependency.ts");
    let index = files.iter().position(|path| path == &excluded).unwrap();
    assert!(index > 0 && index + 1 < files.len());
    let (selected, count) = scope(&root, &files, "middle-exclude.yml");
    let FamilyPathScope::Only(paths) = selected else {
        panic!("expected explicit subset")
    };
    assert_eq!(count, files.len() - 1);
    assert_eq!(paths.len(), files.len() - 1);
    assert!(!paths.contains(&excluded));
    assert!(paths.contains(&files[index - 1]));
    assert!(paths.contains(&files[index + 1]));
    let (empty, count) = scope(&root, &files, "empty-scope.yml");
    assert!(matches!(empty, FamilyPathScope::Only(paths) if paths.is_empty()));
    assert_eq!(count, 0);
}
