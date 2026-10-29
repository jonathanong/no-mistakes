use super::support::{assert_markers, fixture_with_effects};

#[test]
fn primitive_stability_cache_preserves_shared_stable_and_mutated_throw_helpers() {
    let (source, _, heads) = fixture_with_effects("primitive-stability-cache.mts");
    assert_eq!(assert_markers(&source, &heads), 16);
}

#[test]
fn primitive_stability_cache_scans_each_exact_owner_once_including_rejections() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let (source, ts, heads) = crate::diagnostics::with_observer(Some(observer.clone()), || {
        fixture_with_effects("primitive-stability-cache.mts")
    });
    assert_eq!(assert_markers(&source, &heads), 16);
    let module = ts.module_bindings.as_ref().unwrap();
    let throws = module
        .bindings
        .iter()
        .filter(|owner| owner.name == "impossible")
        .collect::<Vec<_>>();
    assert_eq!(throws.len(), 2);
    assert_ne!(throws[0].id, throws[1].id);
    assert_ne!(throws[0].scope_id, throws[1].scope_id);
    assert_eq!(throws[0].references.len(), 8);
    assert_eq!(throws[1].references.len(), 9);
    let producers = module
        .bindings
        .iter()
        .filter(|owner| owner.name.starts_with("producer"))
        .collect::<Vec<_>>();
    assert_eq!(producers.len(), 16);
    assert!(producers.iter().all(|owner| owner.references.len() == 1));
    let work = observer.snapshot().work;
    for (metric, expected) in [
        ("postgres.annotation.primitive_stability_censuses", 18),
        ("postgres.annotation.primitive_stability_refs_visited", 33),
        (
            "postgres.annotation.primitive_stability_positive_censuses",
            17,
        ),
        (
            "postgres.annotation.primitive_stability_negative_censuses",
            1,
        ),
    ] {
        assert_eq!(
            work.get(metric).copied(),
            Some(expected),
            "{metric}: {work:#?}"
        );
    }
}
