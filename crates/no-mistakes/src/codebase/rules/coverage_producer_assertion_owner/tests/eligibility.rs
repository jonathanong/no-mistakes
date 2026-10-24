#[test]
fn execution_eligibility_is_explicit_fact_demand() {
    use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
    let file = fixture().join("src/producers.mts");
    let plain = TsFactPlan {
        function_calls: true,
        value_flow: true,
        ..Default::default()
    };
    let off = collect_ts_facts(std::slice::from_ref(&file), plain);
    let on = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            call_eligibility: true,
            ..plain
        },
    );
    assert!(off.get(&file).unwrap().unreachable_call_ranges.is_empty());
    let collected = on.get(&file).unwrap();
    assert!(!collected.unreachable_call_ranges.is_empty());
    let nested = collected
        .callable_scope_ids
        .iter()
        .find(|(_, name)| name == "nestedDeadRanges")
        .unwrap()
        .0;
    assert_eq!(
        collected
            .unreachable_call_ranges
            .iter()
            .filter(|(owner, _, _)| *owner == Some(nested))
            .count(),
        1,
        "nested same-owner ranges merge once during collection"
    );
    assert!(collected
        .unreachable_call_ranges
        .windows(2)
        .all(|pair| pair[0].0 != pair[1].0 || pair[0].2 < pair[1].1));
    assert_eq!(
        off.get(&file).unwrap().function_calls,
        on.get(&file).unwrap().function_calls
    );
}
