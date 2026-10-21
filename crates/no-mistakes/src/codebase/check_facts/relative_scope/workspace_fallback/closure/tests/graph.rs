use super::*;

#[test]
fn seed_order_is_irrelevant_and_diamond_cycle_modules_resolve_once() {
    let fixture = Fixture::new("workspace-fallback-transitive-graph");
    let first = fixture.path("src/a.mts");
    let second = fixture.path("packages/right/src/index.mts");
    let mut previous = None;
    for order in [
        [first.clone(), second.clone()],
        [second.clone(), first.clone()],
    ] {
        let seeds = order.into_iter().collect();
        let calls = RefCell::new(Vec::new());
        let mut fallback = fixture.fallback(Some(&seeds));
        fallback.prepare(&fixture.files, |specifier, from| {
            calls
                .borrow_mut()
                .push((from.to_path_buf(), specifier.to_owned()));
            None
        });
        let trace = calls.into_inner();
        let mut counts = BTreeMap::new();
        for edge in &trace {
            *counts.entry(edge).or_insert(0) += 1;
        }
        assert_eq!(
            counts.len(),
            7,
            "duplicate import/binding/reexport facts resolve once"
        );
        assert!(counts.values().all(|count| *count == 1));
        assert_eq!(
            fallback.prepared.len(),
            5,
            "diamond and cycle visit five modules once"
        );
        assert_eq!(fallback.importers.as_deref().unwrap().len(), 5);
        assert!(!fallback
            .prepared
            .contains_key(&fixture.path("packages/type-only/src/index.mts")));
        assert!(!trace
            .iter()
            .any(|(_, specifier)| specifier == "@p/type-only" || specifier == "@p/noise"));
        assert_cached(&fallback);
        if let Some(previous) = &previous {
            assert_eq!(&trace, previous);
        }
        previous = Some(trace);
    }
}

#[test]
fn seed_without_prepared_facts_is_retained_without_resolution() {
    let fixture = Fixture::new("workspace-fallback-transitive-chain");
    let missing = fixture.path("missing.mts");
    let seeds = PathSet::from_iter([missing.clone()]);
    let mut fallback = fixture.fallback(Some(&seeds));
    fallback.prepare(&fixture.files, |_, _| {
        panic!("missing facts cannot resolve imports")
    });
    assert!(fallback.prepared.is_empty());
    assert!(fallback.importers.as_deref().unwrap().contains(&missing));
}
