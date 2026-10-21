use super::*;

#[test]
fn relative_primary_hop_admits_outside_helper_and_caches_failures() {
    let fixture = Fixture::new("workspace-fallback-closure-unit");
    let entry = fixture.path("src/a.mts");
    let helper = fixture.path("src/helper.mts");
    let seeds = PathSet::from_iter([entry.clone()]);
    let calls = RefCell::new(BTreeMap::new());
    let mut fallback = fixture.fallback(Some(&seeds));
    fallback.prepare(&fixture.files, |specifier, from| {
        *calls
            .borrow_mut()
            .entry((from.to_path_buf(), specifier.to_owned()))
            .or_insert(0) += 1;
        (from == entry && specifier == "./helper.mts").then(|| helper.clone())
    });
    assert!(!seeds.contains(&helper));
    assert!(fallback.importers.as_deref().unwrap().contains(&helper));
    assert_eq!(fallback.prepared.len(), 3);
    assert_eq!(calls.borrow().len(), 3);
    assert!(calls.borrow().values().all(|count| *count == 1));
    assert!(matches!(
        fallback.prepared[&helper]["@p/missing"],
        ResolvedTarget::Primary(None)
    ));
    assert_cached(&fallback);
}

#[test]
fn invisible_primary_hit_vetoes_workspace_replacement_and_traversal() {
    let fixture = Fixture::new("workspace-fallback-closure-unit");
    let helper = fixture.path("src/helper.mts");
    let seeds = PathSet::from_iter([helper.clone()]);
    let mut fallback = fixture.fallback(Some(&seeds));
    fallback.prepare(&fixture.files, |specifier, _| {
        (specifier == "@p/leaf").then(|| fixture.path("invisible.mts"))
    });
    assert_eq!(fallback.prepared.len(), fixture.files.len());
    assert_eq!(fallback.importers.as_deref().unwrap().len(), 1);
    assert!(matches!(
        fallback.prepared[&helper]["@p/leaf"],
        ResolvedTarget::Primary(None)
    ));
    assert_cached(&fallback);
}
