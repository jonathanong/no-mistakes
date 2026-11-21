use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture_report;
use serde_json::Value;

type Case = (&'static str, std::path::PathBuf, std::path::PathBuf);

fn c(label: &'static str, rule: &str) -> Case {
    case(
        label,
        &format!("test-cases/rules/{rule}/invalid-utf8"),
        ".no-mistakes.yml",
    )
}

fn assert_definite(report: &Value, label: &str, rule: &str) {
    let entry = report
        .get(label)
        .and_then(|entry| entry.get("report"))
        .unwrap_or_else(|| panic!("{label} has no report: {report}"));
    let mut findings = Vec::new();
    for key in ["rules", "advisories"] {
        if let Some(items) = entry.get(key).and_then(Value::as_array) {
            findings.extend(items);
        }
    }
    assert!(
        findings.iter().any(|finding| {
            finding.get("rule").and_then(Value::as_str) == Some(rule)
                && finding.get("target").and_then(Value::as_str) != Some("unanalyzable")
        }),
        "{label} missing definite {rule} finding: {findings:?}"
    );
}

#[test]
fn invalid_utf8_text_rules() {
    let report = assert_saved_fixture_report(
        "invalid_utf8_text_rules",
        &[
            c("no_test_git_sha", "no-test-git-sha"),
            c("integration_test_no_mocks", "integration-test-no-mocks"),
            c("no_sparse_checkout", "no-sparse-checkout"),
            c("markdown_eval_tests", "markdown-eval-tests"),
            c("doc_consistency", "doc-consistency"),
            c("no_git_identity_mutation", "no-git-identity-mutation"),
            c("agents_md_max_size", "agents-md-max-size"),
            c("shellcheck_runner", "shellcheck-runner"),
        ],
        &[],
        &[],
    );
    assert_definite(&report, "no_test_git_sha", "no-test-git-sha");
    assert_definite(
        &report,
        "integration_test_no_mocks",
        "integration-test-no-mocks",
    );
    assert_definite(&report, "no_sparse_checkout", "no-sparse-checkout");
    assert_definite(&report, "markdown_eval_tests", "markdown-eval-tests");
    assert_definite(&report, "doc_consistency", "doc-consistency");
    assert_definite(
        &report,
        "no_git_identity_mutation",
        "no-git-identity-mutation",
    );
    assert_definite(&report, "agents_md_max_size", "agents-md-max-size");
    assert_definite(&report, "shellcheck_runner", "shellcheck-runner");
}

#[test]
fn invalid_utf8_github_actions() {
    let report = assert_saved_fixture_report(
        "invalid_utf8_github_actions",
        &[
            c("github_actions_job_timeouts", "github-actions-job-timeouts"),
            c(
                "github_actions_step_timeouts",
                "github-actions-step-timeouts",
            ),
            c(
                "github_actions_download_integrity",
                "github-actions-download-integrity",
            ),
            c(
                "github_actions_docker_host_ports",
                "github-actions-docker-host-ports",
            ),
            c(
                "github_actions_vitest_project_selection",
                "github-actions-vitest-project-selection",
            ),
            c(
                "github_actions_composite_step_schema",
                "github-actions-composite-step-schema",
            ),
            c(
                "github_actions_action_timeout_pair",
                "github-actions-action-timeout-pair",
            ),
            c(
                "github_actions_test_timeout_literals",
                "github-actions-test-timeout-literals",
            ),
            c(
                "github_actions_action_input_policy",
                "github-actions-action-input-policy",
            ),
        ],
        &[],
        &[],
    );
    assert_definite(
        &report,
        "github_actions_job_timeouts",
        "github-actions-job-timeouts",
    );
    assert_definite(
        &report,
        "github_actions_step_timeouts",
        "github-actions-step-timeouts",
    );
    assert_definite(
        &report,
        "github_actions_download_integrity",
        "github-actions-download-integrity",
    );
    assert_definite(
        &report,
        "github_actions_docker_host_ports",
        "github-actions-docker-host-ports",
    );
    assert_definite(
        &report,
        "github_actions_vitest_project_selection",
        "github-actions-vitest-project-selection",
    );
    assert_definite(
        &report,
        "github_actions_composite_step_schema",
        "github-actions-composite-step-schema",
    );
    assert_definite(
        &report,
        "github_actions_action_timeout_pair",
        "github-actions-action-timeout-pair",
    );
    assert_definite(
        &report,
        "github_actions_test_timeout_literals",
        "github-actions-test-timeout-literals",
    );
    assert_definite(
        &report,
        "github_actions_action_input_policy",
        "github-actions-action-input-policy",
    );
}

#[test]
fn invalid_utf8_config_rules() {
    let report = assert_saved_fixture_report(
        "invalid_utf8_config_rules",
        &[
            c("require_storybook_stories", "require-storybook-stories"),
            c(
                "test_no_unmocked_dynamic_imports",
                "test-no-unmocked-dynamic-imports",
            ),
            c("tsconfig_file_coverage", "tsconfig-file-coverage"),
            c("config_path_references", "config-path-references"),
            c("structured_config_policy", "structured-config-policy"),
        ],
        &[],
        &[],
    );
    assert_definite(
        &report,
        "require_storybook_stories",
        "require-storybook-stories",
    );
    assert_definite(
        &report,
        "test_no_unmocked_dynamic_imports",
        "test-no-unmocked-dynamic-imports",
    );
    assert_definite(&report, "tsconfig_file_coverage", "tsconfig-file-coverage");
    assert_definite(&report, "config_path_references", "config-path-references");
    assert_definite(
        &report,
        "structured_config_policy",
        "structured-config-policy",
    );
}

#[test]
fn invalid_utf8_postgres() {
    let report = assert_saved_fixture_report(
        "invalid_utf8_postgres",
        &[
            c("postgres_explicit_columns", "postgres-explicit-columns"),
            c(
                "postgres_no_generated_column_writes",
                "postgres-no-generated-column-writes",
            ),
            c("postgres_idempotent_insert", "postgres-idempotent-insert"),
            c(
                "postgres_no_transformed_uuid_predicates",
                "postgres-no-transformed-uuid-predicates",
            ),
            c("postgres_sql_shape_policy", "postgres-sql-shape-policy"),
            c(
                "postgres_no_test_state_mutation",
                "postgres-no-test-state-mutation",
            ),
            c("postgres_bounded_statements", "postgres-bounded-statements"),
            c(
                "postgres_require_query_annotation",
                "postgres-require-query-annotation",
            ),
            c(
                "postgres_lock_written_rows_only",
                "postgres-lock-written-rows-only",
            ),
            c("postgres_conflict_ordering", "postgres-conflict-ordering"),
            c("postgres_lock_ordering", "postgres-lock-ordering"),
            c(
                "postgres_sql_statement_policy",
                "postgres-sql-statement-policy",
            ),
            c(
                "postgres_generated_column_predicates",
                "postgres-generated-column-predicates",
            ),
            c(
                "postgres_required_predicates",
                "postgres-required-predicates",
            ),
            c("postgres_fk_index", "postgres-fk-index"),
            c("postgres_redundant_index", "postgres-redundant-index"),
            c("postgres_json_relation_ids", "postgres-json-relation-ids"),
            c(
                "postgres_grouping_not_aggregation",
                "postgres-grouping-not-aggregation",
            ),
            c("postgres_no_offset", "postgres-no-offset"),
            case(
                "postgres_no_offset_sql",
                "test-cases/rules/postgres-no-offset/invalid-utf8-sql",
                ".no-mistakes.yml",
            ),
            case(
                "postgres_no_offset_low",
                "test-cases/rules/postgres-no-offset/invalid-utf8-low",
                ".no-mistakes.yml",
            ),
            case(
                "postgres_no_transformed_uuid_predicates_low",
                "test-cases/rules/postgres-no-transformed-uuid-predicates/invalid-utf8-low",
                ".no-mistakes.yml",
            ),
        ],
        &[],
        &[],
    );
    assert_definite(
        &report,
        "postgres_explicit_columns",
        "postgres-explicit-columns",
    );
    assert_definite(
        &report,
        "postgres_no_generated_column_writes",
        "postgres-no-generated-column-writes",
    );
    assert_definite(
        &report,
        "postgres_idempotent_insert",
        "postgres-idempotent-insert",
    );
    assert_definite(
        &report,
        "postgres_no_transformed_uuid_predicates",
        "postgres-no-transformed-uuid-predicates",
    );
    assert_definite(
        &report,
        "postgres_sql_shape_policy",
        "postgres-sql-shape-policy",
    );
    assert_definite(
        &report,
        "postgres_no_test_state_mutation",
        "postgres-no-test-state-mutation",
    );
    assert_definite(
        &report,
        "postgres_bounded_statements",
        "postgres-bounded-statements",
    );
    assert_definite(
        &report,
        "postgres_require_query_annotation",
        "postgres-require-query-annotation",
    );
    assert_definite(
        &report,
        "postgres_lock_written_rows_only",
        "postgres-lock-written-rows-only",
    );
    assert_definite(
        &report,
        "postgres_conflict_ordering",
        "postgres-conflict-ordering",
    );
    assert_definite(&report, "postgres_lock_ordering", "postgres-lock-ordering");
    assert_definite(
        &report,
        "postgres_sql_statement_policy",
        "postgres-sql-statement-policy",
    );
    assert_definite(
        &report,
        "postgres_generated_column_predicates",
        "postgres-generated-column-predicates",
    );
    assert_definite(
        &report,
        "postgres_required_predicates",
        "postgres-required-predicates",
    );
    assert_definite(&report, "postgres_fk_index", "postgres-fk-index");
    assert_definite(
        &report,
        "postgres_redundant_index",
        "postgres-redundant-index",
    );
    assert_definite(
        &report,
        "postgres_json_relation_ids",
        "postgres-json-relation-ids",
    );
    assert_definite(
        &report,
        "postgres_grouping_not_aggregation",
        "postgres-grouping-not-aggregation",
    );
    assert_definite(&report, "postgres_no_offset", "postgres-no-offset");
    assert_definite(&report, "postgres_no_offset_sql", "postgres-no-offset");
    assert_definite(&report, "postgres_no_offset_low", "postgres-no-offset");
    assert_definite(
        &report,
        "postgres_no_transformed_uuid_predicates_low",
        "postgres-no-transformed-uuid-predicates",
    );
}
