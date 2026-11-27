use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;
use std::process::Command;

#[test]
fn postgres_no_duplicate_keyed_lookups_check_json_snapshot() {
    let rule = "postgres-no-duplicate-keyed-lookups";
    let rows = [
        ("scalar_subqueries", "scalar-subqueries", "enabled-sql.yml"),
        ("joins_and_mixed", "joins", "enabled-sql.yml"),
        ("dml_and_with", "dml-cte", "enabled-sql.yml"),
        ("normalization", "normalization", "enabled-sql.yml"),
        ("query_scopes", "query-scopes", "enabled-sql.yml"),
        ("query_clauses", "query-clauses", "enabled-sql.yml"),
        ("target_list", "target-list", "enabled-sql.yml"),
        (
            "recovered_siblings",
            "recovered-siblings",
            "enabled-sql.yml",
        ),
        (
            "recovered_siblings_advise",
            "recovered-siblings",
            "unanalyzable-native-advise.yml",
        ),
        ("review_planning", "review-planning", "enabled-sql.yml"),
        ("review_edges", "review-edges", "enabled-sql.yml"),
        (
            "review_target_reuse",
            "review-target-reuse",
            "enabled-sql.yml",
        ),
        (
            "review_shared_query",
            "review-shared-query",
            "enabled-sql.yml",
        ),
        (
            "review_window_using",
            "review-window-using",
            "enabled-sql.yml",
        ),
        ("review_soundness", "review-soundness", "enabled-sql.yml"),
        ("distinct_and_clean", "pass", "enabled-sql.yml"),
        ("disabled_by_config", "scalar-subqueries", "disabled.yml"),
        (
            "default_sql_unscanned",
            "scalar-subqueries",
            "default-sql-unscanned.yml",
        ),
        ("custom_message", "scalar-subqueries", "custom-message.yml"),
        ("sql_suppressions", "suppression", "enabled-sql.yml"),
        ("embedded_report", "embedded", "enabled-ts.yml"),
        ("embedded_advise", "embedded", "unanalyzable-advise.yml"),
        ("embedded_ignore", "embedded", "unanalyzable-ignore.yml"),
    ];
    let cases: Vec<(&str, PathBuf, PathBuf)> = rows
        .into_iter()
        .map(|(label, fixture, config)| {
            (
                label,
                rules_path(rule, "fixture", fixture),
                rules_path(rule, "configs", config),
            )
        })
        .collect();
    assert_saved_fixture("postgres_no_duplicate_keyed_lookups", &cases);
}

#[test]
fn postgres_no_duplicate_keyed_lookups_rejects_invalid_options_with_exit_2() {
    let rule = "postgres-no-duplicate-keyed-lookups";
    let root = rules_path(rule, "fixture", "pass");
    for (config, expected) in [
        ("invalid-unknown-option.yml", "unknownFlag"),
        (
            "invalid-unmatched-executors.yml",
            "options.reportUnmatchedExecutorNames",
        ),
        ("invalid-missing-executor.yml", "importSpecifier"),
        ("invalid-both-executors.yml", "extraExecutorNames"),
        ("invalid-glob.yml", "sqlInclude"),
        ("invalid-unanalyzable.yml", "unanalyzableSql"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args([
                "check",
                "--root",
                root.to_str().unwrap(),
                "--config",
                rules_path(rule, "configs", config).to_str().unwrap(),
                "--format",
                "json",
            ])
            .output()
            .expect("run no-mistakes check");
        assert_eq!(
            output.status.code(),
            Some(2),
            "{config}: expected exit 2, got {:?}\nstderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            combined.contains(expected),
            "{config}: expected `{expected}` in output, got:\n{combined}"
        );
    }
}
