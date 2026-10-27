use crate::codebase::postgres::{SchemaCatalog, SqlVerdictStatus};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-bounded-statements")
        .join("catalog-resolution-identity")
}

fn config(name: &str) -> NoMistakesConfig {
    crate::config::v2::load_v2_config(&fixture_root(), Some(&fixture_root().join(name))).unwrap()
}

fn files(catalog: &str, sql: &str) -> Vec<PathBuf> {
    let root = fixture_root();
    vec![root.join(catalog), root.join("sql").join(sql)]
}

#[test]
fn schema_qualified_view_policy_never_borrows_same_named_table_keys() {
    let root = fixture_root();
    let mut files = files("catalog-public-collision.json", "qualified-relations.sql");
    files.push(root.join("src/unqualified-collision.ts"));
    let config = config("view-identity.yml");
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        sources.clone(),
        &config,
        &[super::RULE_ID],
    )
    .unwrap();
    facts.enable_sql_verdicts();
    // Recording the check fills sql verdicts. The finding itself is in the snapshot.
    crate::codebase::rules::postgres_bounded_statements::check_with_files_sources_and_facts(
        &root, &config, &files, &sources, &facts,
    )
    .unwrap();

    let select_verdict = facts
        .sql_verdicts()
        .into_iter()
        .find(|verdict| verdict.subcheck == "select")
        .expect("embedded bare relation should keep an unknown SELECT verdict");
    assert_eq!(select_verdict.status, SqlVerdictStatus::Abstain);
    assert!(select_verdict.advisory);
    assert!(select_verdict
        .residual_reasons
        .iter()
        .any(|reason| reason == "unknown-relation"));
}

#[test]
fn schema_qualified_policy_keys_need_matching_catalog_schema_fallback() {
    let root = fixture_root();
    let no_schema = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("catalog-no-schema.json")).unwrap(),
    )
    .unwrap();
    assert!(no_schema.view_relation("archive.active_accounts").is_none());

    let error = crate::codebase::rules::postgres_bounded_statements::check_with_files(
        &root,
        &config("no-schema-view-policy.yml"),
        &files("catalog-no-schema.json", "bare-view.sql"),
    )
    .unwrap_err();
    assert!(
        format!("{error:#}")
            .to_ascii_lowercase()
            .contains("viewbounds"),
        "{error:#}"
    );

    let archive = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("catalog-archive-schema.json")).unwrap(),
    )
    .unwrap();
    assert!(archive.view_relation("archive.active_accounts").is_some());
}
