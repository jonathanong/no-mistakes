use super::*;

#[test]
fn deferred_writer_preflight_rejects_a_budget_too_small_for_its_sources() {
    let file = root().join("../../../value-flow-forward-receiver-alias/fixture/src/budget-128.mts");
    for limit in [0, 1] {
        let (facts, work) = collect(&file, Some(limit), true);
        assert!(facts.value_flow.incomplete_reason.is_some());
        assert!(facts.value_flow.bindings.is_empty());
        assert_eq!(work, [0, 1]);
    }
}

#[test]
fn deferred_forward_work_is_charged_and_stops_at_the_file_budget() {
    for writers in [128, 256] {
        let file = root().join(format!(
            "../../../value-flow-forward-receiver-alias/fixture/src/budget-{writers}.mts"
        ));
        let (bounded, work) = collect(&file, Some(5000), true);
        assert!(bounded.value_flow.incomplete_reason.is_some());
        assert!(bounded.value_flow.bindings.is_empty());
        assert!(work[0] <= 5000);
        assert_eq!(work[1], 1);
        let (unlimited, work) = collect(&file, Some(usize::MAX), true);
        assert!(unlimited.value_flow.incomplete_reason.is_none());
        assert!(work[0] > 5000);
    }
}

#[test]
fn forward_parameter_controls_keep_identical_facts() {
    for name in ["parameter-live", "parameter-object", "grouped", "scale-64"] {
        let file = root().join(format!(
            "../../../value-flow-forward-receiver-alias/fixture/src/{name}.mts"
        ));
        let bounded = collect(&file, None, true).0;
        assert!(bounded.value_flow.incomplete_reason.is_none(), "{name}");
        assert_eq!(bounded, collect(&file, Some(usize::MAX), true).0, "{name}");
    }
}
