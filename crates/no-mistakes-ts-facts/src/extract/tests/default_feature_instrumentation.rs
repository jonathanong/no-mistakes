use super::saved_assignment_facts;

// Default-feature test compilation is the protected configuration: production
// replacement facts still hold when instrumentation counters are not fields.
#[test]
fn default_feature_tests_compile_without_instrumentation_counters() {
    let discarded = saved_assignment_facts("discarded-callable-staging.mts", false);
    assert!(discarded.value_flow.replacements.is_none());
    assert!(discarded.value_flow.callable_assignments().is_empty());
    let mixed = saved_assignment_facts("mixed-callable-receiver.mts", true);
    let capsule = mixed.value_flow.replacements.as_ref().unwrap();
    assert!(!capsule.callable_only);
    assert!(capsule.requires_receiver_proof());
    assert!(!capsule.writes.is_empty());
    assert_eq!(mixed.value_flow.callable_assignments().len(), 1);
    assert!(mixed.value_flow.callable_assignment_is_used(0, "callback"));
}
