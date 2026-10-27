use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

// Output cases also recorded in cli_postgres_snapshot_undeclared_executors.
// These tests stay: verdict ledgers, isolated counts beside annotation
// findings, null importSpecifier, empty facts, and invalid options.

const RULE: &str = "postgres-require-query-annotation";

fn inputs(policy: &str) -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts/embedded");
    let file = root.join("undeclared-executor-types.ts");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE.into(), scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(&format!(
            "importSpecifier: '@example/db'\nexecutorNames: [query, execute]\nexecutorTypeNames: [Executor]\nunanalyzableSql: {policy}"
        )), ..Default::default()
    });
    (root, config, vec![file])
}

#[test]
fn missing_types_follow_policy_without_becoming_executed_sql() {
    for (policy, expected_findings, advisory) in [
        ("report", 9, false),
        ("advise", 0, true),
        ("ignore", 0, false),
    ] {
        let (root, config, files) = inputs(policy);
        let sources = super::super::source_store_for_files(&files);
        let mut facts = crate::codebase::postgres::prepare_rule_sql_facts(
            &root,
            &files,
            sources,
            &config,
            &[RULE],
        )
        .unwrap();
        facts.enable_sql_verdicts();
        let findings = check(&root, &config, RULE, &files, &facts).unwrap();
        assert_eq!(findings.len(), expected_findings, "{policy}: {findings:?}");
        let verdicts = facts.sql_verdicts();
        assert_eq!(verdicts.len(), 9);
        assert!(verdicts
            .iter()
            .all(|v| v.status == SqlVerdictStatus::Abstain && v.advisory == advisory));
        let omitted = verdicts
            .iter()
            .filter(|v| v.subcheck == "undeclared-executor-type")
            .collect::<Vec<_>>();
        assert_eq!(omitted.len(), 8);
        assert!(omitted
            .iter()
            .all(|v| v.residual_reasons[0].contains("PoolClient")
                && v.residual_reasons[0].contains("executorTypeNames")));
        let unresolved = verdicts
            .iter()
            .filter(|v| v.subcheck != "undeclared-executor-type")
            .collect::<Vec<_>>();
        assert_eq!(unresolved.len(), 1);
        assert!(unresolved[0].residual_reasons[0]
            .contains("configured executor receiver of type `Executor`"));
        assert!(verdicts
            .iter()
            .all(|v| v.application.as_ref().unwrap().index == 0));
    }
}

#[test]
fn declared_imported_types_keep_type_specific_diagnostics_through_constructor_aliases() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/postgres-facts/embedded");
    let files = vec![root.join("undeclared-constructor-alias.ts")];
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE.into(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(
            "importSpecifier: '@example/db'",
        ),
        ..Default::default()
    });
    let sources = super::super::source_store_for_files(&files);
    let facts =
        crate::codebase::postgres::prepare_rule_sql_facts(&root, &files, sources, &config, &[RULE])
            .unwrap();
    let findings = check(&root, &config, RULE, &files, &facts).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(findings[0]
        .message
        .contains("undeclared executor type `PoolClient`"));
    assert!(!findings[0]
        .message
        .contains("has no declared executor type"));
}

#[test]
fn ignore_without_ledger_does_no_diagnostic_work_and_scope_excludes_calls() {
    let (root, mut config, files) = inputs("ignore");
    assert!(
        check(&root, &config, RULE, &files, &CheckFactMap::default())
            .unwrap()
            .is_empty()
    );
    config.rules[0].options["unanalyzableSql"] = "report".into();
    config.rules[0].options["exclude"] = serde_yaml::from_str("['**/*.ts']").unwrap();
    assert!(
        check(&root, &config, RULE, &files, &CheckFactMap::default())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn absent_null_and_empty_module_profiles_do_not_request_facts() {
    for value in [None, Some(serde_yaml::Value::Null), Some("".into())] {
        let (root, mut config, files) = inputs("report");
        config.rules[0]
            .options
            .as_mapping_mut()
            .unwrap()
            .remove(serde_yaml::Value::String("importSpecifier".into()));
        if let Some(value) = value {
            config.rules[0].options["importSpecifier"] = value;
        }
        assert!(
            check(&root, &config, RULE, &files, &CheckFactMap::default())
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn unavailable_facts_cannot_prove_an_undeclared_receiver_type() {
    let (root, config, files) = inputs("report");
    assert!(
        check(&root, &config, RULE, &files, &CheckFactMap::default())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn invalid_policy_options_and_scopes_are_errors_not_silent_omissions() {
    // Even with no prepared facts, invalid configuration must remain visible.
    for (key, value) in [
        ("unanalyzable", serde_yaml::Value::String("invalid".into())),
        ("executorNames", serde_yaml::Value::String("query".into())),
        (
            "include",
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("[".into())]),
        ),
    ] {
        let (root, mut config, files) = inputs("report");
        config.rules[0].options[key] = value;
        assert!(
            check(&root, &config, RULE, &files, &CheckFactMap::default()).is_err(),
            "invalid {key} must not hide the configuration error"
        );
    }
    let (root, mut config, files) = inputs("report");
    config.rules[0].include = vec!["[".into()];
    assert!(check(&root, &config, RULE, &files, &CheckFactMap::default()).is_err());
}
