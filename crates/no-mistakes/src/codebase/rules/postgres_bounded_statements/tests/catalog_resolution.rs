use super::RULE_ID;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog, SqlVerdictStatus};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn root() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-bounded-statements").join("catalog-resolution")
}

fn config(name: &str) -> NoMistakesConfig {
    crate::config::v2::load_v2_config(&root(), Some(&root().join(name))).unwrap()
}

fn check(
    config_name: &str,
    sql_name: &str,
) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    let root = root();
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(config_name)))?;
    let files = vec![root.join("catalog.json"), root.join("sql").join(sql_name)];
    super::check_with_files(&root, &config, &files)
}

#[test]
fn unqualified_unique_key_operators_need_independent_caller_values() {
    let sql = std::fs::read_to_string(root().join("sql/key-operators.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{facts:#?}");
    for index in [1, 2] {
        let fact = &facts.bounds[index];
        assert!(fact.query.items.iter().any(|item| {
            item.pins
                .iter()
                .any(|pin| pin.column == "email_intake_id" && pin.unqualified_candidate)
        }));
    }
}

#[test]
fn unqualified_candidate_evidence_is_reused_with_catalog_specific_ownership() {
    let sql = std::fs::read_to_string(root().join("sql/key-resolution.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{facts:#?}");
    let fact = &facts.bounds[0];
    assert!(fact.query.items.iter().any(|item| {
        item.pins
            .iter()
            .any(|pin| pin.column == "email_intake_id" && pin.unqualified_candidate)
    }));
}

#[test]
fn unconfigured_views_remain_unknown_under_advisory_and_ignore_policies() {
    for (config_name, advisory) in [
        ("view-unconfigured-advise.yml", true),
        ("view-unconfigured-ignore.yml", false),
        ("view-unconfigured-fail.yml", false),
    ] {
        let root = root();
        let config = config(config_name);
        let files = vec![
            root.join("catalog.json"),
            root.join("sql/view-unconfigured.sql"),
            root.join("src/unconfigured-view.ts"),
        ];
        let sources = crate::codebase::rules::source_store_for_files(&files);
        let mut facts = crate::codebase::postgres::prepare_rule_sql_facts(
            &root,
            &files,
            sources.clone(),
            &config,
            &[RULE_ID],
        )
        .unwrap();
        facts.enable_sql_verdicts();
        // Recording the check fills sql verdicts. Findings for these configs are snapshotted.
        crate::codebase::rules::postgres_bounded_statements::check_with_files_sources_and_facts(
            &root, &config, &files, &sources, &facts,
        )
        .unwrap();
        let verdicts = facts.sql_verdicts();
        let select_verdicts: Vec<_> = verdicts
            .iter()
            .filter(|verdict| verdict.subcheck == "select")
            .collect();
        assert_eq!(select_verdicts.len(), 1, "{config_name}: {verdicts:#?}");
        assert_eq!(select_verdicts[0].status, SqlVerdictStatus::Abstain);
        assert_eq!(select_verdicts[0].advisory, advisory);
        assert!(select_verdicts[0]
            .residual_reasons
            .iter()
            .any(|reason| reason == "unknown-relation"));
        assert!(verdicts
            .iter()
            .filter(|verdict| verdict.subcheck != "select")
            .all(|verdict| verdict.status == SqlVerdictStatus::ProvenClear));
    }
}

#[test]
fn partial_catalog_reports_both_relations_on_the_first_statement() {
    // CLI check rejects this ordering-only catalog before it emits JSON.
    let root = root();
    let sql = std::fs::read_to_string(root.join("sql/key-resolution.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    let partial = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("catalog-partial.json")).unwrap(),
    )
    .unwrap();
    let tables: Vec<_> = super::super::evaluate::offenders(&facts.bounds[0], &partial)
        .into_iter()
        .map(|finding| finding.table)
        .collect();
    assert_eq!(tables.len(), 2, "{tables:?}");
}

#[test]
fn invalid_view_policy_keys_are_rejected() {
    for name in [
        "view-bounds-missing.yml",
        "view-bounds-ambiguous.yml",
        "view-bounds-duplicate-alias.yml",
        "view-bounds-table-collision.yml",
    ] {
        let error = check(name, "view-allow.sql").unwrap_err();
        assert!(
            format!("{error:#}")
                .to_ascii_lowercase()
                .contains("viewbounds"),
            "{name}: {error:#}"
        );
    }
}
