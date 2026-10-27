use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

fn repo_case(name: &str) -> PathBuf {
    repo_rel(&format!("fixtures/postgres/conflict-ordering/cases/{name}"))
}

fn push(
    cases: &mut Vec<(&'static str, PathBuf, PathBuf)>,
    config: &PathBuf,
    items: &[(&'static str, PathBuf)],
) {
    for &(label, ref root) in items {
        cases.push((label, PathBuf::clone(root), PathBuf::clone(config)));
    }
}

#[test]
fn postgres_conflict_ordering_check_json_snapshot() {
    let rule = "postgres-conflict-ordering";
    let catalog = rules_path(rule, "configs", "catalog.yml");
    let ignore = rules_path(rule, "configs", "ignore.yml");
    let mut cases = Vec::new();
    push(
        &mut cases,
        &catalog,
        &[
            ("fail_missing_order", repo_case("fail-missing-order")),
            (
                "fail_noncanonical_order",
                repo_case("fail-noncanonical-order"),
            ),
            (
                "fail_unresolved_source_order",
                repo_case("fail-unresolved-source-order"),
            ),
            ("pass_expression", repo_case("pass-expression")),
            ("fail_ambiguous", repo_case("fail-ambiguous")),
            ("fail_reversed_target", repo_case("fail-reversed-target")),
            ("pass_partial", repo_case("pass-partial")),
            (
                "fail_partial_predicate",
                repo_case("fail-partial-predicate"),
            ),
            ("pass_constraint", repo_case("pass-constraint")),
            ("fail_targetless", repo_case("fail-targetless")),
            ("fail_opaque_executor", repo_case("fail-opaque-executor")),
            (
                "fail_opaque_member_query",
                repo_case("fail-opaque-member-query"),
            ),
            (
                "fail_append_missing_order",
                repo_case("fail-append-missing-order"),
            ),
            // The taken path is ordered; the other path has no ON CONFLICT.
            (
                "append_conditional_insert",
                repo_case("fail-append-conditional-insert"),
            ),
            (
                "fail_append_opaque_insert",
                repo_case("fail-append-opaque-insert"),
            ),
            (
                "fail_incomplete_with_append",
                repo_case("fail-incomplete-with-append"),
            ),
            (
                "fail_helper_insert_missing_order",
                repo_case("fail-helper-insert-missing-order"),
            ),
            ("pass_append_mutations", repo_case("pass-append-mutations")),
            ("pass_bare_order_by", repo_case("pass-bare-order-by")),
            // Ambiguous name, unknown table, and reversed order: three noncanonical-order findings.
            ("fail_bare_order_by", repo_case("fail-bare-order-by")),
            ("pass_default_arbiter", repo_case("pass-default-arbiter")),
            // Both missing-canonical-order and unresolved-source-order must remain.
            ("fail_default_arbiter", repo_case("fail-default-arbiter")),
            (
                "pass_template_constant_key",
                repo_case("pass-template-constant-key"),
            ),
            // Control and both user-spelled markers stay missing-canonical-order.
            (
                "fail_template_constant_key",
                repo_case("fail-template-constant-key"),
            ),
            ("pass_order_by_forms", repo_case("pass-order-by-forms")),
            (
                "canonical_insert",
                rules_path(rule, "fixture", "canonical-insert"),
            ),
            (
                "canonical_alias",
                rules_path(rule, "fixture", "canonical-alias"),
            ),
            (
                "canonical_transaction",
                rules_path(rule, "fixture", "canonical-transaction"),
            ),
            (
                "canonical_dynamic",
                rules_path(rule, "fixture", "canonical-dynamic"),
            ),
            (
                "canonical_dynamic_fragment",
                rules_path(rule, "fixture", "canonical-dynamic-fragment"),
            ),
            (
                "canonical_dynamic_select",
                rules_path(rule, "fixture", "canonical-dynamic-select"),
            ),
            (
                "canonical_dynamic_comment",
                rules_path(rule, "fixture", "canonical-dynamic-comment"),
            ),
            (
                "canonical_dynamic_safe",
                rules_path(rule, "fixture", "canonical-dynamic-safe"),
            ),
            (
                "malformed_conflict",
                rules_path(rule, "fixture", "malformed-conflict"),
            ),
            ("not_sql", rules_path(rule, "fixture", "not-sql")),
            (
                "single_row_pinned",
                rules_path(rule, "fixture", "single-row-pinned"),
            ),
            (
                "single_row_lookalike",
                rules_path(rule, "fixture", "single-row-lookalike"),
            ),
            // Message must end with "input.user_id ASC NULLS LAST".
            (
                "constant_missing_order",
                rules_path(rule, "fixture", "constant-missing-order"),
            ),
            (
                "constant_desc",
                rules_path(rule, "fixture", "constant-desc"),
            ),
            (
                "constant_all_keys",
                rules_path(rule, "fixture", "constant-all-keys"),
            ),
            (
                "constant_uncast",
                rules_path(rule, "fixture", "constant-uncast"),
            ),
            // Target must not contain "unanalyzable".
            ("one_bad", rules_path(rule, "variants", "one-bad")),
        ],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "sql-include.yml"),
        &[
            ("pass_sql_include", repo_case("pass-sql-include")),
            // The first statement's directive must not hide the finding on line 5.
            (
                "sql_statement_directive",
                repo_case("fail-sql-statement-directive"),
            ),
        ],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "sql-exclude.yml"),
        &[("sql_excluded", repo_case("pass-sql-include"))],
    );
    push(
        &mut cases,
        &rules_path(rule, "configs", "include-missing.yml"),
        &[("include_missing", repo_case("fail-missing-order"))],
    );
    push(
        &mut cases,
        &ignore,
        &[
            ("opaque_executor_ignored", repo_case("fail-opaque-executor")),
            (
                "canonical_dynamic_ignored",
                rules_path(rule, "fixture", "canonical-dynamic"),
            ),
            (
                "malformed_ignored",
                rules_path(rule, "fixture", "malformed-conflict"),
            ),
        ],
    );
    assert_saved_fixture("postgres_conflict_ordering", &cases);
}
