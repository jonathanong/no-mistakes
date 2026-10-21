use super::RULE_ID;
use std::path::PathBuf;

fn root(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-mutable-sql-registry/fixture")
        .join(scenario)
}

fn findings(scenario: &str) -> Vec<crate::codebase::rules::RuleFinding> {
    crate::codebase::rules::run_check(&root(scenario), None, None)
        .unwrap()
        .into_iter()
        .filter(|finding| finding.rule == RULE_ID)
        .collect()
}

#[test]
fn member_write_alias_and_array_assignment_reach_sql() {
    for scenario in ["fail-member-write", "fail-alias", "fail-array"] {
        let found = findings(scenario);
        assert!(
            found.iter().any(|finding| {
                finding.file.ends_with("query.ts")
                    && finding.message.contains("src/tables.ts")
                    && finding.message.contains("src/mutate.ts")
            }),
            "{scenario}: {found:#?}"
        );
    }
}

#[test]
fn literal_unrelated_prototype_bind_http_and_suppression_stay_silent() {
    for scenario in [
        "pass-literal-registry",
        "pass-unrelated-state",
        "pass-prototype-key",
        "pass-bind-value",
        "pass-http-query",
        "pass-suppressed",
        "pass-namespace",
    ] {
        assert!(
            findings(scenario).is_empty(),
            "{scenario}: {:#?}",
            findings(scenario)
        );
    }
}
