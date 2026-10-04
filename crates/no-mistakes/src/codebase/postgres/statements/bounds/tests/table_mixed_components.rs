#[test]
fn table_components_preserve_independent_quoting() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-mixed-components.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let mut names = Vec::new();
    fn tables(query: &crate::codebase::postgres::SqlBoundQuery, names: &mut Vec<String>) {
        use crate::codebase::postgres::SqlBoundItemKind;
        for item in &query.items {
            match &item.kind {
                SqlBoundItemKind::Table(name) => names.push(name.clone()),
                SqlBoundItemKind::Query(query) => tables(query, names),
                _ => {}
            }
        }
    }
    for bound in &facts.bounds {
        tables(&bound.query, &mut names);
    }
    assert_eq!(
        names,
        [
            "\"Tenant\".accounts",
            "tenant.\"Accounts\"",
            "\"Tenant\".accounts",
            "tenant.\"Accounts\"",
            "\"Tenant.Name\".accounts",
            "tenant.\"Account.Name\""
        ]
    );
}
