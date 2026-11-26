#[test]
fn builtin_provenance_requires_explicit_ownership_demand() {
    use crate::codebase::ts_source::facts::{collect_ts_facts, TsFactPlan};
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let file = root.join("src/proof-callbacks.mts");
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
    assert!(off.get(&file).unwrap().builtin_callback_calls.is_empty());
    assert!(!on.get(&file).unwrap().builtin_callback_calls.is_empty());
    assert_eq!(
        off.get(&file).unwrap().function_calls,
        on.get(&file).unwrap().function_calls
    );
}
