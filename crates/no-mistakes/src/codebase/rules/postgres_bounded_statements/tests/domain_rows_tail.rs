use super::SchemaCatalog;
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::rules::postgres_bounded_statements::{evaluate, view_bounds::ViewBounds};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/domain-rows-tail")
}

fn catalog() -> SchemaCatalog {
    let root = root();
    let path = root.join("schema.json");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

fn summary(cap: Option<u64>) -> BTreeMap<String, usize> {
    let catalog = catalog();
    let mut views = ViewBounds::default();
    views.domain_rows = cap;
    let sql = std::fs::read_to_string(root().join("sql/lookups.sql")).unwrap();
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
fn unset_reports_each_tail_lookup() {
    assert_eq!(
        summary(None),
        BTreeMap::from([
            ("report tail_consents".to_string(), 1),
            ("report tail_import_rows".to_string(), 1),
        ])
    );
}

#[test]
fn cap_5000_accepts_the_enum_suffix_and_the_joined_check_suffix() {
    assert_eq!(
        summary(Some(5000)),
        BTreeMap::from([
            ("accept tail_consents 3 5000".to_string(), 1),
            ("accept tail_import_rows 500 5000".to_string(), 1),
        ])
    );
}
