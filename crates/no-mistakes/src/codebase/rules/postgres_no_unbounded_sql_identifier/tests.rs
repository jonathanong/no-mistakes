use super::RULE_ID;
use std::path::PathBuf;

fn root(scenario: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-no-unbounded-sql-identifier/fixture")
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
fn env_imported_config_and_type_assertions_are_unbounded() {
    for scenario in ["fail-env", "fail-imported-config", "fail-type-assertion"] {
        let found = findings(scenario);
        assert!(
            found
                .iter()
                .any(|finding| finding.file.ends_with("query.ts")
                    && finding.message.contains("finite-domain validator")),
            "{scenario}: {found:#?}"
        );
    }
}

#[test]
fn switch_bind_http_and_suppression_stay_silent() {
    for scenario in [
        "pass-switch",
        "pass-bind-value",
        "pass-http-query",
        "pass-suppressed",
        "pass-imported-process",
    ] {
        assert!(
            findings(scenario).is_empty(),
            "{scenario}: {:#?}",
            findings(scenario)
        );
    }
}
