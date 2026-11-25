use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "queue-no-enqueue-in-transaction";

#[test]
fn queue_no_enqueue_in_transaction_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "behavior");
    let handles = rules_path(RULE, "fixture", "handles-and-builtins");
    let local_exports = rules_path(RULE, "fixture", "local-exports");
    let reexports = rules_path(RULE, "fixture", "reexports");
    let incomplete = rules_path(RULE, "fixture", "incomplete");
    let unanalyzable = rules_path(RULE, "fixture", "unanalyzable");
    let directive_audit = rules_path(RULE, "fixture", "directive-audit");
    let review_gaps = rules_path(RULE, "fixture", "review-gaps");
    let reports = assert_saved_fixture_report(
        "queue_no_enqueue_in_transaction",
        &[
            ("configured", root.clone(), root.join(".no-mistakes.yml")),
            ("default_off", root.clone(), root.join("default.yml")),
            (
                "handles_and_builtins",
                handles.clone(),
                handles.join(".no-mistakes.yml"),
            ),
            (
                "local_exports",
                local_exports.clone(),
                local_exports.join(".no-mistakes.yml"),
            ),
            (
                "reexports",
                reexports.clone(),
                reexports.join(".no-mistakes.yml"),
            ),
            (
                "incomplete",
                incomplete.clone(),
                incomplete.join(".no-mistakes.yml"),
            ),
            (
                "unanalyzable_report",
                unanalyzable.clone(),
                unanalyzable.join(".no-mistakes.yml"),
            ),
            (
                "unanalyzable_advise",
                unanalyzable.clone(),
                unanalyzable.join("advise.yml"),
            ),
            (
                "unanalyzable_ignore",
                unanalyzable.clone(),
                unanalyzable.join("ignore.yml"),
            ),
            (
                "directive_audit",
                directive_audit.clone(),
                directive_audit.join(".no-mistakes.yml"),
            ),
            (
                "review_gaps",
                review_gaps.clone(),
                review_gaps.join(".no-mistakes.yml"),
            ),
        ],
        &[],
        &["--include-suppressed"],
    );

    let configured = &reports["configured"];
    assert_eq!(configured["exitCode"], 1);
    let findings = configured["report"]["rules"].as_array().unwrap();
    let mut locations: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect();
    locations.sort_unstable();
    assert_eq!(
        locations,
        [("src/flows/receipt.ts", 6), ("src/flows/receipt.ts", 9)]
    );
    assert_eq!(
        configured["report"]["suppressed"].as_array().unwrap().len(),
        3
    );

    let default = &reports["default_off"];
    assert_eq!(default["exitCode"], 0);
    assert_eq!(default["report"]["rules"], serde_json::json!([]));

    let handles = &reports["handles_and_builtins"];
    assert_eq!(handles["exitCode"], 1);
    assert_eq!(handles["report"]["advisories"], serde_json::json!([]));
    let handle_findings = handles["report"]["rules"].as_array().unwrap();
    let mut enqueue_locations = Vec::new();
    let mut handle_unanalyzable = Vec::new();
    for finding in handle_findings {
        assert_eq!(finding["rule"], RULE);
        let location = (
            finding["file"].as_str().unwrap(),
            finding["line"].as_u64().unwrap(),
        );
        match finding["target"].as_str() {
            Some("@sample/jobs#enqueue") => enqueue_locations.push(location),
            Some("unanalyzable") => handle_unanalyzable.push(location),
            other => panic!("unexpected handle target {other:?}"),
        }
    }
    enqueue_locations.sort_unstable();
    handle_unanalyzable.sort_unstable();
    assert_eq!(
        enqueue_locations,
        [
            ("src/map-enqueue.ts", 6),
            ("src/map-enqueue.ts", 9),
            ("src/map-enqueue.ts", 10),
            ("src/map-enqueue.ts", 11),
            ("src/map-enqueue.ts", 12),
            ("src/map-enqueue.ts", 13),
            ("src/map-enqueue.ts", 15),
            ("src/map-enqueue.ts", 16),
            ("src/map-enqueue.ts", 17),
            ("src/shared-owner.ts", 10),
            ("src/shared-owner.ts", 21),
        ]
    );
    assert_eq!(handle_unanalyzable, [("src/shared-owner.ts", 9)]);

    let local_exports_report = &reports["local_exports"];
    assert_eq!(local_exports_report["exitCode"], 0);
    assert_eq!(
        local_exports_report["report"]["rules"],
        serde_json::json!([])
    );
    assert_eq!(
        local_exports_report["report"]["advisories"],
        serde_json::json!([])
    );

    let reexports_report = &reports["reexports"];
    assert_eq!(reexports_report["exitCode"], 1);
    assert_eq!(
        reexports_report["report"]["advisories"],
        serde_json::json!([])
    );
    let mut reexport_hits: Vec<_> = reexports_report["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap(),
            )
        })
        .collect();
    reexport_hits.sort_unstable();
    assert_eq!(
        reexport_hits,
        [
            (
                "src/flows/const-alias-reexports.ts",
                6,
                "@sample/jobs#enqueue"
            ),
            (
                "src/flows/const-alias-reexports.ts",
                7,
                "@sample/jobs#enqueue"
            ),
            (
                "src/flows/const-alias-reexports.ts",
                8,
                "@sample/jobs#enqueue"
            ),
            ("src/flows/named-reexports.ts", 17, "@sample/jobs#enqueue"),
            ("src/flows/named-reexports.ts", 18, "@sample/jobs#enqueue"),
            ("src/flows/named-reexports.ts", 19, "@sample/jobs#enqueue"),
            ("src/flows/named-reexports.ts", 20, "@sample/jobs#enqueue"),
            ("src/flows/named-reexports.ts", 21, "@sample/jobs#enqueue"),
            ("src/flows/named-reexports.ts", 22, "@sample/jobs#enqueue"),
            ("src/flows/named-reexports.ts", 23, "@sample/jobs#enqueue"),
            ("src/flows/named-reexports.ts", 24, "@sample/jobs#enqueue"),
            ("src/flows/named-reexports.ts", 25, "@sample/jobs#enqueue"),
            (
                "src/flows/named-reexports.ts",
                33,
                "./local-jobs#enqueueLocal"
            ),
            (
                "src/flows/named-reexports.ts",
                34,
                "./local-jobs#enqueueLocal"
            ),
            ("src/flows/named-reexports.ts", 40, "@sample/jobs#enqueue"),
        ]
    );
    let incomplete = &reports["incomplete"]["report"];
    let failures = incomplete["incomplete"].as_array().unwrap();
    assert!(failures
        .iter()
        .any(|failure| { failure["consumer"] == RULE && failure["file"] == "src/broken.ts" }));
    let healthy = incomplete["rules"].as_array().unwrap();
    assert_eq!(healthy.len(), 1);
    assert_eq!(healthy[0]["file"], "src/healthy.ts");
    assert!(healthy[0]["message"]
        .as_str()
        .unwrap()
        .starts_with("Queue handoff"));

    let reported = &reports["unanalyzable_report"];
    assert_eq!(reported["exitCode"], 1);
    let report_rows = reported["report"]["rules"].as_array().unwrap();
    assert!(
        !report_rows.is_empty()
            && report_rows
                .iter()
                .all(|row| row["rule"] == RULE && row["target"] == "unanalyzable")
    );
    assert!(
        report_rows
            .iter()
            .all(|row| row["file"] != "src/flows/diamond.ts"),
        "a star-reexport diamond that never reaches a configured module must stay clean"
    );
    let advised = &reports["unanalyzable_advise"];
    assert_eq!(advised["exitCode"], 0);
    assert_eq!(advised["report"]["rules"], serde_json::json!([]));
    assert_eq!(
        advised["report"]["advisories"].as_array().unwrap().len(),
        report_rows.len()
    );
    let ignored = &reports["unanalyzable_ignore"];
    assert_eq!(ignored["exitCode"], 0);
    assert_eq!(ignored["report"]["rules"], serde_json::json!([]));
    assert_eq!(ignored["report"]["advisories"], serde_json::json!([]));

    let audit = &reports["directive_audit"];
    assert_eq!(audit["exitCode"], 1);
    let audit_rows = audit["report"]["rules"].as_array().unwrap();
    assert_eq!(audit_rows.len(), 1);
    assert_eq!(audit_rows[0]["rule"], "invalid-disable-directive");
    assert!(audit_rows[0]["message"]
        .as_str()
        .unwrap()
        .contains("disabled or out-of-scope rule `queue-no-enqueue-in-transaction`"));

    let gaps = &reports["review_gaps"];
    assert_eq!(gaps["exitCode"], 1);
    let mut gap_rows: Vec<_> = gaps["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
                finding["target"].as_str().unwrap(),
            )
        })
        .collect();
    gap_rows.sort_unstable();
    assert_eq!(
        gap_rows,
        [
            ("src/alternatives.ts", 8, "unanalyzable"),
            ("src/callback-builtins.ts", 6, "unanalyzable"),
            ("src/coverage-paths.ts", 32, "unanalyzable"),
            ("src/coverage-paths.ts", 33, "unanalyzable"),
            ("src/coverage-paths.ts", 34, "unanalyzable"),
            ("src/coverage-paths.ts", 35, "unanalyzable"),
            ("src/coverage-paths.ts", 36, "unanalyzable"),
            ("src/coverage-paths.ts", 38, "unanalyzable"),
            ("src/coverage-paths.ts", 40, "unanalyzable"),
            ("src/coverage-paths.ts", 40, "unanalyzable"),
            ("src/coverage-paths.ts", 41, "unanalyzable"),
            ("src/coverage-paths.ts", 41, "unanalyzable"),
            ("src/coverage-paths.ts", 42, "@sample/jobs#enqueue"),
            ("src/coverage-paths.ts", 43, "unanalyzable"),
            ("src/cycle-use.ts", 7, "unanalyzable"),
            ("src/cycle-use.ts", 7, "unanalyzable"),
            ("src/cycle-use.ts", 8, "@sample/jobs#enqueue"),
            ("src/fn-member.ts", 6, "unanalyzable"),
            ("src/fn-member.ts", 7, "unanalyzable"),
            ("src/json-parse.ts", 7, "@sample/jobs#enqueue"),
            ("src/member-write.ts", 11, "unanalyzable"),
            ("src/nested-handle.ts", 6, "@sample/jobs#enqueue"),
            ("src/nested-static.ts", 11, "unanalyzable"),
            ("src/reassigned.ts", 8, "unanalyzable"),
            ("src/self-ns-use.ts", 6, "@sample/jobs#enqueue"),
            ("src/self-ref.ts", 9, "@sample/jobs#enqueue"),
            ("src/structural-param.ts", 5, "unanalyzable"),
            ("src/this-queue.ts", 10, "unanalyzable"),
            ("src/this-queue.ts", 12, "@sample/jobs#enqueue"),
        ]
    );
}

#[test]
fn queue_no_enqueue_in_transaction_scoped_handles_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "scoped-handles");
    let reports = assert_saved_fixture_report(
        "queue_no_enqueue_in_transaction_scoped_handles",
        &[
            ("handles", root.clone(), root.join(".no-mistakes.yml")),
            ("mixed_advise", root.clone(), root.join("advise.yml")),
            ("callback_only", root.clone(), root.join("baseline.yml")),
        ],
        &[],
        &[],
    );

    let handles = &reports["handles"];
    assert_eq!(handles["exitCode"], 1);
    assert_eq!(handles["report"]["advisories"], serde_json::json!([]));
    let findings = handles["report"]["rules"].as_array().unwrap();
    let mut locations: Vec<_> = findings
        .iter()
        .filter(|finding| finding["target"] == "@sample/jobs#enqueue")
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect();
    locations.sort_unstable();
    assert_eq!(
        locations,
        [
            ("src/explicit.ts", 7),
            ("src/shadowed.ts", 10),
            ("src/using.ts", 8)
        ]
    );
    let mut uncertain: Vec<_> = findings
        .iter()
        .filter(|finding| finding["target"] == "unanalyzable")
        .map(|finding| finding["file"].as_str().unwrap())
        .collect();
    uncertain.sort_unstable();
    uncertain.dedup();
    assert_eq!(
        uncertain,
        ["src/conditional.ts", "src/escape.ts", "src/unresolved.ts"]
    );

    let mixed = &reports["mixed_advise"];
    assert_eq!(mixed["exitCode"], 1);
    let callback_only = &reports["callback_only"];
    assert_eq!(callback_only["exitCode"], 1);
    let callbacks = callback_only["report"]["rules"].as_array().unwrap();
    assert_eq!(callbacks.len(), 1);
    assert_eq!(callbacks[0]["file"], "src/callback.ts");
    assert_eq!(callbacks[0]["target"], "@sample/jobs#enqueue");

    let mut expected: Vec<_> = findings
        .iter()
        .filter(|finding| finding["target"] == "@sample/jobs#enqueue")
        .chain(callbacks)
        .cloned()
        .collect();
    expected.sort_by_key(serde_json::Value::to_string);
    assert_eq!(mixed["report"]["rules"], serde_json::json!(expected));
    let mut advised: Vec<_> = mixed["report"]["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            assert_eq!(finding["target"], "unanalyzable");
            finding["file"].as_str().unwrap()
        })
        .collect();
    advised.sort_unstable();
    advised.dedup();
    assert_eq!(advised, uncertain);
}
