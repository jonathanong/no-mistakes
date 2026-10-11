use std::path::PathBuf;
use std::process::Command;

fn check(rule: &str, name: &str) -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("variants")
        .join(name);
    let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", String::from_utf8_lossy(&output.stderr)))
}
#[test]
fn one_bad_alternative_is_reported_and_all_good_alternatives_are_analyzed() {
    for rule in [
        "postgres-bounded-statements",
        "postgres-explicit-columns",
        "postgres-idempotent-insert",
        "postgres-generated-column-predicates",
        "postgres-sql-shape-policy",
        "postgres-sql-statement-policy",
        "postgres-required-predicates",
        "postgres-lock-ordering",
        "postgres-conflict-ordering",
        "postgres-no-offset",
        "postgres-no-generated-column-writes",
        "postgres-require-query-annotation",
    ] {
        let bad = check(rule, "one-bad");
        let findings = bad["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{rule}: {bad}");
        assert!(
            !findings[0]["target"]
                .as_str()
                .unwrap()
                .contains("unanalyzable"),
            "{bad}"
        );
        let good = check(rule, "all-good");
        assert!(
            good["rules"].as_array().unwrap().is_empty(),
            "{rule}: {good}"
        );
    }
}
#[test]
fn suppression_inside_a_branch_fragment_uses_its_physical_line() {
    let report = check("postgres-bounded-statements", "suppressed");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
}

#[test]
fn direct_consumers_and_writes_preserve_branch_and_executor_suppressions() {
    for rule in [
        "postgres-lock-ordering",
        "postgres-conflict-ordering",
        "postgres-no-generated-column-writes",
        "postgres-require-query-annotation",
    ] {
        let report = check(rule, "suppressed-branch");
        assert!(
            report["rules"].as_array().unwrap().is_empty(),
            "{rule}: {report}"
        );
    }
}

#[test]
fn executor_suppressions_cover_each_alternative_in_every_consumer() {
    for rule in [
        "postgres-bounded-statements",
        "postgres-explicit-columns",
        "postgres-idempotent-insert",
        "postgres-generated-column-predicates",
        "postgres-sql-shape-policy",
        "postgres-sql-statement-policy",
        "postgres-required-predicates",
        "postgres-lock-ordering",
        "postgres-conflict-ordering",
        "postgres-no-offset",
        "postgres-no-generated-column-writes",
        "postgres-require-query-annotation",
    ] {
        let report = check(rule, "suppressed-call");
        assert!(
            report["rules"].as_array().unwrap().is_empty(),
            "{rule}: {report}"
        );
    }
}

#[test]
fn common_offset_occurrences_are_deduplicated_across_alternatives() {
    for name in ["deduplicated", "deduplicated-prefix"] {
        let report = check("postgres-no-offset", name);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{report}");
        assert_eq!(findings[0]["target"], "offset");
        assert_eq!(findings[0]["line"], 2);
    }
    for name in ["deduplicated-multiple", "optional-offset-prefix"] {
        let report = check("postgres-no-offset", name);
        let targets = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| finding["target"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(targets, ["offset", "offset#2"], "{report}");
    }
}

#[test]
fn nested_fragment_suppressions_anchor_each_consumer_to_sql_tokens() {
    for rule in [
        "postgres-bounded-statements",
        "postgres-explicit-columns",
        "postgres-idempotent-insert",
        "postgres-generated-column-predicates",
        "postgres-sql-shape-policy",
        "postgres-sql-statement-policy",
        "postgres-required-predicates",
        "postgres-lock-ordering",
        "postgres-conflict-ordering",
        "postgres-no-offset",
        "postgres-no-generated-column-writes",
        "postgres-require-query-annotation",
    ] {
        let report = check(rule, "suppressed-fragment");
        assert!(
            report["rules"].as_array().unwrap().is_empty(),
            "{rule}: {report}"
        );
    }
}

#[test]
fn shared_failing_sql_tokens_are_reported_once_per_consumer() {
    for rule in [
        "postgres-bounded-statements",
        "postgres-explicit-columns",
        "postgres-idempotent-insert",
        "postgres-generated-column-predicates",
        "postgres-sql-shape-policy",
        "postgres-sql-statement-policy",
        "postgres-required-predicates",
        "postgres-lock-ordering",
        "postgres-conflict-ordering",
        "postgres-no-offset",
        "postgres-no-generated-column-writes",
        "postgres-require-query-annotation",
    ] {
        let report = check(rule, "deduplicated-common");
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            1,
            "{rule}: {report}"
        );
    }
}

#[test]
fn builder_fragment_and_complete_variant_share_only_their_actual_sql_tokens() {
    let report = check("postgres-sql-shape-policy", "fragment-overlap");
    let findings = report["rules"].as_array().unwrap();
    let lines = findings
        .iter()
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(lines, [4, 8, 12, 12], "{report}");
    assert_eq!(
        findings[2], findings[3],
        "different source tokens stay distinct"
    );
}

#[test]
fn finite_append_fragments_are_checked_without_an_opaque_builder_diagnostic() {
    for name in [
        "finite-fragments-good",
        "finite-fragments-boundary",
        "finite-fragments-suppressed",
        "finite-fragments-host-suppressed",
    ] {
        let report = check("postgres-sql-shape-policy", name);
        assert!(
            report["rules"].as_array().unwrap().is_empty(),
            "{name}: {report}"
        );
    }
    let bad = check("postgres-sql-shape-policy", "finite-fragments-bad");
    let findings = bad["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{bad}");
    assert_eq!(findings[0]["line"], 4);
    assert!(!findings[0]["message"]
        .as_str()
        .unwrap()
        .contains("recoverable"));
    let unexecuted = check("postgres-sql-shape-policy", "finite-fragments-unexecuted");
    let findings = unexecuted["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{unexecuted}");
    assert_eq!(findings[0]["line"], 2);
    assert!(findings[0]["message"]
        .as_str()
        .unwrap()
        .starts_with("src/query.ts:2:"));
    assert_eq!(findings[1]["line"], 10);
    assert!(findings[1]["message"]
        .as_str()
        .unwrap()
        .starts_with("src/query.ts:10: builder SQL"));
    for name in ["finite-fragments-cap", "finite-fragments-opaque"] {
        let report = check("postgres-sql-shape-policy", name);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 2, "{name}: {report}");
        assert!(
            findings
                .iter()
                .all(|finding| finding["message"].as_str().unwrap().contains("recoverable")),
            "{name}: {report}"
        );
    }
}

#[test]
fn distinct_same_line_variant_tokens_survive_final_sorting_without_changing_legacy_output() {
    for rule in [
        "postgres-explicit-columns",
        "postgres-sql-shape-policy",
        "postgres-bounded-statements",
    ] {
        let variants = check(rule, "distinct-columns");
        let findings = variants["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 2, "{rule}: {variants}");
        assert_eq!(findings[0], findings[1], "source identities stay internal");
        let legacy = check(rule, "legacy-columns");
        assert_eq!(
            legacy["rules"].as_array().unwrap().len(),
            1,
            "{rule}: {legacy}"
        );
    }
}

#[test]
fn each_clause_uses_its_branch_token_and_checks_the_other_alternative() {
    for rule in [
        "postgres-bounded-statements",
        "postgres-explicit-columns",
        "postgres-required-predicates",
        "postgres-generated-column-predicates",
        "postgres-sql-shape-policy",
        "postgres-no-generated-column-writes",
        "postgres-sql-statement-policy",
        "postgres-idempotent-insert",
        "postgres-lock-ordering",
        "postgres-conflict-ordering",
        "postgres-require-query-annotation",
        "postgres-no-offset",
    ] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(rule)
            .join("variants/clauses");
        let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
        let mut expected = source
            .lines()
            .enumerate()
            .flat_map(|(index, line)| {
                line.split_once("-- findings: ")
                    .map(|(_, targets)| {
                        targets
                            .split(',')
                            .map(move |target| (index + 1, target.trim().to_string()))
                    })
                    .into_iter()
                    .flatten()
            })
            .collect::<Vec<_>>();
        expected.sort();
        let report = check(rule, "clauses");
        let mut actual = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| {
                (
                    finding["line"].as_u64().unwrap() as usize,
                    finding["target"].as_str().unwrap().to_string(),
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        assert_eq!(actual, expected, "{rule}: {report}");
    }
}

#[test]
fn a_complete_but_invalid_alternative_keeps_each_rules_parse_failure_policy() {
    for (rule, target) in [
        ("postgres-bounded-statements", "unanalyzable"),
        ("postgres-explicit-columns", "unanalyzable"),
        ("postgres-required-predicates", "unanalyzable"),
        ("postgres-generated-column-predicates", "unanalyzable"),
        ("postgres-sql-shape-policy", "banned-function-call"),
        ("postgres-sql-statement-policy", "unanalyzable-sql"),
        ("postgres-idempotent-insert", "insert"),
        ("postgres-lock-ordering", "unparseable"),
        ("postgres-conflict-ordering", "unanalyzable-sql"),
    ] {
        let report = check(rule, "invalid-static");
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{rule}: {report}");
        assert_eq!(findings[0]["target"], target, "{rule}: {report}");
        assert_eq!(findings[0]["line"], 2, "{rule}: {report}");
    }
}
