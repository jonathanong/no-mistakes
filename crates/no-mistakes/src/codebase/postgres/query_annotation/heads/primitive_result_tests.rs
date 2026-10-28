use super::support::{assert_markers, fixture_with_effects};

#[test]
fn closed_primitive_helper_result_keeps_only_total_fixed_heads() {
    let (source, _, heads) = fixture_with_effects("primitive-result.mts");
    assert_eq!(assert_markers(&source, &heads), 15);
    let (source, _, heads) = fixture_with_effects("primitive-result-dynamic.mts");
    assert_eq!(assert_markers(&source, &heads), 1);
}

#[test]
fn primitive_result_indexes_and_owner_censuses_are_lazy_and_linear() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let (source, ts, heads) = crate::diagnostics::with_observer(Some(observer.clone()), || {
        fixture_with_effects("primitive-result-owner-work.mts")
    });
    assert_eq!(assert_markers(&source, &heads), 12);
    let work = observer.snapshot().work;
    let module = ts.module_bindings.as_ref().unwrap();
    assert_eq!(work["postgres.annotation.primitive_export_indexes"], 1);
    assert_eq!(
        work["postgres.annotation.primitive_export_entries_visited"],
        (module.scopes.len() + module.exports.len()) as u64
    );
    assert_eq!(work["postgres.annotation.primitive_owner_censuses"], 7);
    // Six single-use owners plus two references for the rejected owner.
    assert_eq!(work["postgres.annotation.primitive_owner_refs_visited"], 8);
    assert_eq!(work["postgres.annotation.primitive_result_censuses"], 1);

    let observer = crate::diagnostics::InvocationObserver::new(true);
    let (source, _, heads) = crate::diagnostics::with_observer(Some(observer.clone()), || {
        fixture_with_effects("primitive-result-no-candidate.mts")
    });
    assert_eq!(assert_markers(&source, &heads), 1);
    let work = observer.snapshot().work;
    assert!(!work.contains_key("postgres.annotation.primitive_export_indexes"));
    assert!(!work.contains_key("postgres.annotation.primitive_owner_censuses"));
}
