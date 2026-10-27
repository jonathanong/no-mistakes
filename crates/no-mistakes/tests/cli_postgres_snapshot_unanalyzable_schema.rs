use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULES: &[(&str, &str)] = &[
    ("postgres-constraint-validate", "constraint_validate"),
    ("postgres-fk-index", "fk_index"),
    (
        "postgres-generated-column-predicates",
        "generated_column_predicates",
    ),
    ("postgres-idempotent-insert", "idempotent_insert"),
    ("postgres-identifier-length", "identifier_length"),
    ("postgres-no-add-column", "no_add_column"),
    (
        "postgres-no-generated-column-writes",
        "no_generated_column_writes",
    ),
    ("postgres-redundant-index", "redundant_index"),
    ("postgres-require-fk-on-delete", "require_fk_on_delete"),
    (
        "postgres-require-named-constraints",
        "require_named_constraints",
    ),
    ("postgres-sql-statement-policy", "sql_statement_policy"),
];

fn ddl_fixture(subdir: &str) -> PathBuf {
    let mut path = rules_path("postgres-ddl-unanalyzable", "fixture", "report.yml");
    path.pop();
    if !subdir.is_empty() {
        path.push(subdir);
    }
    path
}

fn push_all(cases: &mut Vec<(String, PathBuf, PathBuf)>, group: &str, subdir: &str, config: &str) {
    for (rule, short) in RULES {
        cases.push((
            format!("{group}_{short}"),
            ddl_fixture(subdir),
            rules_path(rule, "configs", config),
        ));
    }
}

fn push_fk(cases: &mut Vec<(String, PathBuf, PathBuf)>, label: &str, subdir: &str, config: &str) {
    cases.push((
        label.to_string(),
        ddl_fixture(subdir),
        rules_path("postgres-fk-index", "configs", config),
    ));
}

#[test]
fn postgres_unanalyzable_schema_check_json_snapshot() {
    // One case per schema rule id. Each config enables only that rule on the
    // same fixture and policy the unit loop used. sqlInclude lists only the
    // files that loop passed (not a broad migrations glob).
    let mut owned = Vec::new();

    // migrations/001.sql lines 3 and 6 stay findings; advisories stay empty.
    // Both the max report.yml policy and effort: low keep that placement.
    push_all(
        &mut owned,
        "failed_statement_report",
        "",
        "ddl-report-001.yml",
    );
    push_all(&mut owned, "failed_statement_low", "", "ddl-low-001.yml");
    // Same two markers move to advisories, or disappear when ignored.
    push_all(&mut owned, "policy_advise", "", "ddl-advise-001.yml");
    push_all(&mut owned, "policy_ignore", "", "ddl-ignore-001.yml");

    // Body default: findings at 2, 33, 35 and advisories at 5, 6, 8, 13, 18, 21, 24, 28.
    push_all(
        &mut owned,
        "body_default",
        "body-policy",
        "ddl-body-default.yml",
    );
    // Explicit policies: report/override/global-alias keep 11 findings;
    // advise/rule-advise keep 11 advisories; ignore/rule-ignore are empty.
    push_all(
        &mut owned,
        "body_report",
        "body-policy",
        "ddl-body-report.yml",
    );
    push_all(
        &mut owned,
        "body_advise",
        "body-policy",
        "ddl-body-advise.yml",
    );
    push_all(
        &mut owned,
        "body_ignore",
        "body-policy",
        "ddl-body-ignore.yml",
    );
    push_all(
        &mut owned,
        "body_override",
        "body-policy",
        "ddl-body-override.yml",
    );
    push_all(
        &mut owned,
        "body_global_alias_report",
        "body-policy",
        "ddl-body-global-alias-report.yml",
    );
    push_all(
        &mut owned,
        "body_rule_advise",
        "body-policy",
        "ddl-body-rule-advise.yml",
    );
    push_all(
        &mut owned,
        "body_rule_ignore",
        "body-policy",
        "ddl-body-rule-ignore.yml",
    );
    // Wrapper findings at 2 and 7; the opaque child at 4 stays an advisory.
    push_all(
        &mut owned,
        "body_wrapper",
        "body-wrapper",
        "ddl-wrapper.yml",
    );

    // Unsupported default: findings 4-7, advisories 1-3. Explicit policies
    // move all 7 markers together (report/override/alias, advise, or ignore).
    push_all(
        &mut owned,
        "unsupported_default",
        "unsupported-policy",
        "ddl-unsupported-default.yml",
    );
    push_all(
        &mut owned,
        "unsupported_report",
        "unsupported-policy",
        "ddl-unsupported-report.yml",
    );
    push_all(
        &mut owned,
        "unsupported_advise",
        "unsupported-policy",
        "ddl-unsupported-advise.yml",
    );
    push_all(
        &mut owned,
        "unsupported_ignore",
        "unsupported-policy",
        "ddl-unsupported-ignore.yml",
    );
    push_all(
        &mut owned,
        "unsupported_override",
        "unsupported-policy",
        "ddl-unsupported-override.yml",
    );
    push_all(
        &mut owned,
        "unsupported_global_alias_report",
        "unsupported-policy",
        "ddl-unsupported-global-alias-report.yml",
    );
    push_all(
        &mut owned,
        "unsupported_rule_advise",
        "unsupported-policy",
        "ddl-unsupported-rule-advise.yml",
    );
    push_all(
        &mut owned,
        "unsupported_rule_ignore",
        "unsupported-policy",
        "ddl-unsupported-rule-ignore.yml",
    );

    // Recursive heads: lines 9, 23, and 25 are advisories; other heads are findings.
    // fk-index line 18 is absent here because the CLI honors the disable comment.
    push_all(
        &mut owned,
        "recursive_report",
        "recursive-heads",
        "ddl-recursive-report.yml",
    );
    push_all(
        &mut owned,
        "recursive_low",
        "recursive-heads",
        "ddl-recursive-low.yml",
    );
    // Original heads stay findings (advisories empty). fk-index drops
    // branch-bodies.sql line 7; constraint-validate line 7 must remain.
    push_all(
        &mut owned,
        "original_heads_report",
        "issue-2731",
        "ddl-heads-report.yml",
    );
    push_all(
        &mut owned,
        "original_heads_low",
        "issue-2731",
        "ddl-heads-low.yml",
    );

    // Three findings on lines 1-3. Messages name postgres-fk-index, the disable
    // comment, and options.unanalyzable advise/ignore. They must not say
    // "rewrite it as supported static".
    push_fk(
        &mut owned,
        "valid_postgres_parser_gaps",
        "unsupported-valid",
        "ddl-valid-parser-gaps.yml",
    );
    // Advisories on lines 4 and 5 (heads contain alpha and beta); rules stay empty.
    push_fk(&mut owned, "nested_do_markers", "nested", "ddl-nested.yml");
    // CLI suppression leaves only line 4. The pre-suppression count stays in the unit test.
    push_fk(
        &mut owned,
        "line_suppression_remaining",
        "suppress",
        "ddl-line-suppress.yml",
    );
    // Only the line 6 advisory remains; file and line directives remove the other three.
    push_fk(
        &mut owned,
        "body_suppression_remaining",
        "body-suppress",
        "ddl-body-suppress.yml",
    );
    push_fk(
        &mut owned,
        "unsupported_line_suppression_remaining",
        "unsupported-suppress",
        "ddl-unsupported-line-suppress.yml",
    );
    // File suppression clears both advisories.
    push_fk(
        &mut owned,
        "unsupported_file_suppression",
        "unsupported-file-suppress",
        "ddl-unsupported-file-suppress.yml",
    );
    // The statement after COPY, line 8, stays a finding. COPY data itself does not.
    push_fk(&mut owned, "copy_following_sql", "copy", "ddl-copy-001.yml");
    // Line 7 is suppressed for fk-index. Line 9 stays an advisory and line 69 stays a finding.
    push_fk(
        &mut owned,
        "rejected_branch_body_default",
        "issue-2731",
        "ddl-branch-default.yml",
    );

    // Each procedural file is the only input. Marker lines from the kept unit
    // test must remain findings; files with no markers stay empty.
    for stem in [
        "valid",
        "unknown",
        "top_level",
        "quoted_do_bodies",
        "malformed_do_suffix",
        "lexical_command_error",
        "return_query",
        "malformed_routine_headers",
        "unknown_procedural_commands",
        "unknown_payloads",
        "top_level_shapes",
        "unknown_loop_query",
        "declaration_cursor_variants",
        "cursor_sql_facts",
        "cursor_first_parameter",
        "dml_query_loops",
        "grammar_options",
        "with_query_calls",
        "grammar_errors",
        "malformed_command_tail",
        "command_followups",
        "block_command_followups",
    ] {
        let file_stem = stem.replace('_', "-");
        push_fk(
            &mut owned,
            &format!("procedural_{stem}"),
            "procedural",
            &format!("ddl-procedural-{file_stem}.yml"),
        );
    }

    let cases: Vec<(&str, PathBuf, PathBuf)> = owned
        .iter()
        .map(|(label, root, config)| (label.as_str(), root.clone(), config.clone()))
        .collect();
    assert_saved_fixture("postgres_unanalyzable_schema", &cases);
}
