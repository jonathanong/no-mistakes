use super::fixture_root;
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::postgres::statements::SqlBoundKind;
use crate::codebase::postgres::{CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::postgres_bounded_statements::evaluate::offenders;

fn remainder_catalog() -> SchemaCatalog {
    let path = fixture_root().join("schema-2961-remainder.json");
    SchemaCatalog::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn sql(name: &str) -> String {
    std::fs::read_to_string(fixture_root().join("sql").join(name)).unwrap()
}

fn offending_kinds(name: &str, catalog: &SchemaCatalog) -> Vec<SqlBoundKind> {
    let facts = extract_sql_statement_facts(&sql(name));
    assert!(!facts.parse_failed, "{name}: {facts:?}");
    facts
        .bounds
        .iter()
        .filter(|fact| !offenders(fact, catalog).is_empty())
        .map(|fact| fact.kind)
        .collect()
}

#[test]
fn materialized_page_limits_bound_the_returning_read() {
    let catalog = remainder_catalog();
    for name in ["issue-2961-page-limit-6.sql", "issue-2961-page-limit-5.sql"] {
        assert!(
            offending_kinds(name, &catalog).is_empty(),
            "{name}: {:?}",
            offending_kinds(name, &catalog)
        );
    }
    let kinds = offending_kinds("issue-2961-page-uncapped.sql", &catalog);
    assert!(kinds.contains(&SqlBoundKind::Select), "{kinds:?}");
}

#[test]
fn a_catalog_view_without_a_policy_is_an_unsized_relation() {
    let catalog = remainder_catalog();
    let open = extract_sql_statement_facts("SELECT id FROM view_users_public");
    let hits = offenders(&open.bounds[0], &catalog);
    assert_eq!(hits.len(), 1);
    assert!(matches!(
        &hits[0].object,
        CatalogObjectRef::View(name) if name.contains("view_users_public")
    ));
    let capped = extract_sql_statement_facts("SELECT id FROM view_users_public LIMIT 1");
    assert!(offenders(&capped.bounds[0], &catalog).is_empty());
    let missing = extract_sql_statement_facts("SELECT id FROM relation_not_in_catalog");
    assert!(offenders(&missing.bounds[0], &catalog).is_empty());
}

#[test]
fn a_bare_name_shared_by_a_table_and_a_view_stays_unproven() {
    let loaded = remainder_catalog();
    let facts = extract_sql_statement_facts("SELECT id FROM active_accounts WHERE id = $1");
    assert!(!facts.parse_failed, "{facts:?}");
    assert!(offenders(&facts.bounds[0], &loaded).is_empty());
    // The shared fixture catalog has no such collision and still proves the table.
    let shared = super::catalog();
    let proved = extract_sql_statement_facts("SELECT id FROM accounts WHERE id = $1");
    assert!(offenders(&proved.bounds[0], &shared).is_empty());
}
