use super::VisiblePathLookup;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn sample_visible() -> HashSet<PathBuf> {
    HashSet::from([PathBuf::from("/fixture/a.ts"), PathBuf::from("/fixture/b.ts")])
}

#[test]
fn reference_and_arc_lookups_delegate_to_the_inner_universe() {
    let visible = sample_visible();
    let by_ref: &HashSet<PathBuf> = &visible;
    let by_arc = Arc::new(visible.clone());
    let path = Path::new("/fixture/a.ts");
    let missing = Path::new("/fixture/missing.ts");
    let expected_key = vec![PathBuf::from("/fixture/a.ts"), PathBuf::from("/fixture/b.ts")];

    assert!(<&HashSet<PathBuf> as VisiblePathLookup>::contains_visible(
        &by_ref, path
    ));
    assert!(!<&HashSet<PathBuf> as VisiblePathLookup>::contains_visible(
        &by_ref, missing
    ));
    assert_eq!(
        <&HashSet<PathBuf> as VisiblePathLookup>::visible_len(&by_ref),
        2
    );
    assert_eq!(
        <&HashSet<PathBuf> as VisiblePathLookup>::visible_cache_key(&by_ref),
        expected_key
    );

    assert!(<Arc<HashSet<PathBuf>> as VisiblePathLookup>::contains_visible(&by_arc, path));
    assert!(!<Arc<HashSet<PathBuf>> as VisiblePathLookup>::contains_visible(
        &by_arc, missing
    ));
    assert_eq!(
        <Arc<HashSet<PathBuf>> as VisiblePathLookup>::visible_len(&by_arc),
        2
    );
    assert_eq!(
        <Arc<HashSet<PathBuf>> as VisiblePathLookup>::visible_cache_key(&by_arc),
        expected_key
    );
}

#[test]
fn path_set_lookups_match_hash_set_membership() {
    let visible: crate::fx::PathSet = sample_visible().into_iter().collect();
    let path = Path::new("/fixture/a.ts");
    let missing = Path::new("/fixture/missing.ts");

    assert!(VisiblePathLookup::contains_visible(&visible, path));
    assert!(!VisiblePathLookup::contains_visible(&visible, missing));
    assert_eq!(VisiblePathLookup::visible_len(&visible), 2);
    assert_eq!(
        VisiblePathLookup::visible_cache_key(&visible),
        vec![PathBuf::from("/fixture/a.ts"), PathBuf::from("/fixture/b.ts")]
    );
}

#[test]
fn import_resolver_with_visible_stays_a_public_pathset_api() {
    let source = include_str!("resolver_impl.rs");
    assert!(
        source.contains("pub fn with_visible(self, visible: &'a crate::fx::PathSet)"),
        "ImportResolver::with_visible must stay a public PathSet API"
    );
}

