mod config;
mod foreign;
mod links;
mod names;
mod naming_regressions;
mod support;

use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

use support::PROPOSED;

#[test]
fn rule_id_is_stable() {
    assert_eq!(super::RULE_ID, "postgres-column-naming");
}

#[test]
fn only_a_catalog_path_reports_nothing() {
    support::expect_none(
        "schemaCatalogPath: schema.json\n",
        support::column("invoices", "whatever", "text"),
    );
}

#[test]
fn proposed_config_reports_unused_allow_and_exempt_entries() {
    assert_eq!(
        support::messages(PROPOSED, support::fixture_body("scenarios/mod-00.json")),
        vec![
            "db/schema.json: stale postgres-column-naming allow entry: column:export_jobs.runner_job_id".to_string(),
            "db/schema.json: stale postgres-column-naming allow entry: column:links.url".to_string(),
            "db/schema.json: stale postgres-column-naming allow entry: column:sessions.expires".to_string(),
            "db/schema.json: stale postgres-column-naming requireForeignKey exempt entry: (^|_)(device|session)_id$".to_string(),
            "db/schema.json: stale postgres-column-naming requireForeignKey exempt entry: ^cursor_".to_string(),
        ]
    );
}

#[test]
fn allow_suppresses_a_real_finding_and_is_not_stale() {
    let messages = support::messages(PROPOSED, support::fixture_body("scenarios/mod-01.json"));
    assert!(messages.is_empty(), "{messages:#?}");
}

#[test]
fn check_with_files_reads_a_prepared_catalog() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/column-naming/pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(
            "schemaCatalogPath: schema.json\ntypeRules:\n  - types: ['timestamp with time zone']\n    namePattern: '_at$'\n",
        )
        .unwrap(),
        ..RuleDef::default()
    });
    let findings = super::check_with_files(&root, &config, &[root.join("schema.json")]).unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
    let again = super::check_with_files(&root, &config, &[root.join("schema.json")]).unwrap();
    assert_eq!(findings, again);
}

#[test]
fn custom_message_and_include_filter_the_catalog() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/column-naming/pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        message: Some("rename the column".to_string()),
        options: serde_yaml::from_str(
            "schemaCatalogPath: schema.json\ntypeRules:\n  - types: ['timestamp with time zone']\n    namePattern: '^created'\n",
        )
        .unwrap(),
        ..RuleDef::default()
    });
    let messages = super::check_with_files(&root, &config, &[root.join("schema.json")])
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec!["schema.json: column:invoices.due_at: rename the column".to_string()]
    );
    config.rules[0].include = vec!["missing.json".to_string()];
    assert!(
        super::check_with_files(&root, &config, &[root.join("schema.json")])
            .unwrap()
            .is_empty()
    );
    config.rules[0].include.clear();
    config.rules[0].exclude = vec!["schema.json".to_string()];
    assert!(
        super::check_with_files(&root, &config, &[root.join("schema.json")])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn missing_catalog_file_is_an_error() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/column-naming/pass");
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("schemaCatalogPath: missing.json").unwrap(),
        ..RuleDef::default()
    });
    let error = super::check_with_files(&root, &config, &[root.join("schema.json")]).unwrap_err();
    assert!(error.to_string().contains("missing.json"), "{error}");
}

#[test]
fn jsonc_catalogs_support_standard_column_suppression_directives() {
    for fixture in ["line-disable", "next-line-disable", "file-disable"] {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-column-naming/fixture")
            .join(fixture);
        let file = root.join("schema.json");
        let mut config = NoMistakesConfig::default();
        config.rules.push(RuleDef {
            rule: super::RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(
                "schemaCatalogPath: schema.json\ntypeRules:\n  - types: ['timestamp with time zone']\n    namePattern: '_at$'\n",
            )
            .unwrap(),
            ..RuleDef::default()
        });
        let mut findings =
            super::check_with_files(&root, &config, std::slice::from_ref(&file)).unwrap();
        if fixture == "file-disable" {
            assert_eq!(findings.len(), 1, "fixture {fixture}");
        } else {
            assert_eq!(findings.len(), 2, "fixture {fixture}");
            assert!(findings.iter().all(|finding| finding.line > 1));
        }
        let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
        super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
        if fixture == "file-disable" {
            assert!(findings.is_empty(), "fixture {fixture}: {findings:#?}");
        } else {
            assert_eq!(findings.len(), 1, "fixture {fixture}: {findings:#?}");
            assert_eq!(
                findings[0].target.as_deref(),
                Some("column:invoices.expires")
            );
        }
    }
}

#[test]
fn project_scoped_rules_check_catalogs_inside_the_selected_project() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/column-naming/project");
    let catalog = root.join("db/schema.json");
    let mut config = NoMistakesConfig::default();
    config.projects.insert(
        "database".to_string(),
        crate::config::v2::schema::Project {
            root: Some("db".to_string()),
            ..Default::default()
        },
    );
    config.rules.push(RuleDef {
        rule: super::RULE_ID.to_string(),
        projects: vec!["database".to_string()],
        options: serde_yaml::from_str(
            "schemaCatalogPath: db/schema.json\ntypeRules:\n  - types: ['timestamp with time zone']\n    namePattern: '_at$'\n",
        )
        .unwrap(),
        ..RuleDef::default()
    });
    let findings = super::check_with_files(&root, &config, &[catalog]).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
}
