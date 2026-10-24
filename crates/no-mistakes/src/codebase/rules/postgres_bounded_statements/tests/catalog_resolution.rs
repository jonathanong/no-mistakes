use super::{check_with_files, offenders, RULE_ID};
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog, SqlVerdictStatus};
use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

fn root() -> PathBuf {
    crate::test_support::rule_fixture_root("postgres-bounded-statements").join("catalog-resolution")
}

fn config(name: &str) -> NoMistakesConfig {
    crate::config::v2::load_v2_config(&root(), Some(&root().join(name))).unwrap()
}

fn sql_files(name: &str, catalog: &str) -> Vec<PathBuf> {
    vec![root().join(catalog), root().join("sql").join(name)]
}

fn check(
    config_name: &str,
    sql_name: &str,
) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    let root = root();
    let config = crate::config::v2::load_v2_config(&root, Some(&root.join(config_name)))?;
    let files = sql_files(sql_name, "catalog.json");
    check_with_files(&root, &config, &files)
}

fn names(fact: &crate::codebase::postgres::SqlBoundFact, catalog: &SchemaCatalog) -> Vec<String> {
    offenders(fact, catalog)
        .into_iter()
        .map(|finding| finding.table)
        .collect()
}

#[test]
fn a_unique_unqualified_key_owner_bounds_joined_known_tables() {
    let findings = check("key-resolution.yml", "key-resolution.sql").unwrap();
    let reported: Vec<_> = findings
        .iter()
        .map(|finding| (finding.line, finding.target.as_deref().unwrap()))
        .collect();
    assert_eq!(
        reported,
        [
            (5, "table:public.ambiguous_left"),
            (5, "table:public.ambiguous_right"),
            (7, "table:public.customer_accounts"),
            (9, "table:public.customer_accounts"),
            (11, "table:public.customer_accounts"),
        ]
    );
}

#[test]
fn unqualified_unique_key_operators_need_independent_caller_values() {
    let findings = check("key-operators.yml", "key-operators.sql").unwrap();
    let reported: std::collections::BTreeSet<_> = findings
        .iter()
        .map(|finding| (finding.line, finding.target.as_deref().unwrap()))
        .collect();
    assert_eq!(
        reported,
        [
            (6, "table:public.customer_accounts"),
            (6, "table:public.owner_jobs"),
            (8, "table:public.customer_accounts"),
            (8, "table:public.owner_jobs"),
        ]
        .into_iter()
        .collect()
    );

    let root = root();
    let sql = std::fs::read_to_string(root.join("sql/key-operators.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{facts:#?}");
    let ambiguous = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("catalog-ambiguous-owner.json")).unwrap(),
    )
    .unwrap();
    for index in [1, 2] {
        let fact = &facts.bounds[index];
        assert!(fact.query.items.iter().any(|item| {
            item.pins
                .iter()
                .any(|pin| pin.column == "email_intake_id" && pin.unqualified_candidate)
        }));
        let ambiguous_names = names(fact, &ambiguous);
        assert!(ambiguous_names
            .iter()
            .any(|name| name == "public.customer_accounts"));
        assert!(ambiguous_names
            .iter()
            .any(|name| name == "public.owner_jobs"));
    }
}

#[test]
fn unqualified_candidate_evidence_is_reused_with_catalog_specific_ownership() {
    let root = root();
    let sql = std::fs::read_to_string(root.join("sql/key-resolution.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{facts:#?}");
    let fact = &facts.bounds[0];
    assert!(fact.query.items.iter().any(|item| {
        item.pins
            .iter()
            .any(|pin| pin.column == "email_intake_id" && pin.unqualified_candidate)
    }));

    let unique =
        SchemaCatalog::from_json(&std::fs::read_to_string(root.join("catalog.json")).unwrap())
            .unwrap();
    let ambiguous = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("catalog-ambiguous-owner.json")).unwrap(),
    )
    .unwrap();
    assert!(names(fact, &unique).is_empty());
    let ambiguous_names = names(fact, &ambiguous);
    assert!(ambiguous_names
        .iter()
        .any(|name| name == "public.customer_accounts"));
    assert!(ambiguous_names
        .iter()
        .any(|name| name == "public.owner_jobs"));
}

#[test]
fn explicit_view_bounds_cover_finite_views_and_report_unbounded_views() {
    let findings = check("view-bounds.yml", "view-bounds.sql").unwrap();
    let reported: Vec<_> = findings
        .iter()
        .map(|finding| (finding.line, finding.target.as_deref().unwrap()))
        .collect();
    assert_eq!(reported, [(3, "view:public.unbounded_archive")]);
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
        let findings = crate::codebase::rules::postgres_bounded_statements::check_with_files_sources_and_facts(
            &root, &config, &files, &sources, &facts,
        )
        .unwrap();
        assert!(findings.is_empty(), "{config_name}: {findings:#?}");
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

#[test]
fn view_allow_entries_and_line_suppressions_cover_unbounded_view_findings() {
    assert!(check("view-allow.yml", "view-allow.sql")
        .unwrap()
        .is_empty());
    let root = root();
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("view-suppression.yml")), None)
            .unwrap();
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn boolean_key_ownership_and_incomplete_catalogs_remain_distinct() {
    let found = check("key-booleans.yml", "key-booleans.sql").unwrap();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].target.as_deref(),
        Some("table:public.customer_accounts")
    );
    let root = root();
    let sql = std::fs::read_to_string(root.join("sql/key-resolution.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    let partial = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("catalog-partial.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(names(&facts.bounds[0], &partial).len(), 2);
}
