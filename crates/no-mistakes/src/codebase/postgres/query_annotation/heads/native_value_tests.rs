use super::support::{assert_markers, fixture_with_effects};

#[test]
fn native_value_observations_preserve_heads_without_predicting_values() {
    let (source, _, heads) = fixture_with_effects("native-value-reads.mts");
    assert_eq!(assert_markers(&source, &heads), 4);
}

#[test]
fn native_value_observations_require_complete_private_ownership() {
    let (source, _, heads) = fixture_with_effects("native-value-reads-unsafe.mts");
    assert_eq!(assert_markers(&source, &heads), 29);
}

#[test]
fn native_constructor_runtime_escapes_disable_separation() {
    let (source, _, heads) = fixture_with_effects("native-value-reads-constructor.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}

#[test]
fn native_observation_owner_cache_keeps_negative_results() {
    let (source, _, heads) = fixture_with_effects("native-value-reads-cache.mts");
    assert_eq!(assert_markers(&source, &heads), 4);
}

#[test]
fn native_observation_proof_rejects_dynamic_binding_gaps() {
    let (source, _, heads) = fixture_with_effects("native-value-reads-gap.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}

#[test]
fn ordinary_heads_do_not_build_native_observation_ownership() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let (source, _, heads) = fixture_with_effects("direct-only.mts");
        assert_eq!(assert_markers(&source, &heads), 2);
    });
    assert!(!observer
        .snapshot()
        .work
        .keys()
        .any(|key| key.contains("native_value")));
}

#[test]
fn native_observations_need_original_semantic_owners() {
    let (source, mut ts, heads) = fixture_with_effects("native-value-reads.mts");
    let calls = super::support::decisions(&source, &heads)
        .into_iter()
        .map(|(span, _)| span)
        .collect::<Vec<_>>();
    ts.module_bindings = None;
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("low-heads/native-value-reads.mts");
    let heads = crate::ast::with_program(&path, &source, |program, _| {
        super::collect_with_helpers(
            program,
            &ts,
            calls.iter().copied(),
            &Default::default(),
            true,
            true,
        )
    })
    .unwrap();
    assert!(calls.iter().all(|span| !heads.accepts(*span)));
}
