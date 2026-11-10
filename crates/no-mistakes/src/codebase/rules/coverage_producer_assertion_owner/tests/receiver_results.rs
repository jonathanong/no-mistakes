use serde_json::Value;

fn report() -> Value {
    let root = crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins");
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root,
        "config": root.join("review-receiver-results.yml")
    }));
    serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap()
}

fn finding<'a>(findings: &'a [Value], target: &str) -> &'a Value {
    findings
        .iter()
        .find(|finding| finding["target"] == target)
        .unwrap_or_else(|| panic!("missing {target}: {findings:#?}"))
}

fn cause_count(finding: &Value) -> usize {
    finding
        .get("causes")
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

#[test]
fn remaining_builtin_receivers_drop_unresolved_dispatches() {
    let report = report();
    let findings = report["rules"].as_array().unwrap();
    let resolved = [
        "shards",
        "isoStamp",
        "boundDate",
        "asyncShutdown",
        "localResults",
        "boundRows",
        "moduleFailures",
        "localRows",
    ];
    let resolved_causes: usize = resolved
        .iter()
        .map(|target| cause_count(finding(findings, target)))
        .sum();
    assert_eq!(
        resolved_causes, 0,
        "these receivers were unresolved dispatches before result and mutated-array proof: {report}"
    );
    assert!(
        findings
            .iter()
            .all(|finding| finding["target"] != "provenCallback"),
        "a caller-proven callback parameter reaches the assertion: {findings:#?}"
    );
    let untyped = finding(findings, "untypedMap");
    assert_eq!(cause_count(untyped), 1, "{untyped}");
    assert_eq!(untyped["causes"][0]["callee"], "values.map");
    let unproven = finding(findings, "unprovenCallback");
    assert_eq!(cause_count(unproven), 1, "{unproven}");
    assert_eq!(unproven["causes"][0]["callee"], "fn");
}
