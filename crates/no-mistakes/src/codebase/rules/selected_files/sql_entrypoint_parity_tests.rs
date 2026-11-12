use super::*;
use crate::codebase::rules;
use serde_json::Value;

type Check = fn(&Path, &NoMistakesConfig, &[PathBuf]) -> Result<Vec<RuleFinding>>;

#[test]
fn standalone_sql_rules_match_prepared_check_reports_on_saved_counterexamples() {
    let checks: &[(&str, Check)] = &[
        (
            "postgres-lock-ordering",
            rules::postgres_lock_ordering::check_with_files,
        ),
        (
            "postgres-status-with-lifecycle-timestamps",
            rules::postgres_status_with_lifecycle_timestamps::check_with_files,
        ),
        (
            "postgres-array-columns",
            rules::postgres_array_columns::check_with_files,
        ),
        (
            "postgres-no-offset",
            rules::postgres_no_offset::check_with_files,
        ),
        (
            "postgres-finite-text-columns",
            rules::postgres_finite_text_columns::check_with_files,
        ),
        (
            "postgres-table-shape",
            rules::postgres_table_shape::check_with_files,
        ),
        (
            "postgres-redundant-index",
            rules::postgres_redundant_index::check_with_files,
        ),
        (
            "postgres-duplicate-function-body",
            rules::postgres_duplicate_function_body::check_with_files,
        ),
        (
            "postgres-fk-index",
            rules::postgres_fk_index::check_with_files,
        ),
        (
            "postgres-no-add-column",
            rules::postgres_no_add_column::check_with_files,
        ),
        (
            "postgres-require-fk-on-delete",
            rules::postgres_require_fk_on_delete::check_with_files,
        ),
        (
            "postgres-required-predicates",
            rules::postgres_required_predicates::check_with_files,
        ),
        (
            "postgres-column-requires-trigger",
            rules::postgres_column_requires_trigger::check_with_files,
        ),
        (
            "postgres-column-check-constraints",
            rules::postgres_column_check_constraints::check_with_files,
        ),
        (
            "postgres-sql-shape-policy",
            rules::postgres_sql_shape_policy::check_with_files,
        ),
        (
            "postgres-required-comments",
            rules::postgres_required_comments::check_with_files,
        ),
        (
            "postgres-column-naming",
            rules::postgres_column_naming::check_with_files,
        ),
        (
            "postgres-explicit-columns",
            rules::postgres_explicit_columns::check_with_files,
        ),
        (
            "postgres-require-query-annotation",
            rules::postgres_require_query_annotation::check_with_files,
        ),
    ];
    for &(rule, check) in checks {
        let root = crate::codebase::ts_resolver::normalize_path(
            &crate::test_support::rule_fixture_root(rule).join("fail"),
        );
        let config_path = root.join(".no-mistakes.yml");
        let config = crate::config::v2::load_v2_config(&root, Some(&config_path)).unwrap();
        let files = discover_files(&root, &config.filesystem.skip_directories);
        assert!(!files.is_empty(), "{rule}");
        let mut actual = check(&root, &config, &files).unwrap();
        rules::suppression::suppress_rule_findings_with_sources(
            &root,
            &mut actual,
            &rules::source_store_for_files(&files),
        );
        let actual: Vec<Value> = actual
            .into_iter()
            .filter(|finding| finding.rule == rule)
            .map(|finding| serde_json::to_value(finding).unwrap())
            .collect();
        assert!(
            !actual.is_empty(),
            "{rule} counterexample must still report"
        );
        let prepared =
            crate::check_runner::run_all_with_evidence(root, Some(config_path), None, false, false)
                .unwrap();
        let report = crate::check_runner::json_value(&prepared);
        let expected: Vec<Value> = report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .chain(report["advisories"].as_array().unwrap())
            .filter(|finding| finding["rule"] == rule)
            .cloned()
            .collect();
        assert_eq!(actual, expected, "{rule}");
    }
}
