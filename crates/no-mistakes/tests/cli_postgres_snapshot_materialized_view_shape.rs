use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::{Command, Output};

const RULE: &str = "postgres-materialized-view-shape";

fn fixture(name: &str) -> PathBuf {
    rules_path(RULE, "fixture", name)
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

fn run(root: &str, options: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--jobs", "2", "check", "--root"])
        .arg(fixture(root))
        .arg("--config")
        .arg(config(options))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

fn report(root: &str, options: &str) -> Value {
    let output = run(root, options);
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn targets(report: &Value) -> BTreeSet<&str> {
    report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|finding| finding["target"].as_str())
        .collect()
}

#[test]
fn postgres_materialized_view_shape_check_json_snapshot() {
    let cases = [
        ("default_errors", "fail", "default.yml"),
        ("explicit_error_severity", "fail", "error.yml"),
        ("advisory_severity", "fail", "advisory.yml"),
        ("index_and_identity_edges", "edges", "edges.yml"),
        ("compliant", "pass", "pass.yml"),
        ("ordering_catalog", "ordering", "default.yml"),
        ("table_view_identity", "collision", "collision.yml"),
        ("legacy_index_data", "legacy-index-data", "advisory.yml"),
        ("empty_index_data", "empty-index-data", "advisory.yml"),
        ("view_read_path", "view-read-path", "advisory.yml"),
        ("mixed_severities", "mixed", "mixed.yml"),
        ("excluded_error_application", "mixed", "excluded-error.yml"),
        ("zero_additional", "fail", "zero-additional.yml"),
        ("two_additional", "fail", "two-additional.yml"),
        (
            "unconfigured_window_function",
            "fail",
            "no-window-functions.yml",
        ),
        ("no_base_allow", "fail", "no-base-allow.yml"),
        ("allow_view", "fail", "allow.yml"),
        ("suppressed", "suppressed", "suppressed.yml"),
        ("off", "fail", "off.yml"),
    ]
    .map(|(label, root, options)| (label, fixture(root), config(options)));
    assert_saved_fixture("postgres_materialized_view_shape", &cases);
}

#[test]
fn materialized_view_controls_cover_indexes_and_every_base_read() {
    let checked = report("fail", "advisory.yml");
    assert_eq!(checked["rules"], serde_json::json!([]), "{checked}");
    assert_eq!(checked["warnings"], serde_json::json!([]), "{checked}");
    let expected: BTreeSet<_> = [
        "bad_nested_view",
        "bad_no_indexes",
        "bad_no_unique",
        "bad_only_unique",
        "bad_partial_unique",
        "bad_expression_unique",
        "bad_invalid_unique",
        "bad_not_ready",
        "bad_not_live",
        "bad_unbounded",
        "bad_equality",
        "bad_unindexed",
        "bad_or",
        "bad_join",
        "bad_left_join",
        "bad_cte",
        "bad_self_join",
        "bad_scalar_subquery",
        "bad_limit",
        "bad_unknown",
        "bad_parse",
    ]
    .into_iter()
    .map(|name| format!("materialized-view:{name}"))
    .collect();
    let actual: BTreeSet<_> = targets(&checked).into_iter().map(str::to_owned).collect();
    assert_eq!(actual, expected, "{checked}");
    assert!(checked["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .all(|finding| { finding["rule"] == RULE && finding["file"] == "schema.json" }));
    let compliant = report("pass", "pass.yml");
    assert!(targets(&compliant).is_empty(), "{compliant}");
}

#[test]
fn materialized_view_configuration_changes_only_selected_requirements() {
    let zero = report("fail", "zero-additional.yml");
    let zero_targets = targets(&zero);
    assert!(!zero_targets.contains("materialized-view:bad_only_unique"));
    assert!(zero_targets.contains("materialized-view:bad_no_unique"));
    assert!(zero_targets.contains("materialized-view:bad_unbounded"));

    let two = report("fail", "two-additional.yml");
    let two_targets = targets(&two);
    assert!(two_targets.contains("materialized-view:good_range"));
    assert!(!two_targets.contains("materialized-view:good_extra_two"));
    assert!(two_targets.contains("materialized-view:bad_unbounded"));

    let no_window = report("fail", "no-window-functions.yml");
    assert!(targets(&no_window).contains("materialized-view:good_window"));
    assert!(!targets(&no_window).contains("materialized-view:good_time"));
    assert!(targets(&no_window).contains("materialized-view:bad_unbounded"));

    let no_allow = report("fail", "no-base-allow.yml");
    assert!(targets(&no_allow).contains("materialized-view:good_allowed"));
    assert!(targets(&no_allow).contains("materialized-view:bad_unbounded"));

    let allowed = report("fail", "allow.yml");
    assert!(!targets(&allowed).contains("materialized-view:bad_unbounded"));
    assert!(targets(&allowed).contains("materialized-view:bad_join"));
}

#[test]
fn materialized_view_rule_off_and_suppression_preserve_empty_reports() {
    let off = run("fail", "off.yml");
    let empty = run("fail", "empty.yml");
    assert!(off.status.success(), "{off:?}");
    assert!(empty.status.success(), "{empty:?}");
    assert_eq!(off.stdout, empty.stdout);
    let suppressed = report("suppressed", "suppressed.yml");
    assert!(targets(&suppressed).is_empty(), "{suppressed}");
}

#[test]
fn materialized_view_invalid_options_fail_with_actionable_errors() {
    for (options, field) in [
        ("invalid-reason.yml", "reason"),
        ("invalid-min.yml", "minAdditionalIndexes"),
        ("missing-catalog.yml", "schemaCatalogPath"),
        ("invalid-window-name.yml", "windowFunctions"),
        ("invalid-window-target.yml", "windowFunctions"),
        ("duplicate-base-allow.yml", "allowBaseTables"),
        ("empty-base-table.yml", "allowBaseTables"),
    ] {
        let output = run("fail", options);
        assert!(!output.status.success(), "{options}: {output:?}");
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(text.contains(RULE), "{options}: {text}");
        assert!(text.contains(field), "{options}: {text}");
    }
}

#[test]
fn materialized_view_index_and_identity_edges_keep_valid_controls_clear() {
    let checked = report("edges", "edges.yml");
    let expected: BTreeSet<_> = [
        "bad_nonkey_function_equality",
        "bad_distinct_subquery",
        "bad_named_window_subquery",
        "bad_renamed_table_columns",
        "bad_from_function_subquery",
        "bad_lateral_function_subquery",
        "bad_unnest_subquery",
        "bad_in_list",
        "bad_function_range_nonkey",
        "bad_function_between_nonkey",
        "bad_function_wrapped_range_nonkey",
        "bad_nested_outer_on",
        "bad_bare_ambiguous_range",
        "bad_bare_unknown_peer",
        "bad_or_different_aliases",
        "bad_nested_full_on",
        "bad_table_arm",
        "bad_query_kind",
        "bad_multiple_statements",
        "bad_union_table_arm",
        "bad_cte_table_arm",
        "bad_empty_unique",
        "bad_deferred_unique",
        "bad_invalid_read",
        "bad_not_ready_read",
        "bad_partial_base_index",
        "bad_nonleading_base_index",
        "bad_recursive",
        "bad_quoted_wrong_case",
        "bad_quoted_function_identity",
        "bad_quoted_function_case",
        "bad_foreign_schema",
        "bad_or_unindexed_column",
    ]
    .into_iter()
    .map(|name| format!("materialized-view:{name}"))
    .collect();
    let actual: BTreeSet<_> = targets(&checked).into_iter().map(str::to_owned).collect();
    assert_eq!(actual, expected, "{checked}");
}

#[test]
fn materialized_view_default_severity_fails_unless_shared_severity_is_advisory() {
    let output = run("fail", "default.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let explicit = run("fail", "error.yml");
    assert_eq!(explicit.status.code(), Some(1), "{explicit:?}");
    assert_eq!(output.stdout, explicit.stdout);
    let checked: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        checked["advisories"].as_array().unwrap().is_empty(),
        "{checked}"
    );
    assert!(
        checked["rules"].as_array().unwrap().iter().any(|finding| {
            finding["rule"] == RULE && finding["target"] == "materialized-view:bad_unbounded"
        }),
        "{checked}"
    );
    let advisory = report("fail", "advisory.yml");
    assert!(
        advisory["rules"].as_array().unwrap().is_empty(),
        "{advisory}"
    );
    assert!(targets(&advisory).contains("materialized-view:bad_unbounded"));
}

#[test]
fn materialized_view_shape_requires_complete_catalog_evidence() {
    let output = run("ordering", "default.yml");
    assert!(!output.status.success(), "{output:?}");
    let checked: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(checked["incomplete"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| {
            entry["message"]
                .as_str()
                .unwrap()
                .contains("complete schema catalog")
        }));
}

#[test]
fn materialized_view_mixed_applications_follow_shared_rule_severity() {
    let output = run("mixed", "mixed.yml");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let checked: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(targets(&checked).is_empty(), "{checked}");
    let errors = checked["rules"].as_array().unwrap();
    let error_targets: BTreeSet<_> = errors
        .iter()
        .filter_map(|finding| finding["target"].as_str())
        .collect();
    assert_eq!(
        error_targets,
        BTreeSet::from([
            "materialized-view:bad_advised",
            "materialized-view:bad_reported",
        ])
    );
    assert!(errors
        .iter()
        .filter(|finding| finding["target"] == "materialized-view:bad_advised")
        .all(|finding| finding["message"]
            .as_str()
            .unwrap()
            .starts_with("Review the advised view")));
    let excluded = run("mixed", "excluded-error.yml");
    assert_eq!(excluded.status.code(), Some(1), "{excluded:?}");
    let checked: Value = serde_json::from_slice(&excluded.stdout).unwrap();
    assert!(targets(&checked).is_empty(), "{checked}");
    let error_targets: BTreeSet<_> = checked["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|finding| finding["target"].as_str())
        .collect();
    assert_eq!(
        error_targets,
        BTreeSet::from(["materialized-view:bad_advised"])
    );
}

#[test]
fn materialized_view_bounds_do_not_borrow_keys_or_allowances_across_view_collisions() {
    let checked = report("collision", "collision.yml");
    assert_eq!(
        targets(&checked),
        BTreeSet::from([
            "materialized-view:bad_collision",
            "materialized-view:bad_qualified_view",
        ])
    );
}

#[test]
fn materialized_view_missing_index_data_requires_one_catalog_regeneration_finding() {
    let checked = report("legacy-index-data", "advisory.yml");
    let findings = checked["advisories"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{checked}");
    assert!(findings[0]["target"].is_null(), "{checked}");
    let message = findings[0]["message"].as_str().unwrap();
    assert!(message.contains("catalog lacks materialized-view index data"));
    assert!(message.contains("no-mistakes postgres catalog --coverage complete"));
    assert!(!message.contains("missing a usable"));
    let explicit = report("empty-index-data", "advisory.yml");
    let findings = explicit["advisories"].as_array().unwrap();
    assert_eq!(findings.len(), 4, "{explicit}");
    for name in ["summary_one", "summary_two"] {
        assert_eq!(
            findings
                .iter()
                .filter(|finding| { finding["target"] == format!("materialized-view:{name}") })
                .count(),
            2,
            "{explicit}"
        );
    }
}

#[test]
fn materialized_view_unbounded_reads_retain_each_referenced_view_path() {
    let checked = report("view-read-path", "advisory.yml");
    let findings = checked["advisories"].as_array().unwrap();
    assert_eq!(findings.len(), 5, "{checked}");
    for (target, expected) in [
        ("nested_read", "records via view some_view;"),
        (
            "nested_chain",
            "records via view wrapper_view -> view some_view;",
        ),
        ("direct_read", "records;"),
        ("both_reads", "records;"),
        ("both_reads", "records via view some_view;"),
    ] {
        assert!(
            findings.iter().any(|finding| {
                finding["target"] == format!("materialized-view:{target}")
                    && finding["message"].as_str().unwrap().contains(expected)
            }),
            "{target}: {checked}"
        );
    }
    assert!(!targets(&checked).contains("materialized-view:bounded_direct"));
}
