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
    let warnings = report["warnings"].as_array().unwrap();
    assert_eq!(warnings.len(), 5, "{warnings:?}");
    for location in [
        "abstain.ts:36",
        "bundles-far.ts:4",
        "loops.ts:64",
        "mixed.ts:12",
        "opaque.ts:4",
    ] {
        assert!(
            warnings.iter().any(|warning| {
                let warning = warning.as_str().unwrap();
                warning.contains(location) && warning.contains("debounce-ttl was not judged")
            }),
            "{location}: {warnings:?}"
        );
    }
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

#[test]
fn queue_priority_producer_evidence_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/priority-producer-evidence/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_priority_producer_evidence",
        &[("producers", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let report = &reports["producers"]["report"];
    let findings = report["queues"].as_array().unwrap();
    assert_eq!(reports["producers"]["exitCode"], 1);
    assert!(findings
        .iter()
        .all(|finding| finding["kind"] == "mixed-priority"));
    let mut producers = findings
        .iter()
        .map(|finding| {
            (
                finding["queueName"].as_str().unwrap(),
                finding["job"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    producers.sort_unstable();
    assert_eq!(
        producers,
        [
            ("computed", "plain"),
            ("computed", "rush"),
            ("crossFileFlow", "direct"),
            ("crossFileFlow", "flowRush"),
            ("duplicate", "newName"),
            ("innerScoped", "innerPlain"),
            ("innerScoped", "innerRush"),
            ("iteratedIn", "beforeRush"),
            ("iteratedIn", "iterablePlain"),
            ("iteratedOf", "beforeRush"),
            ("iteratedOf", "iterablePlain"),
            ("localQueue", "plain"),
            ("localQueue", "rush"),
            ("numeric", "large"),
            ("numeric", "plain"),
            ("outerScoped", "outerPlain"),
            ("outerScoped", "outerRush"),
            ("redeclared", "plain"),
            ("redeclared", "rush"),
            ("sameCase", "caseRush"),
            ("sameCase", "plain"),
            ("sameLine", "plain"),
            ("sameLine", "plain"),
            ("shadowed", "plain"),
            ("staticBlock", "classMethod"),
            ("staticBlock", "duringStatic"),
        ],
    );
    assert!(findings.iter().all(|finding| {
        let queue_file = if matches!(
            finding["queueName"].as_str(),
            Some(
                "localQueue"
                    | "outerScoped"
                    | "innerScoped"
                    | "redeclared"
                    | "iteratedIn"
                    | "iteratedOf"
            )
        ) {
            "enqueue.ts"
        } else {
            "queues.ts"
        };
        finding["queueFile"] == queue_file
    }));
}

#[test]
fn queue_priority_flow_definitions_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop/priority-flow-definitions/fixture");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "queue_priority_flow_definitions",
        &[
            ("production", root.clone(), root.join(".no-mistakes.yml")),
            (
                "include-tests",
                root.clone(),
                root.join("include-tests.yml"),
            ),
        ],
        &[],
        &[],
    );
    let report = &reports["production"]["report"];
    let queues = report["queues"].as_array().unwrap();
    assert_eq!(queues.len(), 2);
    assert!(queues
        .iter()
        .all(|finding| finding["queueName"] == "orders"));
    let warnings = report["warnings"].as_array().unwrap();
    for site in ["direct.ts:3", "enqueue.ts:5", "enqueue.ts:8"] {
        assert!(
            warnings.iter().any(|warning| {
                let warning = warning.as_str().unwrap();
                warning.starts_with(&format!(
                    "queues-check: {site} mixed-priority was not judged"
                )) && warning.contains("queue \"reports\"")
            }),
            "missing abstention at {site}"
        );
    }
    assert_eq!(warnings.len(), 3);
    let inclusive = &reports["include-tests"]["report"];
    assert!(inclusive["queues"].as_array().unwrap().is_empty());
    assert_eq!(inclusive["warnings"].as_array().unwrap().len(), 5);
}
