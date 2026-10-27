use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::{Path, PathBuf};

fn push(
    cases: &mut Vec<(String, PathBuf, PathBuf)>,
    rule: &str,
    config: &Path,
    scenarios: &[&str],
) {
    for scenario in scenarios {
        cases.push((
            scenario.replace('-', "_"),
            rules_path(rule, "fixture", scenario),
            config.to_path_buf(),
        ));
    }
}

#[test]
fn postgres_no_mutable_sql_registry_check_json_snapshot() {
    let rule = "postgres-no-mutable-sql-registry";
    let config = rules_path(rule, "configs", "registry.yml");
    let mut owned = Vec::new();
    // Each scenario's .no-mistakes.yml is this single rule and importSpecifier.
    // A finding file ends with query.ts and the message names src/tables.ts
    // and src/mutate.ts.
    push(
        &mut owned,
        rule,
        &config,
        &[
            "fail-member-write",
            "fail-computed-key",
            "fail-alias",
            "fail-array",
            "fail-namespace",
            "fail-reexport-alias",
            "fail-export-star",
            "fail-delete",
            "fail-define-property",
        ],
    );
    push(
        &mut owned,
        rule,
        &config,
        &[
            "pass-literal-registry",
            "pass-unrelated-state",
            "pass-bind-value",
            "pass-http-query",
            "pass-suppressed",
            "pass-env-identifier",
            "pass-same-file-write",
        ],
    );
    // One finding: file src/query.ts, line 3, target src/tables.ts, message
    // contains src/mutate.ts.
    push(
        &mut owned,
        rule,
        &config,
        &[
            "fail-literal-computed",
            "fail-dynamic-key",
            "fail-static-require",
            "fail-many-const-requires",
            "fail-require-namespace",
            "fail-namespace-reexport",
            "fail-namespace-named-reexport",
        ],
    );
    push(
        &mut owned,
        rule,
        &config,
        &["pass-dynamic-require", "pass-dynamic-unrelated"],
    );
    let cases: Vec<(&str, PathBuf, PathBuf)> = owned
        .iter()
        .map(|(label, root, config)| (label.as_str(), root.clone(), config.clone()))
        .collect();
    assert_saved_fixture("postgres_no_mutable_sql_registry", &cases);
}
