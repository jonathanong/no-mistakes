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
fn import_closure_keeps_dotted_module_names() {
    let root = fixture("check-facts/family-scope-dotted");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let start = normalize_path(&root.join("src/entry.ts"));
    let service = normalize_path(&root.join("src/user.service.ts"));
    let scope = super::super::closure::with_import_closure(
        &root,
        &files,
        sources.as_ref(),
        [start.clone()].into_iter().collect(),
    );
    assert!(
        matches!(&scope, FamilyPathScope::Only(_))
            && crate::codebase::check_facts::family_scope::applies(&scope, &start)
            && crate::codebase::check_facts::family_scope::applies(&scope, &service),
        "specifier ./user.service must resolve to user.service.ts, not user.ts: {scope:?}"
    );
}

#[test]
fn import_closure_follows_backtick_and_multiline_call_specifiers() {
    let root = fixture("check-facts/family-scope-dynamic-import");
    let files = discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let start = normalize_path(&root.join("src/entry.ts"));
    let scope = super::super::closure::with_import_closure(
        &root,
        &files,
        sources.as_ref(),
        [start.clone()].into_iter().collect(),
    );
    let included = [
        "src/backtick.ts",
        "src/multiline.ts",
        "src/spaced.ts",
        "src/required.ts",
    ];
    let unrelated = normalize_path(&root.join("src/unrelated.ts"));
    assert!(
        matches!(&scope, FamilyPathScope::Only(_))
            && included.iter().all(|rel| {
                crate::codebase::check_facts::family_scope::applies(
                    &scope,
                    &normalize_path(&root.join(rel)),
                )
            })
            && !crate::codebase::check_facts::family_scope::applies(&scope, &unrelated),
        "import()/require() must keep backtick, whitespace, and multiline specifiers: {scope:?}"
    );
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
