use std::path::PathBuf;

#[test]
fn queue_priority_mix_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/priority-mix/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_priority_mix",
        &[("mixed", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["mixed"]["report"];
    assert_eq!(reports["mixed"]["exitCode"], 1);
    let findings = report["queues"].as_array().unwrap();
    assert!(findings
        .iter()
        .all(|finding| finding["kind"] == "mixed-priority"));
    assert!(findings.iter().all(|finding| finding["file"] != "notes.ts"));
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding["file"] == "labels.ts")
            .count(),
        1
    );
    assert_eq!(findings.len(), 19, "{findings:?}");
    assert!(report["warnings"].as_array().unwrap().is_empty());
    for key in ["rules", "codebase", "integration", "react", "advisories"] {
        assert!(
            report[key].as_array().unwrap().is_empty(),
            "{key}: {report}"
        );
    }
}

#[test]
fn queue_priority_scope_order_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/priority-scope-order/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_priority_scope_order",
        &[("scope", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["scope"]["report"];
    let findings = report["queues"].as_array().unwrap();
    assert_eq!(reports["scope"]["exitCode"], 1);
    assert_eq!(findings.len(), 7, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding["kind"] == "mixed-priority"));
    for name in ["fnParams", "arrowParams", "pattern", "switchOuter"] {
        assert!(
            findings.iter().any(|finding| finding["queueName"] == name),
            "{name}"
        );
    }
    assert!(findings.iter().all(|finding| !matches!(
        finding["queueName"].as_str(),
        Some("switchScoped" | "crossCase")
    )));
    let outer = findings
        .iter()
        .find(|finding| finding["queueName"] == "switchOuter")
        .unwrap();
    let message = outer["message"].as_str().unwrap();
    assert!(message.contains("inDiscriminant, priority 5"), "{message}");
    assert!(message.contains("afterSwitch, priority 5"), "{message}");
}
