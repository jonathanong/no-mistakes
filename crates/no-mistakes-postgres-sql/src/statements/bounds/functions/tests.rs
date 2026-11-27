mod projection_arguments;

use super::scalar::SCALAR;

#[test]
fn trusted_scalar_builtin_inventory_has_scalar_projection_facts() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/scalar-projection-builtins.sql"));
    let facts = crate::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), SCALAR.len());
    assert!(facts.bounds.iter().all(|fact| fact
        .query
        .items
        .iter()
        .all(|item| matches!(item.kind, super::SqlBoundItemKind::Other))));
}
