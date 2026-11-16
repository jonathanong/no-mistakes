use super::SchemaCatalog;
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::rules::postgres_bounded_statements::{evaluate, view_bounds::ViewBounds};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/domain-rows-alias-shapes")
}

fn summary(cap: Option<u64>) -> BTreeMap<String, usize> {
    summary_of("schema.json", "sql/lookups.sql", cap)
}

fn summary_of(schema: &str, sql_path: &str, cap: Option<u64>) -> BTreeMap<String, usize> {
    let root = root();
    let path = root.join(schema);
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let catalog = SchemaCatalog::load(&root, schema, &sources).unwrap();
    let mut views = ViewBounds::default();
    views.domain_rows = cap;
    let sql = std::fs::read_to_string(root.join(sql_path)).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{facts:?}");
    let mut rows = BTreeMap::new();
    for fact in &facts.bounds {
        let judgment = evaluate::judgment_with_views(fact, &catalog, &views);
        for offender in judgment.offenders {
            let label = match offender.domain_over {
                Some(product) => format!("over {table} {product}", table = offender.table),
                None => format!("report {table}", table = offender.table),
            };
            *rows.entry(label).or_insert(0) += 1;
        }
        for accept in judgment.accepts {
            let label = format!(
                "accept {table} {product} {cap}",
                table = accept.table,
                product = accept.product,
                cap = accept.cap
            );
            *rows.entry(label).or_insert(0) += 1;
        }
    }
    rows
}

#[test]
fn unset_reports_every_alias_shape() {
    assert_eq!(
        summary(None),
        BTreeMap::from([
            ("report seats".to_string(), 2),
            ("report shadows".to_string(), 1),
            ("report gaps".to_string(), 1),
            ("report others".to_string(), 1),
            ("report audit.bands".to_string(), 1),
            ("report audit.misses".to_string(), 1),
        ])
    );
}

#[test]
fn cap_5_accepts_partial_alias_gap_and_qualified_key() {
    assert_eq!(
        summary(Some(5)),
        BTreeMap::from([
            ("accept seats 5 5".to_string(), 1),
            ("report seats".to_string(), 1),
            ("report shadows".to_string(), 1),
            ("accept gaps 5 5".to_string(), 1),
            ("report others".to_string(), 1),
            ("accept audit.bands 5 5".to_string(), 1),
            ("report audit.misses".to_string(), 1),
        ])
    );
}

#[test]
fn cap_5_keeps_a_schema_qualifier_when_the_catalog_names_no_schema() {
    assert_eq!(
        summary_of("schema-unscoped.json", "sql/unscoped.sql", Some(5)),
        BTreeMap::from([("report plain".to_string(), 1)])
    );
}
