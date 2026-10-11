use std::path::PathBuf;
use std::process::Command;
#[path = "cli_check_rules_postgres_variants/append_contributors.rs"]
mod append_contributors;
#[path = "cli_check_rules_postgres_variants/branch_correlation.rs"]
mod branch_correlation;

fn check(rule: &str, name: &str) -> serde_json::Value {
    check_with_suppressed(rule, name, false)
}

fn check_with_suppressed(rule: &str, name: &str, include_suppressed: bool) -> serde_json::Value {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("variants")
        .join(name);
    let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
    command
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"));
    if include_suppressed {
        command.arg("--include-suppressed");
    }
    let output = command.output().unwrap();
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

#[test]
fn shared_sql_tokens_preserve_active_executions_and_suppression_accounting_in_both_orders() {
    for (name, disabled_line) in [
        ("mixed-suppression-first", 4),
        ("mixed-suppression-last", 5),
    ] {
        let report = check_with_suppressed("postgres-no-offset", name, true);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{name}: {report}");
        assert_eq!(findings[0]["line"], 2, "{report}");
        assert_eq!(findings[0]["target"], "offset", "{report}");
        let suppressed = report["suppressed"].as_array().unwrap();
        assert_eq!(suppressed.len(), 1, "{name}: {report}");
        assert_eq!(suppressed[0]["line"], disabled_line, "{report}");
        assert_eq!(suppressed[0]["directive"]["kind"], "nextLine", "{report}");
    }
}

#[test]
fn lock_safe_directives_authorize_their_branch_or_the_actual_executor_invocation() {
    for (name, line) in [
        ("safe-directive-first", 3),
        ("safe-directive-last", 2),
        ("safe-directive-identifier", 3),
        ("safe-directive-disabled", 2),
    ] {
        let report = check("postgres-lock-ordering", name);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{name}: {report}");
        assert_eq!(findings[0]["line"], line, "{report}");
        assert_eq!(findings[0]["target"], "lock-ordering", "{report}");
    }
    for name in ["safe-directive-call", "safe-directive-prefix"] {
        let report = check("postgres-lock-ordering", name);
        assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    }
}

#[test]
fn unexecuted_shared_fragments_preserve_active_appends_and_account_for_disabled_hosts() {
    for (name, disabled_line) in [
        ("mixed-suppression-first", 6),
        ("mixed-suppression-last", 7),
    ] {
        let report = check_with_suppressed("postgres-sql-shape-policy", name, true);
        let findings = report["rules"].as_array().unwrap();
        assert_eq!(findings.len(), 1, "{name}: {report}");
        assert_eq!(findings[0]["line"], 2, "{report}");
        assert_eq!(findings[0]["target"], "banned-function-call", "{report}");
        let suppressed = report["suppressed"].as_array().unwrap();
        assert_eq!(suppressed.len(), 1, "{name}: {report}");
        assert_eq!(suppressed[0]["line"], disabled_line, "{report}");
    }
}

#[test]
fn ordinary_append_alias_mutations_preserve_real_offset_origins_or_fail_closed() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-offset/variants/alias-appends");
    let source = std::fs::read_to_string(root.join("src/query.ts")).unwrap();
    let mut expected = source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            line.split_once("// findings: ")
                .map(|(_, target)| (index + 1, target.trim().to_string()))
        })
        .collect::<Vec<_>>();
    expected.sort();
    let report = check("postgres-no-offset", "alias-appends");
    let mut actual = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["line"].as_u64().unwrap() as usize,
                finding["target"]
                    .as_str()
                    .unwrap()
                    .split('#')
                    .next()
                    .unwrap()
                    .to_string(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(actual, expected, "{report}");
}

#[test]
fn effectful_switch_labels_keep_queries_inside_and_after_the_switch_unanalyzable() {
    let report = check("postgres-lock-ordering", "switch-label-effects");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{report}");
    assert_eq!(findings[0]["line"], 6, "{report}");
    assert_eq!(findings[1]["line"], 8, "{report}");
    assert!(
        findings
            .iter()
            .all(|finding| finding["target"] == "unanalyzable"),
        "{report}"
    );
}

#[test]
fn equivalent_physical_branches_do_not_exhaust_the_concrete_version_cap() {
    let report = check("postgres-bounded-statements", "equivalent-branches");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
}
