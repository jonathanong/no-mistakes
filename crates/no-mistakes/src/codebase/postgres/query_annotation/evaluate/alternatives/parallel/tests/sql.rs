use super::*;
pub(super) fn cyclic<F>(evaluator: &Evaluator<'_, F>, opaque: bool) {
    assert_eq!(evaluator.events.len(), 1);
    let events = evaluator.events.values().next().unwrap();
    assert_eq!(events.len(), 2);
    let sql = events
        .iter()
        .map(|(_, value)| {
            let value = if opaque {
                let Value::Possible(values) = value.clone().exposed() else {
                    panic!("retained uncertain SQL candidate")
                };
                assert_eq!(values.len(), 1);
                values[0].clone().exposed()
            } else {
                value.clone().exposed()
            };
            let Value::Prefix(sql, true, _) = value else {
                panic!("concrete SQL candidate")
            };
            sql
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        sql,
        std::collections::BTreeSet::from(["SELECT 301".into(), "/* safe */ SELECT 302".into()])
    );
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn configured_cycle_has_a_concrete_finding_with_observer_off_and_on() {
    let root = support::root()
        .parent()
        .unwrap()
        .join("helper-tracing-arm-parallel-factories");
    let off = crate::benchmark_support::check_json(&root).unwrap();
    let (on, _) = crate::benchmark_support::check_json_observed(&root, true).unwrap();
    assert_eq!(off, on);
    let report: serde_json::Value = serde_json::from_str(&off).unwrap();
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1);
    assert!(report["warnings"].as_array().unwrap().is_empty());
    assert!(report["advisories"].as_array().unwrap().is_empty());
    assert!(findings
        .iter()
        .any(|finding| finding["file"] == "src/cyclic.mts"
            && finding["line"] == 7
            && finding["rule"] == "postgres-require-query-annotation"));
}
