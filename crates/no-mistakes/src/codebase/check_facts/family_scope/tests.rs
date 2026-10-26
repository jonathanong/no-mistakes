use super::{applies, merge, FamilyPathScope};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;

fn only(paths: &[&str]) -> FamilyPathScope {
    FamilyPathScope::Only(Arc::new(
        paths.iter().map(PathBuf::from).collect::<BTreeSet<_>>(),
    ))
}

#[test]
fn all_applies_to_every_path() {
    assert!(applies(
        &FamilyPathScope::All,
        PathBuf::from("a.ts").as_path()
    ));
}

#[test]
fn only_applies_to_listed_paths() {
    let scope = only(&["src/a.ts"]);
    assert!(applies(&scope, PathBuf::from("src/a.ts").as_path()));
    assert!(!applies(&scope, PathBuf::from("src/b.ts").as_path()));
}

#[test]
fn merge_keeps_a_restricted_scope_when_the_other_family_is_off() {
    let left = only(&["src/a.ts"]);
    assert_eq!(merge(true, &left, false, &FamilyPathScope::All), left);
}

#[test]
fn merge_widens_to_all_when_either_enabled_family_is_unrestricted() {
    let left = only(&["src/a.ts"]);
    assert_eq!(
        merge(true, &left, true, &FamilyPathScope::All),
        FamilyPathScope::All
    );
}

#[test]
fn merge_reuses_the_same_only_set() {
    let left = only(&["src/a.ts"]);
    let right = left.clone();
    let merged = merge(true, &left, true, &right);
    match (&left, &merged) {
        (FamilyPathScope::Only(left), FamilyPathScope::Only(merged)) => {
            assert!(Arc::ptr_eq(left, merged));
        }
        _ => panic!("{left:?} {merged:?}"),
    }
}

#[test]
fn merge_takes_the_enabled_side_when_the_other_family_is_off() {
    let right = only(&["src/b.ts"]);
    assert_eq!(merge(false, &FamilyPathScope::All, true, &right), right);
}

#[test]
fn merge_unions_restricted_scopes() {
    let merged = merge(true, &only(&["src/a.ts"]), true, &only(&["src/b.ts"]));
    assert!(applies(&merged, PathBuf::from("src/a.ts").as_path()));
    assert!(applies(&merged, PathBuf::from("src/b.ts").as_path()));
    assert!(!applies(&merged, PathBuf::from("src/c.ts").as_path()));
}
