use super::SchemaCatalog;
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::rules::postgres_bounded_statements::{
    column_bounds::ColumnBounds, evaluate, view_bounds::ViewBounds, Options,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/domain-rows")
}

fn catalog() -> SchemaCatalog {
    let root = root();
    let path = root.join("schema.json");
    let sources = crate::codebase::rules::source_store_for_files(std::slice::from_ref(&path));
    SchemaCatalog::load(&root, "schema.json", &sources).unwrap()
}

fn views(cap: Option<u64>) -> ViewBounds {
    let mut views = ViewBounds::default();
    views.domain_rows = cap;
    views
}

fn summary(cap: Option<u64>) -> BTreeMap<String, usize> {
    let catalog = catalog();
    let views = views(cap);
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

fn expect(rows: &[(&str, usize)]) -> BTreeMap<String, usize> {
    rows.iter()
        .map(|(label, count)| ((*label).to_string(), *count))
        .collect()
}

#[test]
fn unset_reports_every_saved_lookup_without_a_product() {
    assert_eq!(
        summary(None),
        expect(&[
            ("report questions", 5),
            ("report participants", 1),
            ("report daily", 1),
            ("report wide_slots", 1),
            ("report flags", 1),
            ("report ranks", 1),
            ("report maybe_flags", 1),
            ("report drafts", 1),
            ("report open_days", 1),
            ("report spans", 2),
        ])
    );
}

#[test]
fn cap_5000_accepts_finite_domains_and_names_products_above_it() {
    assert_eq!(
        summary(Some(5000)),
        expect(&[
            ("report questions", 5),
            ("accept participants 50 5000", 1),
            ("accept daily 1460 5000", 1),
            ("over wide_slots 5001", 1),
            ("accept flags 2 5000", 1),
            ("accept ranks 10 5000", 1),
            ("report maybe_flags", 1),
            ("report drafts", 1),
            ("report open_days", 1),
            ("accept spans 80 5000", 1),
            ("over spans 6000", 1),
        ])
    );
}

#[test]
fn cap_50_accepts_the_equal_product_and_reports_larger_ones() {
    assert_eq!(
        summary(Some(50)),
        expect(&[
            ("report questions", 5),
            ("accept participants 50 50", 1),
            ("over daily 1460", 1),
            ("over wide_slots 5001", 1),
            ("accept flags 2 50", 1),
            ("accept ranks 10 50", 1),
            ("report maybe_flags", 1),
            ("report drafts", 1),
            ("report open_days", 1),
            ("over spans 80", 1),
            ("over spans 6000", 1),
        ])
    );
}

#[test]
fn zero_cap_accepts_no_domain_product() {
    let rows = summary(Some(0));
    assert_eq!(rows.get("over participants 50"), Some(&1));
    assert_eq!(rows.get("over flags 2"), Some(&1));
    assert_eq!(rows.get("over ranks 10"), Some(&1));
    assert!(
        rows.keys().all(|label| !label.starts_with("accept ")),
        "{rows:?}"
    );
}

#[test]
fn column_bounds_do_not_take_a_domain_product() {
    let catalog = catalog();
    let options: Options =
        serde_yaml::from_str("columnBounds: {labeled.bucket: {reason: five labels}}").unwrap();
    let mut views = ViewBounds::default();
    views.domain_rows = Some(5000);
    views.keys = ColumnBounds::compile(&options.column_bounds, &catalog).unwrap();
    let sql = std::fs::read_to_string(root().join("sql/labeled.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed, "{facts:?}");
    assert_eq!(facts.bounds.len(), 1);
    let judgment = evaluate::judgment_with_views(&facts.bounds[0], &catalog, &views);
    assert!(judgment.accepts.is_empty(), "{:?}", judgment.accepts);
    assert_eq!(judgment.offenders.len(), 1);
    assert_eq!(judgment.offenders[0].table, "labeled");
    assert_eq!(judgment.offenders[0].domain_over, None);
}

#[test]
fn max_domain_rows_deserializes_as_a_non_negative_integer() {
    let options: Options = serde_yaml::from_str("maxDomainRows: 5000").unwrap();
    assert_eq!(options.max_domain_rows, Some(5000));
    let options: Options = serde_yaml::from_str("{}").unwrap();
    assert_eq!(options.max_domain_rows, None);
    assert!(serde_yaml::from_str::<Options>("maxDomainRows: -1").is_err());
}
