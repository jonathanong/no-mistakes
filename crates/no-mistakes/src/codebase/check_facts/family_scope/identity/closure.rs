use super::{closure_from, fixture, FamilyPathScope};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::discover_files;

#[test]
fn import_closure_falls_back_to_all_on_path_aliases() {
    let root = fixture("check-facts/family-scope-alias");
    assert_eq!(closure_from(&root, "src/entry.ts"), FamilyPathScope::All);
}

#[test]
fn import_closure_falls_back_to_all_on_scoped_package_specifiers() {
    let root = fixture("check-facts/family-scope-alias");
    assert_eq!(closure_from(&root, "src/scoped.ts"), FamilyPathScope::All);
}

#[test]
fn import_closure_keeps_relative_cycles_restricted() {
    let root = fixture("check-facts/family-scope-cycle");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let start = normalize_path(&root.join("src/a.ts"));
    let scope = super::super::closure::with_import_closure(
        &root,
        &files,
        sources.as_ref(),
        [start].into_iter().collect(),
    );
    assert!(matches!(scope, FamilyPathScope::Only(_)), "{scope:?}");
}

#[test]
fn empty_import_seeds_stay_all() {
    let root = fixture("check-facts/family-scope-alias");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    assert_eq!(
        super::super::closure::with_import_closure(
            &root,
            &files,
            sources.as_ref(),
            Default::default()
        ),
        FamilyPathScope::All
    );
}
