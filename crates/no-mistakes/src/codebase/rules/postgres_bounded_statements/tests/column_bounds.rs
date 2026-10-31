use super::{catalog, fixture_root};
use crate::codebase::postgres::extract_sql_statement_facts;
use crate::codebase::rules::postgres_bounded_statements::{
    column_bounds::ColumnBounds, evaluate, view_bounds::ViewBounds, Options,
};

#[test]
fn declared_keys_share_unique_key_caller_bound_proofs() {
    let catalog = catalog();
    let options: Options =
        serde_yaml::from_str("columnBounds: {orders.account_id: {reason: finite account scope}}")
            .unwrap();
    let views = ViewBounds::compile(&options.view_bounds, &catalog).unwrap();
    let mut declared = ViewBounds::compile(&options.view_bounds, &catalog).unwrap();
    declared.keys = ColumnBounds::compile(&options.column_bounds, &catalog).unwrap();
    let sql = std::fs::read_to_string(fixture_root().join("column-policy/sql/column-bounds.sql"))
        .unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 10);
    let count = |views: &ViewBounds| {
        facts
            .bounds
            .iter()
            .map(|fact| evaluate::offenders_with_views(fact, &catalog, views).len())
            .collect::<Vec<_>>()
    };
    assert_eq!(count(&views), [1, 1, 1, 1, 1, 2, 1, 2, 1, 1]);
    assert_eq!(count(&declared), [0, 0, 0, 0, 1, 2, 1, 2, 1, 1]);
}

#[test]
fn column_bounds_validate_catalog_identity_columns_and_reasons() {
    let catalog = catalog();
    for (yaml, expected) in [
        ("columnBounds: {orders.account_id: {reason: ' '}}", "needs a reason"),
        ("columnBounds: {orders: {reason: reviewed}}", "must name table.column"),
        ("columnBounds: {'orders.1': {reason: reviewed}}", "must name table.column"),
        ("columnBounds: {'orders.account_id trailing': {reason: reviewed}}", "must name table.column"),
        ("columnBounds: {'orders.\"': {reason: reviewed}}", "must name table.column"),
        ("columnBounds: {missing.id: {reason: reviewed}}", "catalog table"),
        ("columnBounds: {orders.missing: {reason: reviewed}}", "catalog column"),
        ("columnBounds: {orders.account_id: {reason: reviewed}, public.orders.account_id: {reason: reviewed}}", "duplicate catalog column"),
    ] {
        let options: Options = serde_yaml::from_str(yaml).unwrap();
        let error = ColumnBounds::compile(&options.column_bounds, &catalog).err().unwrap().to_string();
        assert!(error.contains(expected), "{error}");
    }
    assert!(serde_yaml::from_str::<Options>("columnBounds: {orders.account_id: {}}").is_err());
}

#[test]
fn declared_keys_preserve_quoted_and_qualified_catalog_identity() {
    let root = fixture_root().join("column-policy");
    let path = root.join("schema-column-bounds.json");
    let sources = crate::codebase::rules::source_store_for_files(&[path]);
    let catalog = super::SchemaCatalog::load(&root, "schema-column-bounds.json", &sources).unwrap();
    let options: Options = serde_yaml::from_str(
        "columnBounds: {'public.orders.account_id': {reason: scoped}, '\"Mixed.Table\".\"Owner Id\"': {reason: scoped}}",
    ).unwrap();
    let declared = ColumnBounds::compile(&options.column_bounds, &catalog).unwrap();
    assert_eq!(
        declared.columns("public.orders").collect::<Vec<_>>(),
        ["account_id"]
    );
    assert_eq!(
        declared.columns("\"Mixed.Table\"").collect::<Vec<_>>(),
        ["Owner Id"]
    );
    let options: Options =
        serde_yaml::from_str("columnBounds: {orders.account_id: {reason: scoped}}").unwrap();
    assert!(ColumnBounds::compile(&options.column_bounds, &catalog)
        .err()
        .unwrap()
        .to_string()
        .contains("catalog table"));
}

#[test]
fn declared_column_bounds_a_composite_unique_prefix_only() {
    let root = fixture_root().join("column-policy");
    let sources =
        crate::codebase::rules::source_store_for_files(&[root.join("schema-column-bounds.json")]);
    let catalog = super::SchemaCatalog::load(&root, "schema-column-bounds.json", &sources).unwrap();
    let options: Options = serde_yaml::from_str(
        "columnBounds: {memberships.account_id: {reason: bounded memberships}}",
    )
    .unwrap();
    let mut declared = ViewBounds::compile(&options.view_bounds, &catalog).unwrap();
    declared.keys = ColumnBounds::compile(&options.column_bounds, &catalog).unwrap();
    let sql = std::fs::read_to_string(root.join("sql/column-bounds-composite-prefix.sql")).unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let count = |views: &ViewBounds| {
        facts
            .bounds
            .iter()
            .map(|fact| evaluate::offenders_with_views(fact, &catalog, views).len())
            .collect::<Vec<_>>()
    };
    assert_eq!(count(&ViewBounds::default()), [1, 1, 0]);
    assert_eq!(count(&declared), [0, 1, 0]);
}

#[test]
fn correlated_declared_columns_require_a_known_lateral_driver() {
    let catalog = catalog();
    let options: Options = serde_yaml::from_str(
        "columnBounds: {orders.account_id: {reason: bounded orders per account}}",
    )
    .unwrap();
    let mut declared = ViewBounds::compile(&options.view_bounds, &catalog).unwrap();
    declared.keys = ColumnBounds::compile(&options.column_bounds, &catalog).unwrap();
    let sql = std::fs::read_to_string(
        fixture_root().join("column-policy/sql/column-bounds-correlated.sql"),
    )
    .unwrap();
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let count = |views: &ViewBounds| {
        facts
            .bounds
            .iter()
            .map(|fact| evaluate::offenders_with_views(fact, &catalog, views).len())
            .collect::<Vec<_>>()
    };
    assert_eq!(count(&ViewBounds::default()), [1, 2, 1, 1]);
    assert_eq!(count(&declared), [0, 1, 1, 1]);
}
