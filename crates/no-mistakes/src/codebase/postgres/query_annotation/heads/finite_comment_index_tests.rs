//! Construction counters lock joins to one file pass and one domain per helper.
use super::super::support::{assert_markers, fixture_with_effects};
use crate::diagnostics::{with_observer, InvocationObserver};
#[test]
fn finite_comment_indexes_do_not_rescan_calls_or_helpers_for_each_root() {
    let observer = InvocationObserver::new(true);
    let (source, ts, heads) = with_observer(Some(observer.clone()), || {
        fixture_with_effects("finite-comment-index-work.mts")
    });
    assert_eq!(assert_markers(&source, &heads), 64);
    let work = observer.snapshot().work;
    let module = ts.module_bindings.as_ref().unwrap();
    for (metric, expected) in [
        ("postgres.annotation.finite_index_builds", 1),
        ("postgres.annotation.finite_index_call_visits", 647),
        ("postgres.annotation.finite_index_helper_call_visits", 583),
        ("postgres.annotation.finite_index_function_visits", 2),
        ("postgres.annotation.finite_index_select_visits", 2),
        ("postgres.annotation.finite_helper_domain_censuses", 2),
        ("postgres.annotation.finite_stability_censuses", 8),
        ("postgres.annotation.finite_reference_visits", 80),
        ("postgres.annotation.finite_root_endpoint_visits", 32),
    ] {
        assert_eq!(work.get(metric).copied(), Some(expected), "{metric}");
    }
    assert_eq!(
        work["postgres.annotation.finite_index_binding_visits"],
        module.bindings.len() as u64
    );
    assert_eq!(
        work["postgres.annotation.finite_index_export_visits"],
        module.exports.len() as u64
    );
}

#[test]
fn direct_finite_templates_skip_unrelated_root_and_helper_tables() {
    let observer = InvocationObserver::new(true);
    let (source, _, heads) = with_observer(Some(observer.clone()), || {
        fixture_with_effects("finite-comment-direct-index-work.mts")
    });
    assert_eq!(assert_markers(&source, &heads), 1);
    let work = observer.snapshot().work;
    assert_eq!(
        work.get("postgres.annotation.finite_index_builds").copied(),
        Some(1)
    );
    assert_eq!(
        work.get("postgres.annotation.finite_stability_censuses")
            .copied(),
        Some(1)
    );
    for metric in [
        "postgres.annotation.finite_index_call_visits",
        "postgres.annotation.finite_index_helper_call_visits",
        "postgres.annotation.finite_index_function_visits",
        "postgres.annotation.finite_index_select_visits",
        "postgres.annotation.finite_index_binding_visits",
        "postgres.annotation.finite_helper_domain_censuses",
        "postgres.annotation.finite_root_endpoint_visits",
    ] {
        assert_eq!(work.get(metric).copied().unwrap_or(0), 0, "{metric}");
    }
}
