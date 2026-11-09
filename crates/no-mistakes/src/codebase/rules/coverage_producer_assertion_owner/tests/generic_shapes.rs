use serde_json::Value;

fn fixture() -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("coverage-producer-assertion-owner-builtins")
}

fn report(config: &str) -> Value {
    let root = fixture();
    let input = crate::napi_api::options::test_json_arg(serde_json::json!({
        "root": root,
        "config": root.join(config)
    }));
    serde_json::from_str(&crate::napi_api::check_json_impl(input).unwrap()).unwrap()
}

fn finding<'a>(findings: &'a [Value], target: &str) -> &'a Value {
    findings
        .iter()
        .find(|finding| finding["target"] == target)
        .unwrap_or_else(|| panic!("missing {target}: {findings:#?}"))
}

#[test]
fn generic_native_shapes_reduce_unresolved_dispatches() {
    let report = report("review-generic-shapes.yml");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 4, "{report}");

    let script = finding(findings, "scriptRun");
    let script_causes = script
        .get("causes")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    assert_eq!(
        script_causes, 0,
        "the frozen 73e baseline has two dispatches here: {script}"
    );
    assert_eq!(
        findings
            .iter()
            .filter_map(|finding| finding.get("causes").and_then(Value::as_array))
            .map(Vec::len)
            .sum::<usize>(),
        2,
        "the frozen 73e baseline has five dispatches across these same four findings: {report}"
    );
    let explicitly_unknown = finding(findings, "explicitUnknown");
    assert!(
        explicitly_unknown
            .get("causes")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty),
        "explicit unknown receiver is a non-dispatching built-in leaf: {explicitly_unknown}"
    );

    let wrong_receiver = finding(findings, "wrongKnownReceiver");
    assert_eq!(
        wrong_receiver["causes"][0]["callee"], "values.map",
        "a known Set receiver must not inherit Array.map callback proof: {wrong_receiver}"
    );
    let unknown_caller = finding(findings, "unresolvedWithoutCaller");
    assert_eq!(unknown_caller["causes"][0]["callee"], "callback");
    assert!(
        findings
            .iter()
            .all(|finding| finding["target"] != "customMethod"),
        "the custom helper reaches the assertion: {findings:#?}"
    );
}

#[test]
fn generic_native_shape_uncertainty_obeys_report_policy() {
    let advise = report("review-generic-shapes-advise.yml");
    let rules = advise["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 2, "{advise}");
    assert_eq!(finding(rules, "scriptRun")["target"], "scriptRun");
    finding(rules, "explicitUnknown");
    let advisories = advise["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 2, "{advise}");
    finding(advisories, "wrongKnownReceiver");
    finding(advisories, "unresolvedWithoutCaller");
    let ignored = report("review-generic-shapes-ignore.yml");
    let ignored_rules = ignored["rules"].as_array().unwrap();
    assert_eq!(ignored_rules.len(), 2, "{ignored}");
    finding(ignored_rules, "scriptRun");
    finding(ignored_rules, "explicitUnknown");
    assert!(
        ignored["advisories"].as_array().unwrap().is_empty(),
        "{ignored}"
    );
}
