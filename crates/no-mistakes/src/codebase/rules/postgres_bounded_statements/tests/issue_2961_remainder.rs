use super::fixture_root;
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::postgres::statements::SqlBoundKind;
use crate::codebase::postgres::{CatalogObjectRef, SchemaCatalog};
use crate::codebase::rules::postgres_bounded_statements::evaluate::{
    offenders, offenders_with_views,
};
use crate::codebase::rules::postgres_bounded_statements::view_bounds::{ViewBoundMode, ViewBounds};
use std::collections::BTreeMap;

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
fn a_one_to_one_view_key_bounds_a_lookup_or_join() {
    let catalog = remainder_catalog();
    for sql in [
        "SELECT * FROM view_feeds WHERE view_feeds.id = $1",
        "WITH sel AS MATERIALIZED (SELECT id FROM base WHERE owner_id = $1 ORDER BY id DESC LIMIT 1) SELECT v.* FROM sel JOIN view_base v ON v.id = sel.id",
        "SELECT t.id FROM things t JOIN view_people p ON p.id = t.updated_by_id WHERE t.id = $1",
        "WITH c AS (SELECT id FROM items WHERE id = $1 UNION ALL SELECT id FROM items WHERE id = $2) SELECT v.id FROM c JOIN view_items v ON c.id = v.id",
        "SELECT id FROM view_users_public WHERE view_users_public.id = $1",
    ] {
        let facts = extract_sql_statement_facts(sql);
        assert!(!facts.parse_failed, "{sql}: {facts:?}");
        assert!(
            facts
                .bounds
                .iter()
                .all(|fact| offenders(fact, &catalog).is_empty()),
            "{sql}: {:?}",
            facts
                .bounds
                .iter()
                .map(|fact| offenders(fact, &catalog))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn an_unkeyed_view_is_not_a_finding_and_a_keyed_view_without_a_pin_is() {
    let catalog = remainder_catalog();
    for sql in [
        "SELECT n FROM view_counts",
        "SELECT status FROM view_status WHERE status = $1",
        "SELECT id FROM view_union WHERE id = $1",
        "SELECT id FROM view_distinct WHERE id = $1",
        "SELECT id FROM view_grouped WHERE id = $1",
        "SELECT id FROM view_limited WHERE id = $1",
        "SELECT id FROM view_inner WHERE view_inner.id = $1",
    ] {
        let facts = extract_sql_statement_facts(sql);
        assert!(!facts.parse_failed, "{sql}: {facts:?}");
        assert!(offenders(&facts.bounds[0], &catalog).is_empty(), "{sql}");
    }
    let open = extract_sql_statement_facts("SELECT id FROM view_feeds");
    let hits = offenders(&open.bounds[0], &catalog);
    assert_eq!(hits.len(), 1);
    assert!(matches!(
        &hits[0].object,
        CatalogObjectRef::View(name) if name.contains("view_feeds")
    ));
    let logs = extract_sql_statement_facts(
        "SELECT l.id FROM logs l JOIN items i ON true LEFT JOIN view_people v ON v.id = l.person_id WHERE l.id = $1",
    );
    let roots: Vec<_> = offenders(&logs.bounds[0], &catalog)
        .into_iter()
        .map(|offender| offender.table)
        .collect();
    assert_eq!(roots, ["items"]);
}

#[test]
fn an_unbounded_view_policy_stays_a_finding_when_its_key_is_pinned() {
    let catalog = remainder_catalog();
    let views = ViewBounds::compile(
        &BTreeMap::from([("view_feeds".to_string(), ViewBoundMode::Unbounded)]),
        &catalog,
    )
    .unwrap();
    let facts = extract_sql_statement_facts("SELECT id FROM view_feeds WHERE view_feeds.id = $1");
    assert!(!facts.parse_failed, "{facts:?}");
    let hits = offenders_with_views(&facts.bounds[0], &catalog, &views);
    assert!(matches!(
        hits.first().map(|hit| &hit.object),
        Some(CatalogObjectRef::View(name)) if name.contains("view_feeds")
    ));
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
