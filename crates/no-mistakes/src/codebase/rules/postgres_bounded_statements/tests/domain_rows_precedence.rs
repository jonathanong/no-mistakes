use super::SchemaCatalog;
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::rules::postgres_bounded_statements::{evaluate, view_bounds::ViewBounds};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements")
        .join(name)
}

fn summary(name: &str, cap: Option<u64>) -> BTreeMap<String, usize> {
    let root = root(name);
    let path = root.join("schema.json");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    let catalog = SchemaCatalog::load(&root, "schema.json", &sources).unwrap();
    let mut views = ViewBounds::default();
    views.domain_rows = cap;
    let sql = std::fs::read_to_string(root.join("sql/lookups.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{facts:?}");
    let mut rows = BTreeMap::new();
    for fact in &facts.bounds {
        let judgment = evaluate::judgment_with_views(fact, &catalog, &views);
        for offender in judgment.offenders {
            *rows
                .entry(format!("report {table}", table = offender.table))
                .or_insert(0) += 1;
        }
        for accept in judgment.accepts {
            *rows
                .entry(format!(
                    "accept {table} {product}",
                    table = accept.table,
                    product = accept.product
                ))
                .or_insert(0) += 1;
        }
    }
    rows
}

fn expect(rows: &[(&str, usize)]) -> BTreeMap<String, usize> {
    rows.iter()
        .map(|(label, count)| ((*label).to_string(), *count))
        .collect()
}

#[test]
fn a_limit_or_an_open_scan_does_not_take_a_domain_product() {
    let bounded = expect(&[("report mixed", 2), ("report notes", 1)]);
    assert_eq!(summary("domain-rows-precedence", None), bounded);
    assert_eq!(
        summary("domain-rows-precedence", Some(5000)),
        expect(&[
            ("accept mixed 10", 1),
            ("report mixed", 1),
            ("report notes", 1),
        ])
    );
    assert_eq!(
        summary("domain-rows-redundant-allow", None),
        expect(&[("report finite_codes", 1), ("report open_notes", 1),])
    );
    assert_eq!(
        summary("domain-rows-redundant-allow", Some(5000)),
        expect(&[("accept finite_codes 3", 1), ("report open_notes", 1)])
    );
}
